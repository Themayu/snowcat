use crate::signal_vec::wrap_poll_result;
use futures_signals::signal_vec::{SignalVec, VecDiff};
use pin_project::pin_project;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Poll, Context};

// signalvec item ordering, using std::cmp::Ordering
//
// order_fn(left, right) -> less
// left | right
//
// order_fn(left, right) -> equal
// unstable, unimplemented
//
// order_fn(left, right) -> greater
// right | left
#[must_use = "Merge2 does nothing unless polled"]
#[pin_project(project = Merge2Proj)]
#[derive(Debug)]
pub struct Merge2<Left, Right, OrderFn>
where Left: SignalVec,
      Right: SignalVec,
      Left::Item: Debug + Clone,
      Right::Item: Debug + Clone,
      OrderFn: Fn(&Left::Item, &Right::Item) -> Ordering,
{
	items: Vec<MergedVecItem<Left::Item, Right::Item>>,
	order_fn: OrderFn,

	pending_ops_left: VecDeque<VecDiff<Left::Item>>,
	pending_ops_right: VecDeque<VecDiff<Right::Item>>,

	#[pin]
	left: Left,

	#[pin]
	right: Right,
}

impl<Left, Right, OrderFn> Merge2<Left, Right, OrderFn>
where Left: SignalVec,
      Right: SignalVec,
      Left::Item: Debug + Clone,
      Right::Item: Debug + Clone,
      OrderFn: Fn(&Left::Item, &Right::Item) -> Ordering,
{
	pub fn new(left: Left, right: Right, order_fn: OrderFn) -> Self {
		Merge2 {
			left,
			order_fn,
			right,

			items: vec![],
			pending_ops_left: VecDeque::new(),
			pending_ops_right: VecDeque::new(),
		}
	}
}

impl<Left, Right, OrderFn> SignalVec for Merge2<Left, Right, OrderFn>
where Left: SignalVec,
      Right: SignalVec,
      Left::Item: Debug + Clone,
      Right::Item: Debug + Clone,
      OrderFn: Fn(&Left::Item, &Right::Item) -> Ordering,
{
	type Item = MergedVecItem<Left::Item, Right::Item>;

	// TODO: handle merging two SignalVecs of potentially different types into a single SignalVec that may return either
	fn poll_vec_change(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<VecDiff<Self::Item>>> {
		let Merge2Proj {
			items,
			order_fn,
			pending_ops_left,
			pending_ops_right,
			mut left,
			mut right,
		} = self.project();

		let poll_result = {
			let next_left = pending_ops_left.pop_front()
				.map(|item| wrap_poll_result(item))
				.unwrap_or_else(|| left.as_mut().poll_vec_change(cx));

			let next_right = pending_ops_right.pop_front()
				.map(|item| wrap_poll_result(item))
				.unwrap_or_else(|| right.as_mut().poll_vec_change(cx));

			match (next_left, next_right) {
				(Poll::Pending, Poll::Pending)
				| (Poll::Ready(None), Poll::Pending)
				| (Poll::Pending, Poll::Ready(None)) => Poll::Pending,

				(Poll::Ready(None), Poll::Ready(None)) => Poll::Ready(None),

				// left is ready, right is not
				(Poll::Ready(Some(left)), Poll::Pending)
				| (Poll::Ready(Some(left)), Poll::Ready(None)) => {
					Poll::Ready(Some(MergedVecItem::Left(left)))
				},

				// right is ready, left is not
				(Poll::Pending, Poll::Ready(Some(right)))
				| (Poll::Ready(None), Poll::Ready(Some(right))) => {
					Poll::Ready(Some(MergedVecItem::Right(right)))
				},

				// both are ready, choose one and stash the other for next
				// iteration
				(
					Poll::Ready(Some(left)),
					Poll::Ready(Some(right)),
				) => {
					let priority = get_priority(&left, &right);
					log::trace!(
						"{file}:{line} [{module}::<Merge2 as SignalVec>::poll_vec_change] priority is {priority:?}",
						file = file!(), line = line!(), module = module_path!(),
					);

					match priority {
						MergedVecItem::Left(base) => {
							pending_ops_right.push_front(right.clone());
							Poll::Ready(Some(MergedVecItem::Left(base.clone())))
						},

						MergedVecItem::Right(other) => {
							pending_ops_left.push_front(left.clone());
							Poll::Ready(Some(MergedVecItem::Right(other.clone())))
						}
					}
				},
			}
		};

		let op = match poll_result {
			Poll::Ready(Some(MergedVecItem::Left(op))) => {
				log::trace!(
					"{file}:{line} [{module}::<Merge2 as SignalVec>::poll_vec_change] op is Left({op:?})",
					file = file!(), line = line!(), module = module_path!(),
				);

				MergedVecDiff::from_left(op)
			},

			Poll::Ready(Some(MergedVecItem::Right(op))) => {
				log::trace!(
					"{file}:{line} [{module}::<Merge2 as SignalVec>::poll_vec_change] op is Right({op:?})",
					file = file!(), line = line!(), module = module_path!(),
				);

				MergedVecDiff::from_right(op)
			},

			Poll::Ready(None) => return Poll::Ready(None),
			Poll::Pending => return Poll::Pending,
		};

		// VecDiff might become non-exhaustive in the future
		#[allow(unreachable_patterns)]
		return Poll::Ready(match &op {
			MergedVecDiff::Replace(replace) => {
				let new_values = replace.values();

				let mut retained: VecDeque<MergedVecItem<Left::Item, Right::Item>> = match new_values {
					MergedVecItem::Left(_) => items.drain(..)
						.filter(|item| matches!(item, MergedVecItem::Right(_)))
						.collect(),

					MergedVecItem::Right(_) => items.drain(..)
						.filter(|item| matches!(item, MergedVecItem::Left(_)))
						.collect(),
				};

				match new_values {
					MergedVecItem::Left(values) => values.into_iter().for_each(|item| loop {
						let other = retained.front();
						let ordering = other.map(|value| order_fn(&item, value.as_right())).unwrap_or(Ordering::Less);

						match ordering {
							Ordering::Less => break items.push(to_left(item)),
							Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),

							Ordering::Greater => {
								// we're working on this exact item
								items.push(retained.pop_front().unwrap());
							},
						};
					}),

					MergedVecItem::Right(values) => values.into_iter().for_each(|item| loop {
						let other = retained.front();
						let ordering = other.map(|value| order_fn(value.as_left(), &item)).unwrap_or(Ordering::Greater);

						match ordering {
							Ordering::Less => {
								// we're working on this exact item
								items.push(retained.pop_front().unwrap());
							}

							Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
							Ordering::Greater => break items.push(to_right(item)),
						}
					}),
				};

				// inject any remaining retained elements into the end of
				// the collection.
				items.extend(retained.into_iter());

				Some(VecDiff::Replace { values: items.clone() })
			},

			MergedVecDiff::InsertAt(insert_at) => {
				let (target, _unused) = get_index(items, &op);

				Some(traverse_insert_into_at(items, insert_at.value().cloned(), target, order_fn))
			},

			MergedVecDiff::UpdateAt(update_at) => {
				let (index, _unused) = get_index(items, &op);
				let value = update_at.value().cloned();

				items[index] = value.clone();
				Some(VecDiff::UpdateAt { index, value })
			},

			MergedVecDiff::RemoveAt(_) => {
				let (index, _unused) = get_index(items, &op);

				items.remove(index);
				Some(VecDiff::RemoveAt { index })
			},

			MergedVecDiff::MoveItem(_) => {
				let (old_index, new_index) = get_index(items, &op);

				let value = items.remove(old_index);
				items.insert(new_index, value);

				Some(VecDiff::Move { old_index, new_index })
			},

			MergedVecDiff::PushItem(push_item) => {
				Some(traverse_push(items, push_item.value().cloned(), order_fn))
			},

			MergedVecDiff::PopItem(pop_item) => {
				let op = match get_last_index(items, pop_item.side()) {
					Some(index) if index < items.len() - 1 => {
						items.remove(index);
						VecDiff::RemoveAt { index }
					},

					Some(_) => {
						items.pop();
						VecDiff::Pop {}
					},

					None => unreachable!("item is guaranteed to exist"),
				};

				Some(op)
			},

			MergedVecDiff::Clear(clear) => {
				items.retain(|item| match item {
					MergedVecItem::Left(_) if matches!(clear, MergedVecClear::Right { .. }) => true,
					MergedVecItem::Right(_) if matches!(clear, MergedVecClear::Left { .. }) => true,
					_ => false,
				});

				if items.len() == 0 {
					Some(VecDiff::Clear {})
				} else {
					Some(VecDiff::Replace { values: items.clone() })
				}
			},
		});
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergedVecItem<Left, Right>
where Left: Debug,
      Right: Debug,
{
	Left(Left),
	Right(Right)
}

impl<Left, Right> Copy for MergedVecItem<Left, Right>
where Left: Debug + Copy,
      Right: Debug + Copy, {}

impl<Left, Right> MergedVecItem<Left, Right>
where Left: Debug,
      Right: Debug, {
	/// Attempt to reference a `MergedVecItem::Left` value, panicking if `self`
	/// is `MergedVecItem::Right`.
	fn as_left(&self) -> &Left {
		match self {
			MergedVecItem::Left(value) => &value,
			MergedVecItem::Right(_) => panic!("called `MergedVecItem::as_left()` on a `Right` value"),
		}
	}

	/// Attempt to reference a `MergedVecItem::Right` value, panicking if `self`
	/// is `MergedVecItem::Left`
	fn as_right(&self) -> &Right {
		match self {
			MergedVecItem::Left(_) => panic!("called `MergedVecItem::as_right()` on a `Left` value"),
			MergedVecItem::Right(value) => &value,
		}
	}

	/// Determines if the value inside this `MergedVecItem` is a
	/// `MergedVecItem::Left` value
	pub fn is_left(&self) -> bool {
		matches!(self, MergedVecItem::Left(_))
	}

	/// Determines if the value inside this `MergedVecItem` is a
	/// `MergedVecItem::Right` value
	pub fn is_right(&self) -> bool {
		matches!(self, MergedVecItem::Right(_))
	}

	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	pub fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecItem::Left(_) => MergedVecItem::Left(()),
			MergedVecItem::Right(_) => MergedVecItem::Right(()),
		}
	}

	/// Attempt to unwrap a `MergedVecItem::Left` value, panicking if `self`
	/// is `MergedVecItem::Right`.
	pub fn unwrap_left(self) -> Left {
		match self {
			MergedVecItem::Left(value) => value,
			MergedVecItem::Right(_) => panic!("called `MergedVecItem::unwrap_left()` on a `Right` value"),
		}
	}

	/// Attempt to unwrap a `MergedVecItem::Right` value, panicking if `self`
	/// is `MergedVecItem::Left`
	pub fn unwrap_right(self) -> Right {
		match self {
			MergedVecItem::Left(_) => panic!("called `MergedVecItem::unwrap_right()` on a `Left` value"),
			MergedVecItem::Right(value) => value,
		}
	}
}

impl<Left, Right> MergedVecItem<&Left, &Right>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	/// Clones the internal value of this MergedVecItem.
	fn cloned(&self) -> MergedVecItem<Left, Right> {
		match self {
			MergedVecItem::Left(value) => to_left(value),
			MergedVecItem::Right(value) => to_right(value),
		}
	}
}

/// A version of [`VecDiff`](futures_signals::signal_vec::VecDiff) that can be
/// used for processing a merge.
#[derive(Debug, Clone)]
enum MergedVecDiff<Left, Right> {
	Replace(MergedVecReplace<Left, Right>),
	InsertAt(MergedVecInsertAt<Left, Right>),
	UpdateAt(MergedVecUpdateAt<Left, Right>),
	RemoveAt(MergedVecRemoveAt<Left, Right>),
	MoveItem(MergedVecMoveItem<Left, Right>),
	PushItem(MergedVecPushItem<Left, Right>),
	PopItem(MergedVecPopItem<Left, Right>),
	Clear(MergedVecClear<Left, Right>),
}

impl<Left, Right> MergedVecDiff<Left, Right> {
	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecDiff::Replace(replace) => replace.side(),
			MergedVecDiff::InsertAt(insert_at) => insert_at.side(),
			MergedVecDiff::UpdateAt(update_at) => update_at.side(),
			MergedVecDiff::RemoveAt(remove_at) => remove_at.side(),
			MergedVecDiff::MoveItem(move_item) => move_item.side(),
			MergedVecDiff::PushItem(push_item) => push_item.side(),
			MergedVecDiff::PopItem(pop_item) => pop_item.side(),
			MergedVecDiff::Clear(clear) => clear.side(),
		}
	}
}

impl<Left, Right> MergedVecDiff<Left, Right>
where Left: Debug,
      Right: Debug,
{
	fn from_left(left: VecDiff<Left>) -> MergedVecDiff<Left, Right> {
		// VecDiff might become non-exhaustive in the future
		#[allow(unreachable_patterns)]
		match left {
			VecDiff::Replace { values } =>
				Self::Replace(MergedVecReplace::Left { values }),

			VecDiff::InsertAt { index, value } =>
				Self::InsertAt(MergedVecInsertAt::Left { index, value }),

			VecDiff::UpdateAt { index, value } =>
				Self::UpdateAt(MergedVecUpdateAt::Left { index, value }),

			VecDiff::RemoveAt { index } =>
				Self::RemoveAt(MergedVecRemoveAt::Left { index, _marker: PhantomData }),

			VecDiff::Move { old_index, new_index } =>
				Self::MoveItem(MergedVecMoveItem::Left { old_index, new_index, _marker: PhantomData }),

			VecDiff::Push { value } =>
				Self::PushItem(MergedVecPushItem::Left { value }),

			VecDiff::Pop {} =>
				Self::PopItem(MergedVecPopItem::Left { _marker: PhantomData }),

			VecDiff::Clear {} =>
				Self::Clear(MergedVecClear::Left { _marker: PhantomData }),

			_ => todo!("convert diff type to Left: {left:?}"),
		}
	}

	fn from_right(right: VecDiff<Right>) -> MergedVecDiff<Left, Right> {
		// VecDiff might become non-exhaustive in the future
		#[allow(unreachable_patterns)]
		match right {
			VecDiff::Replace { values } =>
				Self::Replace(MergedVecReplace::Right { values }),

			VecDiff::InsertAt { index, value } =>
				Self::InsertAt(MergedVecInsertAt::Right { index, value }),

			VecDiff::UpdateAt { index, value } =>
				Self::UpdateAt(MergedVecUpdateAt::Right { index, value }),

			VecDiff::RemoveAt { index } =>
				Self::RemoveAt(MergedVecRemoveAt::Right { index, _marker: PhantomData }),

			VecDiff::Move { old_index, new_index } =>
				Self::MoveItem(MergedVecMoveItem::Right { old_index, new_index, _marker: PhantomData }),

			VecDiff::Push { value } =>
				Self::PushItem(MergedVecPushItem::Right { value }),

			VecDiff::Pop {} =>
				Self::PopItem(MergedVecPopItem::Right { _marker: PhantomData }),

			VecDiff::Clear {} =>
				Self::Clear(MergedVecClear::Right { _marker: PhantomData }),

			_ => todo!("convert diff type to Right: {right:?}"),
		}
	}
}

#[derive(Debug, Clone)]
enum MergedVecReplace<Left, Right> {
	Left {
		values: Vec<Left>,
	},

	Right {
		values: Vec<Right>,
	},
}

impl<Left, Right> MergedVecReplace<Left, Right> {
	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecReplace::Left { .. } => MergedVecItem::Left(()),
			MergedVecReplace::Right { .. } => MergedVecItem::Right(()),
		}
	}
}

impl<Left, Right> MergedVecReplace<Left, Right>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	fn values(&self) -> MergedVecItem<&Vec<Left>, &Vec<Right>> {
		match self {
			MergedVecReplace::Left { values } => into_left(values),
			MergedVecReplace::Right { values } => into_right(values),
		}
	}
}

#[derive(Debug, Clone)]
enum MergedVecInsertAt<Left, Right> {
	Left {
		index: usize,
		value: Left,
	},

	Right {
		index: usize,
		value: Right,
	},
}

impl<Left, Right> MergedVecInsertAt<Left, Right> {
	fn index(&self) -> usize {
		match self {
			MergedVecInsertAt::Left { index, .. } => *index,
			MergedVecInsertAt::Right { index, .. } => *index,
		}
	}

	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecInsertAt::Left { .. } => MergedVecItem::Left(()),
			MergedVecInsertAt::Right { .. } => MergedVecItem::Right(()),
		}
	}
}

impl<Left, Right> MergedVecInsertAt<Left, Right>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	fn value(&self) -> MergedVecItem<&Left, &Right> {
		match self {
			MergedVecInsertAt::Left { value, .. } => into_left(value),
			MergedVecInsertAt::Right { value, .. } => into_right(value),
		}
	}
}

#[derive(Debug, Clone)]
enum MergedVecUpdateAt<Left, Right> {
	Left {
		index: usize,
		value: Left,
	},

	Right {
		index: usize,
		value: Right,
	},
}

impl<Left, Right> MergedVecUpdateAt<Left, Right> {
	fn index(&self) -> usize {
		match self {
			MergedVecUpdateAt::Left { index, .. } => *index,
			MergedVecUpdateAt::Right { index, .. } => *index,
		}
	}

	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecUpdateAt::Left { .. } => MergedVecItem::Left(()),
			MergedVecUpdateAt::Right { .. } => MergedVecItem::Right(()),
		}
	}
}

impl<Left, Right> MergedVecUpdateAt<Left, Right>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	fn value(&self) -> MergedVecItem<&Left, &Right> {
		match self {
			MergedVecUpdateAt::Left { value, .. } => into_left(value),
			MergedVecUpdateAt::Right { value, .. } => into_right(value),
		}
	}
}

#[derive(Debug, Clone)]
enum MergedVecRemoveAt<Left, Right> {
	Left {
		index: usize,
		_marker: PhantomData<Left>,
	},

	Right {
		index: usize,
		_marker: PhantomData<Right>,
	}
}

impl<Left, Right> MergedVecRemoveAt<Left, Right> {
	fn index(&self) -> usize {
		match self {
			MergedVecRemoveAt::Left { index, .. } => *index,
			MergedVecRemoveAt::Right { index, .. } => *index,
		}
	}

	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecRemoveAt::Left { .. } => MergedVecItem::Left(()),
			MergedVecRemoveAt::Right { .. } => MergedVecItem::Left(()),
		}
	}
}

#[derive(Debug, Clone)]
enum MergedVecMoveItem<Left, Right> {
	Left {
		old_index: usize,
		new_index: usize,
		_marker: PhantomData<Left>,
	},

	Right {
		old_index: usize,
		new_index: usize,
		_marker: PhantomData<Right>,
	}
}

impl<Left, Right> MergedVecMoveItem<Left, Right> {
	fn new_index(&self) -> usize {
		match self {
			MergedVecMoveItem::Left { new_index, .. } => *new_index,
			MergedVecMoveItem::Right { new_index, .. } => *new_index,
		}
	}

	fn old_index(&self) -> usize {
		match self {
			MergedVecMoveItem::Left { old_index, .. } => *old_index,
			MergedVecMoveItem::Right { old_index, .. } => *old_index,
		}
	}

	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecMoveItem::Left { .. } => MergedVecItem::Left(()),
			MergedVecMoveItem::Right { .. } => MergedVecItem::Right(()),
		}
	}
}

#[derive(Debug, Clone)]
enum MergedVecPushItem<Left, Right> {
	Left {
		value: Left,
	},

	Right {
		value: Right,
	},
}

impl<Left, Right> MergedVecPushItem<Left, Right> {
	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecPushItem::Left { .. } => MergedVecItem::Left(()),
			MergedVecPushItem::Right { .. } => MergedVecItem::Right(()),
		}
	}
}

impl<Left, Right> MergedVecPushItem<Left, Right>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	fn value(&self) -> MergedVecItem<&Left, &Right> {
		match self {
			MergedVecPushItem::Left { value } => into_left(value),
			MergedVecPushItem::Right { value } => into_right(value),
		}
	}
}

#[derive(Debug, Clone)]
enum MergedVecPopItem<Left, Right> {
	Left {
		_marker: PhantomData<Left>,
	},

	Right {
		_marker: PhantomData<Right>,
	},
}

impl<Left, Right> MergedVecPopItem<Left, Right> {
	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecPopItem::Left { .. } => MergedVecItem::Left(()),
			MergedVecPopItem::Right { .. } => MergedVecItem::Right(()),
		}
	}
}

#[derive(Debug, Clone)]
enum MergedVecClear<Left, Right> {
	Left {
		_marker: PhantomData<Left>,
	},

	Right {
		_marker: PhantomData<Right>,
	},
}

impl<Left, Right> MergedVecClear<Left, Right> {
	/// Returns a zero-size `MergedVecItem` to indicate the active side.
	fn side(&self) -> MergedVecItem<(), ()> {
		match self {
			MergedVecClear::Left { .. } => MergedVecItem::Left(()),
			MergedVecClear::Right { .. } => MergedVecItem::Right(()),
		}
	}
}

/// Prioritise one operation over the other, based on the order they appear in
/// [`VecDiff<T>`](futures_signals::signal_vec::VecDiff).
fn get_priority<'op, Left, Right>(left: &'op VecDiff<Left>, right: &'op VecDiff<Right>)
-> MergedVecItem<&'op VecDiff<Left>, &'op VecDiff<Right>>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	// VecDiff might become non-exhaustive in the future
	#[allow(unreachable_patterns)]
	match (left, right) {
		(op @ VecDiff::Replace { .. }, VecDiff::Replace { .. }) => into_left(&op),
		(op @ VecDiff::Replace { .. }, VecDiff::InsertAt { .. }) => into_left(&op),
		(op @ VecDiff::Replace { .. }, VecDiff::UpdateAt { .. }) => into_left(&op),
		(op @ VecDiff::Replace { .. }, VecDiff::RemoveAt { .. }) => into_left(&op),
		(op @ VecDiff::Replace { .. }, VecDiff::Move { .. }) => into_left(&op),
		(op @ VecDiff::Replace { .. }, VecDiff::Push { .. }) => into_left(&op),
		(op @ VecDiff::Replace { .. }, VecDiff::Pop {}) => into_left(&op),
		(VecDiff::Replace { .. }, op @ VecDiff::Clear {}) => into_right(&op),

		(VecDiff::InsertAt { .. }, op @ VecDiff::Replace { .. }) => into_right(&op),
		(op @ VecDiff::InsertAt { .. }, VecDiff::InsertAt { .. }) => into_left(&op),
		(op @ VecDiff::InsertAt { .. }, VecDiff::UpdateAt { .. }) => into_left(&op),
		(op @ VecDiff::InsertAt { .. }, VecDiff::RemoveAt { .. }) => into_left(&op),
		(op @ VecDiff::InsertAt { .. }, VecDiff::Move { .. }) => into_left(&op),
		(op @ VecDiff::InsertAt { .. }, VecDiff::Push { .. }) => into_left(&op),
		(op @ VecDiff::InsertAt { .. }, VecDiff::Pop {}) => into_left(&op),
		(VecDiff::InsertAt { .. }, op @ VecDiff::Clear {}) => into_right(&op),

		(VecDiff::UpdateAt { .. }, op @ VecDiff::Replace { .. }) => into_right(&op),
		(VecDiff::UpdateAt { .. }, op @ VecDiff::InsertAt { .. }) => into_right(&op),
		(op @ VecDiff::UpdateAt { .. }, VecDiff::UpdateAt { .. }) => into_left(&op),
		(op @ VecDiff::UpdateAt { .. }, VecDiff::RemoveAt { .. }) => into_left(&op),
		(op @ VecDiff::UpdateAt { .. }, VecDiff::Move { .. }) => into_left(&op),
		(op @ VecDiff::UpdateAt { .. }, VecDiff::Push { .. }) => into_left(&op),
		(op @ VecDiff::UpdateAt { .. }, VecDiff::Pop {}) => into_left(&op),
		(VecDiff::UpdateAt { .. }, op @ VecDiff::Clear {}) => into_right(&op),

		(VecDiff::RemoveAt { .. }, op @ VecDiff::Replace { .. }) => into_right(&op),
		(VecDiff::RemoveAt { .. }, op @ VecDiff::InsertAt { .. }) => into_right(&op),
		(VecDiff::RemoveAt { .. }, op @ VecDiff::UpdateAt { .. }) => into_right(&op),
		(op @ VecDiff::RemoveAt { .. }, VecDiff::RemoveAt { .. }) => into_left(&op),
		(op @ VecDiff::RemoveAt { .. }, VecDiff::Move { .. }) => into_left(&op),
		(op @ VecDiff::RemoveAt { .. }, VecDiff::Push { .. }) => into_left(&op),
		(op @ VecDiff::RemoveAt { .. }, VecDiff::Pop {}) => into_left(&op),
		(VecDiff::RemoveAt { .. }, op @ VecDiff::Clear {}) => into_right(&op),

		(VecDiff::Move { .. }, op @ VecDiff::Replace { .. }) => into_right(&op),
		(VecDiff::Move { .. }, op @ VecDiff::InsertAt { .. }) => into_right(&op),
		(VecDiff::Move { .. }, op @ VecDiff::UpdateAt { .. }) => into_right(&op),
		(VecDiff::Move { .. }, op @ VecDiff::RemoveAt { .. }) => into_right(&op),
		(op @ VecDiff::Move { .. }, VecDiff::Move { .. }) => into_left(&op),
		(op @ VecDiff::Move { .. }, VecDiff::Push { .. }) => into_left(&op),
		(op @ VecDiff::Move { .. }, VecDiff::Pop {}) => into_left(&op),
		(VecDiff::Move { .. }, op @ VecDiff::Clear {}) => into_right(&op),

		(VecDiff::Push { .. }, op @ VecDiff::Replace { .. }) => into_right(&op),
		(VecDiff::Push { .. }, op @ VecDiff::InsertAt { .. }) => into_right(&op),
		(VecDiff::Push { .. }, op @ VecDiff::UpdateAt { .. }) => into_right(&op),
		(VecDiff::Push { .. }, op @ VecDiff::RemoveAt { .. }) => into_right(&op),
		(VecDiff::Push { .. }, op @ VecDiff::Move { .. }) => into_right(&op),
		(op @ VecDiff::Push { .. }, VecDiff::Push { .. }) => into_left(&op),
		(op @ VecDiff::Push { .. }, VecDiff::Pop {}) => into_left(&op),
		(VecDiff::Push { .. }, op @ VecDiff::Clear {}) => into_right(&op),

		(VecDiff::Pop {}, op @ VecDiff::Replace { .. }) => into_right(&op),
		(VecDiff::Pop {}, op @ VecDiff::InsertAt { .. }) => into_right(&op),
		(VecDiff::Pop {}, op @ VecDiff::UpdateAt { .. }) => into_right(&op),
		(VecDiff::Pop {}, op @ VecDiff::RemoveAt { .. }) => into_right(&op),
		(VecDiff::Pop {}, op @ VecDiff::Move { .. }) => into_right(&op),
		(VecDiff::Pop {}, op @ VecDiff::Push { .. }) => into_right(&op),
		(op @ VecDiff::Pop {}, VecDiff::Pop {}) => into_left(&op),
		(VecDiff::Pop {}, op @ VecDiff::Clear {}) => into_right(&op),

		(VecDiff::Clear {}, op @ VecDiff::Replace { .. }) => into_right(&op),
		(VecDiff::Clear {}, op @ VecDiff::InsertAt { .. }) => into_right(&op),
		(VecDiff::Clear {}, op @ VecDiff::UpdateAt { .. }) => into_right(&op),
		(VecDiff::Clear {}, op @ VecDiff::RemoveAt { .. }) => into_right(&op),
		(VecDiff::Clear {}, op @ VecDiff::Move { .. }) => into_right(&op),
		(VecDiff::Clear {}, op @ VecDiff::Push { .. }) => into_right(&op),
		(VecDiff::Clear {}, op @ VecDiff::Pop {}) => into_right(&op),
		(op @ VecDiff::Clear {}, VecDiff::Clear {}) => into_left(&op),

		(_, _) => todo!("diff type {:?}", (left, right))
	}
}

/// Given an operation that carries positions in the SignalVec it comes from,
/// transforms those indices into positions in the combined output of the
/// Merge2 SignalVed
///
/// # Panics
///
/// Only some operations carry positions with them. Because it is not possible
/// to statically limit which enum variants can be passed to a function right
/// now, this function will panic if given an operation that does not carry any
/// positions.
fn get_index<Left, Right>(
	items: &[MergedVecItem<Left, Right>],
	op: &MergedVecDiff<Left, Right>,
) -> (usize, usize)
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	match op {
		MergedVecDiff::InsertAt(insert_at) => {
			let mut current = 0;
			let target = insert_at.index();
			for (idx, item) in items.into_iter().enumerate() {
				let is_match = match op.side() {
					MergedVecItem::Left(()) => matches!(item, MergedVecItem::Left(_)),
					MergedVecItem::Right(()) => matches!(item, MergedVecItem::Right(_)),
				};

				if is_match {
					current += 1;

					if current == target {
						return (idx + 1, 0);
					}
				}
			}

			(items.len(), 0)
		},

		MergedVecDiff::UpdateAt(update_at) => {
			let mut current = 0;
			let target = update_at.index();

			for (idx, item) in items.into_iter().enumerate() {
				let is_match = match op.side() {
					MergedVecItem::Left(()) => matches!(item, MergedVecItem::Left(_)),
					MergedVecItem::Right(()) => matches!(item, MergedVecItem::Right(_)),
				};

				if is_match {
					if current == target {
						return (idx, 0);
					}

					current += 1;
				}
			}

			unreachable!("update requires index to be in bounds")
		},

		MergedVecDiff::RemoveAt(remove_at) => {
			let mut current = 0;
			let target = remove_at.index();

			for (idx, item) in items.into_iter().enumerate() {
				let is_match = match op.side() {
					MergedVecItem::Left(()) => matches!(item, MergedVecItem::Left(_)),
					MergedVecItem::Right(()) => matches!(item, MergedVecItem::Right(_)),
				};

				if is_match {
					if current == target {
						return (idx, 0);
					}

					current += 1;
				}
			}

			unreachable!("remove requires index to be in bounds");
		}

		MergedVecDiff::MoveItem(move_item) => {
			// find old_index
			let mut old_idx = None;
			let mut current_old = 0;
			let target_old = move_item.old_index();

			for (idx, item) in items.into_iter().enumerate() {
				let is_match = match op.side() {
					MergedVecItem::Left(()) => matches!(item, MergedVecItem::Left(_)),
					MergedVecItem::Right(()) => matches!(item, MergedVecItem::Right(_)),
				};

				if is_match {
					if current_old == target_old {
						old_idx = Some(idx);
						break;
					}

					current_old += 1;
				}
			}

			let old_idx = match old_idx {
				Some(index) => index,
				None => unreachable!("move item requires old_index to be in bounds"),
			};

			// find new_index
			let mut new_idx = None;
			let mut current_new = 0;
			let target_new = move_item.new_index();

			for (idx, item) in items.into_iter().enumerate() {
				let is_match = match op.side() {
					MergedVecItem::Left(()) => matches!(item, MergedVecItem::Left(_)),
					MergedVecItem::Right(()) => matches!(item, MergedVecItem::Right(_)),
				};

				if is_match {
					if current_new == target_new {
						new_idx = Some(idx);
						break;
					}

					current_new += 1;
				}
			}

			let new_idx = match new_idx {
				Some(index) => index,
				None => items.len(),
			};

			(old_idx, new_idx)
		},

		op => unimplemented!("{op:?} does not specify an index to be transformed"),
	}
}

fn get_last_index<Left, Right>(items: &[MergedVecItem<Left, Right>], side: MergedVecItem<(), ()>) -> Option<usize>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	for (idx, item) in items.into_iter().enumerate().rev() {
		let is_match = match side {
			MergedVecItem::Left(()) => matches!(item, MergedVecItem::Left(_)),
			MergedVecItem::Right(()) => matches!(item, MergedVecItem::Right(_)),
		};

		if is_match {
			return Some(idx);
		}
	}

	None
}

fn insert_into_at<Left, Right>(items: &mut Vec<MergedVecItem<Left, Right>>, index: usize, value: MergedVecItem<Left, Right>)
-> VecDiff<MergedVecItem<Left, Right>>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	items.insert(index, value.clone());
	VecDiff::InsertAt { index, value }
}

fn into_left<Left, Right>(item: Left) -> MergedVecItem<Left, Right>
where Left: Debug,
      Right: Debug,
{
	MergedVecItem::Left(item)
}

fn into_right<Left, Right>(item: Right) -> MergedVecItem<Left, Right>
where Left: Debug,
      Right: Debug,
{
	MergedVecItem::Right(item)
}

fn push<Left, Right>(items: &mut Vec<MergedVecItem<Left, Right>>, value: MergedVecItem<Left, Right>)
-> VecDiff<MergedVecItem<Left, Right>>
where Left: Debug + Clone,
	  Right: Debug + Clone,
{
	items.push(value.clone());
	VecDiff::Push { value }
}

fn to_left<Left, Right>(item: &Left) -> MergedVecItem<Left, Right>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	into_left(item.clone())
}

fn to_right<Left, Right>(item: &Right) -> MergedVecItem<Left, Right>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	into_right(item.clone())
}

fn traverse_insert_into_at<Left, Right, OrderFn>(
	items: &mut Vec<MergedVecItem<Left, Right>>,
	value: MergedVecItem<Left, Right>,
	target: usize,
	order_fn: &mut OrderFn,
) -> VecDiff<MergedVecItem<Left, Right>>
where Left: Debug + Clone,
      Right: Debug + Clone,
	  OrderFn: Fn(&Left, &Right) -> Ordering,
{
	let is_end = target == items.len();
	let is_start = target == 0;

	log::trace!(
		"{file}:{line} [{module}::traverse_insert_into_at] is_start = {is_start}, is_end = {is_end}",
		file = file!(), line = line!(), module = module_path!(),
	);

	if items.len() == 0 {
		log::trace!(
			"{file}:{line} [{module}::traverse_insert_into_at] items is empty; inserting {value:?} at 0",
			file = file!(), line = line!(), module = module_path!(),
		);

		return insert_into_at(items, 0, value);
	}

	let last = items.len() - 1;
	match value {
		// loop (items = [enum(left, right)], order_fn = (&left, &right) -> enum(less, greater), value = left, index = usize, last = usize):
		// - if items[index] is left then end with insert into next index
		// - else if items[index] is right:
		//   - if order_fn(value, items[index]) is less:
		//     - if index > 0 then restart with previous index
		//     - else end with insert into current index
		//   - else if order_fn(value, items[index]) is greater then end with insert into next index
		value @ MergedVecItem::Left(_) if is_end => {
			let mut index = items.len() - 1;

			loop {
				log::trace!(
					"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
					file = file!(), line = line!(), module = module_path!(),
				);

				let existing = &items[index];

				match existing {
					MergedVecItem::Left(_) => {
						log::trace!(
							"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
							file = file!(), line = line!(), module = module_path!(),
						);

						break insert_into_at(items, index + 1, value);
					},

					MergedVecItem::Right(existing) => match order_fn(value.as_left(), existing) {
						Ordering::Less => if index > 0 {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] lowering index from {index} to {}", index - 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							index = index - 1;
							continue;
						} else {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index, value);
						},

						Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
						Ordering::Greater => {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index + 1, value);
						},
					},
				}
			}
		}

		// loop (items = [enum(left, right)], order_fn = (&left, &right) -> enum(less, greater), value = left, index = usize, last = usize):
		// - if items[index] is left then end with insert into current index
		// - else if items[index] is right:
		//   - if order_fn(value, items[index]) is less then end with insert into current index
		//   - else if order_fn(value, items[index]) is greater:
		//     - if index < last then restart with next index
		//     - else insert into next index
		value @ MergedVecItem::Left(_) if is_start => {
			let mut index = 0;

			loop {
				log::trace!(
					"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
					file = file!(), line = line!(), module = module_path!(),
				);

				let existing = &items[index];

				match existing {
					MergedVecItem::Left(_) => {
						log::trace!(
							"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
							file = file!(), line = line!(), module = module_path!(),
						);

						break insert_into_at(items, index, value);
					},

					MergedVecItem::Right(existing) => match order_fn(value.as_left(), existing) {
						Ordering::Less => {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index, value);
						},
						Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
						Ordering::Greater => if index < last {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] raising index from {index} to {}", index + 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							index = index + 1;
							continue;
						} else {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index + 1, value);
						}
					},
				}
			}
		}

		// (items = [enum(left, right)], order_fn = (&left, &right) -> enum(less, greater), value = left, index = usize, last = usize):
		// - if items[index] is left then insert into current index
		// - else if items[index] is right:
		//   - if order_fn(value, items[index]) is less then loop with previous index:
		//     - if items[index] is left then end with insert into next index
		//     - else if items[index] is right:
		//       - if order_fn(value, items[index]) is less:
		//         - if index > 0 then continue with previous index
		//         - else end with insert into current index
		//       - else if order_fn(value, items[index]) is greater then end with insert into next index
		//   - else if order_fn(value, items[index]) is greater then loop with next index:
		//     - if items[index] is left then end with insert into current index
		//     - else if items[index] is right:
		//       - if order_fn(value, items[index]) is less then end with insert into previous index
		//       - else if order_fn(value, items[index]) is greater:
		//         - if index < last then continue with next index
		//         - else end with insert into next index
		value @ MergedVecItem::Left(_) => {
			let index = target;
			let existing = &items[index];

			log::trace!(
				"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
				file = file!(), line = line!(), module = module_path!(),
			);

			// I realise this is basically just replicating what happens above
			// but in reverse. I can't be bothered to figure out how to put
			// this behaviour in its own function right now.
			match existing {
				MergedVecItem::Left(_) => {
					log::trace!(
						"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
						file = file!(), line = line!(), module = module_path!(),
					);

					insert_into_at(items, index + 1, value)
				},

				MergedVecItem::Right(existing) => match order_fn(value.as_left(), existing) {
					Ordering::Less => {
						let mut index = index - 1;

						loop {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
								file = file!(), line = line!(), module = module_path!(),
							);

							let existing = &items[index];

							match existing {
								MergedVecItem::Left(_) => {
									log::trace!(
										"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
										file = file!(), line = line!(), module = module_path!(),
									);

									break insert_into_at(items, index + 1, value);
								},

								MergedVecItem::Right(existing) => match order_fn(value.as_left(), existing) {
									Ordering::Less => if index > 0 {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] lowering index from {index} to {}", index - 1,
											file = file!(), line = line!(), module = module_path!(),
										);

										index = index - 1;
										continue;
									} else {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
											file = file!(), line = line!(), module = module_path!(),
										);

										break insert_into_at(items, index, value);
									},

									Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
									Ordering::Greater => {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
											file = file!(), line = line!(), module = module_path!(),
										);

										break insert_into_at(items, index + 1, value);
									},
								},
							}
						}
					},

					Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
					Ordering::Greater => {
						let mut index = if index < last {
							index + 1
						} else {
							index
						};

						loop {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
								file = file!(), line = line!(), module = module_path!(),
							);

							let existing = &items[index];

							match existing {
								MergedVecItem::Left(_) => {
									log::trace!(
										"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
										file = file!(), line = line!(), module = module_path!(),
									);

									break insert_into_at(items, index, value);
								},

								MergedVecItem::Right(existing) => match order_fn(value.as_left(), existing) {
									Ordering::Less => {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
											file = file!(), line = line!(), module = module_path!(),
										);

										break insert_into_at(items, index, value);
									},

									Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
									Ordering::Greater => if index < last {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] raising index from {index} to {}", index + 1,
											file = file!(), line = line!(), module = module_path!(),
										);

										index = index + 1;
										continue;
									} else {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
											file = file!(), line = line!(), module = module_path!(),
										);

										break insert_into_at(items, index + 1, value);
									}
								},
							}
						}
					},
				}
			}
		}

		// loop (items = [enum(left, right)], order_fn = (&left, &right) -> enum(less, greater), value = right, index = usize, last = usize):
		// - if items[index] is left:
		//   - if order_fn(items[index], value) is less then end with insert into next index
		//   - else if order_fn(items[index], value) is greater:
		//     - if index > 0 then restart with previous index
		//     - else end with insert into current index
		// - else if items[index] is right then end with insert into next index
		value @ MergedVecItem::Right(_) if is_end => {
			let mut index = items.len() - 1;

			loop {
				log::trace!(
					"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
					file = file!(), line = line!(), module = module_path!(),
				);

				let existing = &items[index];

				match existing {
					MergedVecItem::Left(existing) => match order_fn(existing, value.as_right()) {
						Ordering::Less => {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index + 1, value);
						},

						Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),

						Ordering::Greater => if index > 0 {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] lowering index from {index} to {}", index - 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							index = index - 1;
							continue;
						} else {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index, value);
						}
					},

					MergedVecItem::Right(_) => {
						log::trace!(
							"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
							file = file!(), line = line!(), module = module_path!(),
						);

						break insert_into_at(items, index + 1, value);
					},
				}
			}
		}

		// loop (items = [enum(left, right)], order_fn = (&left, &right) -> enum(less, greater), value = right, index = usize, last = usize):
		// - if items[index] is left:
		//   - if order_fn(items[index], value) is less:
		//     - if index < last then continue with next index
		//     - else end with insert into next index
		//   - else if order_fn(items[index], value) is greater then end with insert into current index
		// - else if items[index] is right then end with insert into current index
		value @ MergedVecItem::Right(_) if is_start => {
			let mut index = 0;

			loop {
				log::trace!(
					"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
					file = file!(), line = line!(), module = module_path!(),
				);

				let existing = &items[index];

				match existing {
					MergedVecItem::Left(existing) => match order_fn(existing, value.as_right()) {
						Ordering::Less => if index < last {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] raising index from {index} to {}", index + 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							index = index + 1;
							continue;
						} else {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index + 1, value);
						}

						Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
						Ordering::Greater => {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index, value);
						},
					},

					MergedVecItem::Right(_) => {
						log::trace!(
							"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
							file = file!(), line = line!(), module = module_path!(),
						);

						break insert_into_at(items, index, value);
					},
				}
			}
		}

		// (items = [enum(left, right)], order_fn = (&left, &right) -> enum(less, greater), value = right, index = usize, last = usize):
		// - if index[items] is left:
		//   - if order_fn(index[items], value) is less then loop with next index:
		//     - if index[items] is left:
		//       - if order_fn(index[items], value) is less:
		//         - if index < last then continue with next index
		//         - else end with insert into next index
		//       - else if order_fn(index[items], value) is greater then end with insert into current index
		//     - else if index[items] is right then end with insert into current index
		//   - else if order_fn(index[items], value) is greater then loop with previous index:
		//     - if index[items] is left:
		//       - if order_fn(items[index], value) is less then end with insert into next index
		//       - else if order_fn(index[index], value) is greater:
		//         - if index > 0 then continue with previous index
		//         - else end with insert into current index
		//     - else if index[items] is right end with insert into next index
		// - else if index[items] is right insert into current index
		value @ MergedVecItem::Right(_) => {
			let index = target;
			let existing = &items[index];

			log::trace!(
				"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
				file = file!(), line = line!(), module = module_path!(),
			);

			// I realise this is basically just replicating what happens above
			// but in reverse. I can't be bothered to figure out how to put
			// this behaviour in its own function right now.
			match existing {
				MergedVecItem::Left(existing) => match order_fn(existing, value.as_right()) {
					Ordering::Less => {
						let mut index = if index < last {
							index + 1
						} else {
							index
						};

						loop {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
								file = file!(), line = line!(), module = module_path!(),
							);
				
							let existing = &items[index];

							match existing {
								MergedVecItem::Left(existing) => match order_fn(existing, value.as_right()) {
									Ordering::Less => if index < last {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] raising index from {index} to {}", index + 1,
											file = file!(), line = line!(), module = module_path!(),
										);
			
										index = index + 1;
										continue;
									} else {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
											file = file!(), line = line!(), module = module_path!(),
										);
			
										break insert_into_at(items, index + 1, value);
									},

									Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
									Ordering::Greater => {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
											file = file!(), line = line!(), module = module_path!(),
										);

										break insert_into_at(items, index, value);
									},
								},

								MergedVecItem::Right(_) => {
									log::trace!(
										"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
										file = file!(), line = line!(), module = module_path!(),
									);

									break insert_into_at(items, index, value);
								},
							}
						}
					},

					Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
					Ordering::Greater => {
						let mut index = index - 1;

						loop {
							log::trace!(
								"{file}:{line} [{module}::traverse_insert_into_at] checking location items[{index}]",
								file = file!(), line = line!(), module = module_path!(),
							);
				
							let existing = &items[index];

							match existing {
								MergedVecItem::Left(existing) => match order_fn(existing, value.as_right()) {
									Ordering::Less => {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
											file = file!(), line = line!(), module = module_path!(),
										);
			
										break insert_into_at(items, index + 1, value);
									},

									Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
									Ordering::Greater => if index > 0 {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] lowering index from {index} to {}", index - 1,
											file = file!(), line = line!(), module = module_path!(),
										);
			
										index = index - 1;
										continue;
									} else {
										log::trace!(
											"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
											file = file!(), line = line!(), module = module_path!(),
										);
			
										break insert_into_at(items, index, value);
									},
								},

								MergedVecItem::Right(_) => {
									log::trace!(
										"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {}", index + 1,
										file = file!(), line = line!(), module = module_path!(),
									);

									break insert_into_at(items, index + 1, value);
								},
							}
						}
					},
				}

				MergedVecItem::Right(_) => {
					log::trace!(
						"{file}:{line} [{module}::traverse_insert_into_at] inserting {value:?} at {index}",
						file = file!(), line = line!(), module = module_path!(),
					);

					insert_into_at(items, index, value)
				},
			}
		}
	}
}

fn traverse_push<Left, Right, OrderFn>(
	items: &mut Vec<MergedVecItem<Left, Right>>,
	value: MergedVecItem<Left, Right>,
	order_fn: &mut OrderFn,
) -> VecDiff<MergedVecItem<Left, Right>>
where Left: Debug + Clone,
      Right: Debug + Clone,
	  OrderFn: FnMut(&Left, &Right) -> Ordering,
{
	if items.len() == 0 {
		log::trace!(
			"{file}:{line} [{module}::traverse_push] pushing {value:?} to 0",
			file = file!(), line = line!(), module = module_path!()
		);

		return push(items, value);
	}

	let last = items.len() - 1;
	let matches_end = matches!(
		(&value, items.last().expect("collection should not be empty")),
		(MergedVecItem::Left(_), MergedVecItem::Left(_)) | (MergedVecItem::Right(_), MergedVecItem::Right(_)),
	);

	match value {
		value @ MergedVecItem::Left(_) if matches_end => {
			log::trace!(
				"{file}:{line} [{module}::traverse_push] pushing {value:?} to {}", items.len(),
				file = file!(), line = line!(), module = module_path!()
			);

			push(items, value)
		},

		// loop(items = [enum(left, right)], order_fn = (&left, &right) -> enum(less, greater), value = left, index = usize, last = usize)
		// - if items[index] is left then end with insert at next index
		// - else if items[index] is right:
		//   - if order_fn(value, items[index]) is less:
		//     - if index > 0 then continue with previous index
		//     - else end with insert at current index
		//   - else if order_fn(left, items[index]) is greater:
		//     - if index < last then end with insert at next index
		//     - else end with push
		value @ MergedVecItem::Left(_) => {
			let mut index = last;

			loop {
				log::trace!(
					"{file}:{line} [{module}::traverse_push] checking location items[{index}]",
					file = file!(), line = line!(), module = module_path!()
				);

				let existing = &items[index];

				match existing {
					MergedVecItem::Left(_) => {
						log::trace!(
							"{file}:{line} [{module}::traverse_push] inserting {value:?} at {index}",
							file = file!(), line = line!(), module = module_path!()
						);

						break insert_into_at(items, index + 1, value);
					},

					MergedVecItem::Right(existing) => match order_fn(value.as_left(), existing) {
						Ordering::Less => if index > 0 {
							log::trace!(
								"{file}:{line} [{module}::traverse_push] lowering index from {index} to {}", index - 1,
								file = file!(), line = line!(), module = module_path!()
							);

							index = index - 1;
							continue;
						} else {
							log::trace!(
								"{file}:{line} [{module}::traverse_push] inserting {value:?} at {}", index + 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index, value);
						},

						Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),

						Ordering::Greater => if index < last {
							log::trace!(
								"{file}:{line} [{module}::traverse_push] inserting {value:?} at {}", index + 1,
								file = file!(), line = line!(), module = module_path!()
							);

							break insert_into_at(items, index + 1, value);
						} else {
							log::trace!(
								"{file}:{line} [{module}::traverse_push] pushing {value:?} to {}", items.len(),
								file = file!(), line = line!(), module = module_path!()
							);

							break push(items, value);
						},
					},
				}
			}
		},

		value @ MergedVecItem::Right(_) if matches_end => {
			log::trace!(
				"{file}:{line} [{module}::traverse_push] pushing {value:?} to {}", items.len(),
				file = file!(), line = line!(), module = module_path!(),
			);

			push(items, value)
		},

		// loop(items = [enum(left, right)], order_fn = (&left, &right) -> enum(less, greater), value = right, index = usize)
		// - if items[index] is left:
		//   - if order_fn(items[index], value) is less:
		//     - if index < last then end with insert at next index
		//     - else end with push
		//   - else if order_fn(items[index], value) is greater:
		//     - if index > 0 then continue with previous index
		//     - else end with insert at current index
		// - else if items[index] is right then end with insert at next index
		value @ MergedVecItem::Right(_) => {
			let mut index = last;

			loop {
				log::trace!(
					"{file}:{line} [{module}::traverse_push] checking location items[{index}]",
					file = file!(), line = line!(), module = module_path!(),
				);

				let existing = &items[index];

				match existing {
					MergedVecItem::Left(existing) => match order_fn(existing, value.as_right()) {
						Ordering::Less => if index < last {
							log::trace!(
								"{file}:{line} [{module}::traverse_push] inserting {value:?} at {}", index + 1,
								file = file!(), line = line!(), module = module_path!()
							);

							break insert_into_at(items, index + 1, value);
						} else {
							log::trace!(
								"{file}:{line} [{module}::traverse_push] pushing {value:?} to {}", items.len(),
								file = file!(), line = line!(), module = module_path!()
							);

							break push(items, value);
						},

						Ordering::Equal => unimplemented!("Ordering::Equal has unstable sorting sematics"),
						Ordering::Greater => if index > 0 {
							log::trace!(
								"{file}:{line} [{module}::traverse_push] lowering index from {index} to {}", index - 1,
								file = file!(), line = line!(), module = module_path!(),
							);

							index = index - 1;
							continue;
						} else {
							log::trace!(
								"{file}:{line} [{module}::traverse_push] inserting {value:?} at {index}",
								file = file!(), line = line!(), module = module_path!(),
							);

							break insert_into_at(items, index, value);
						},
					},

					MergedVecItem::Right(_) => {
						log::trace!(
							"{file}:{line} [{module}::traverse_push] inserting {value:?} at {}",
							index + 1, file = file!(), line = line!(), module = module_path!(),
						);

						break insert_into_at(items, index + 1, value);
					},
				}
			}
		},
	}
}
