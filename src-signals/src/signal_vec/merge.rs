use crate::signal_vec::wrap_poll_result;
use futures_signals::signal_vec::{SignalVec, VecDiff};
use pin_project::pin_project;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::mem;
use std::pin::Pin;
use std::task::{Poll, Context};

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

		loop {
			let poll_result = {
				let next_left = pending_ops_left.pop_front()
					.map(|item| wrap_poll_result(item.clone()))
					.unwrap_or_else(|| left.as_mut().poll_vec_change(cx));

				let next_right = pending_ops_right.pop_front()
					.map(|item| wrap_poll_result(item.clone()))
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
						log::trace!("[{module}::<Merge2 as SignalVec>::poll_vec_change] priority is {priority:?}", module = module_path!());

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
				Poll::Ready(Some(MergedVecItem::Left(op))) => MergedVecDiff::from_left(op),
				Poll::Ready(Some(MergedVecItem::Right(op))) => MergedVecDiff::from_right(op),
				Poll::Ready(None) => return Poll::Ready(None),
				Poll::Pending => return Poll::Pending,
			};

			// VecDiff might become non-exhaustive in the future
			// TODO: Find a better way of doing this, because keeping these two
			//       synchronised is going to be a PITA.
			#[allow(unreachable_patterns)]
			return Poll::Ready(match op {
				MergedVecDiff::Replace(replace) => match replace {
					MergedVecDiffReplace::Left { values } => {
						let mut items_right: VecDeque<Right::Item> = items.drain(..)
							.filter_map(|item| {
								matches!(item, MergedVecItem::Right(_)).then(|| item.unwrap_right())
							})
							.collect();

							// re-insert all items one at a time into the items
							// collection.
							values.into_iter().for_each(|item| loop {
								let item_right = items_right.front();
								let ordering = item_right.map(|value| order_fn(&item, value)).unwrap_or(Ordering::Greater);

								match ordering {
									Ordering::Less => {
										// we're working on this exact item
										items.push(MergedVecItem::Right(items_right.pop_front().unwrap()));
									}

									Ordering::Equal => {
										unimplemented!("Ordering::Equal has unpredictable results")
									}

									Ordering::Greater => {
										break items.push(MergedVecItem::Left(item));
									},
								};
							});

							Some(VecDiff::Replace { values: items.clone() })
					},

					MergedVecDiffReplace::Right { values } => {
						let mut items_left: VecDeque<Left::Item> = items.drain(..)
						.filter_map(|item| {
							matches!(item, MergedVecItem::Left(_)).then(|| item.unwrap_left())
						})
						.collect();

						// re-insert all items one at a time into the items
						// collection.
						values.into_iter().for_each(|item| loop {
							let item_left = items_left.front();
							let ordering = item_left.map(|value| order_fn(value, &item)).unwrap_or(Ordering::Less);

							match ordering {
								Ordering::Greater => {
									// we're working on this exact item
									items.push(MergedVecItem::Left(items_left.pop_front().unwrap()));
								}

								Ordering::Equal => {
									unimplemented!("Ordering::Equal has unpredictable results")
								}

								Ordering::Less => {
									break items.push(MergedVecItem::Right(item));
								},
							};
						});

						Some(VecDiff::Replace { values: items.clone() })
					},
				},

				MergedVecDiff::InsertAt(insert_at) => {
					let (index, value) = match insert_at {
						MergedVecInsertAt::Left { index, value } => (
							get_index(items, MergedVecItem::Left(index)),
							MergedVecItem::Left(value),
						),

						MergedVecInsertAt::Right { index, value } => (
							get_index(items, MergedVecItem::Right(index)),
							MergedVecItem::Right(value),
						),
					};

					let index = match index {
						Some(index) => index,
						None => items.len(),
					};

					let is_match = mem::discriminant(&items[index]) == mem::discriminant(&value);
					let index = if is_match || index == items.len() {
						index
					} else {
						let existing = &items[index];

						match &value {
							MergedVecItem::Left(value) => match order_fn(value, existing.as_right()) {
								Ordering::Less => index,
								Ordering::Equal => unimplemented!("Ordering::Equal has unpredictable results"),
								Ordering::Greater => index + 1,
							},

							MergedVecItem::Right(value) => match order_fn(existing.as_left(), value) {
								Ordering::Less => index + 1,
								Ordering::Equal => unimplemented!("Ordering::Equal has unpredictable results"),
								Ordering::Greater => index,
							},
						}
					};

					items.insert(index, value.clone());
					Some(VecDiff::InsertAt { index, value })
				},

				MergedVecDiff::UpdateAt(update_at) => {
					let (index, value) = match update_at {
						MergedVecUpdateAt::Left { index, value } => (
							get_index(items, MergedVecItem::Left(index)),
							MergedVecItem::Left(value),
						),

						MergedVecUpdateAt::Right { index, value } => (
							get_index(items, MergedVecItem::Right(index)),
							MergedVecItem::Right(value),
						),
					};

					match index {
						Some(index) => {
							items[index] = value.clone();
							Some(VecDiff::UpdateAt { index, value })
						},

						None => unreachable!("update requires pre-existing index")
					}
				},

				MergedVecDiff::RemoveAt(remove_at) => {
					let index = match remove_at {
						MergedVecRemoveAt::Left { index, .. } =>
							get_index(items, MergedVecItem::Left(index)),

						MergedVecRemoveAt::Right { index, .. } =>
							get_index(items, MergedVecItem::Right(index)),
					};

					match index {
						Some(index) => {
							items.remove(index);
							Some(VecDiff::RemoveAt { index })
						}

						None => unreachable!("remove requires pre-existing index")
					}
				},

				MergedVecDiff::MoveItem(move_item) => {
					let (old_index, new_index) = match move_item {
						MergedVecMoveItem::Left { old_index, new_index, .. } => (
							get_index(items, MergedVecItem::Left(old_index)),
							get_index(items, MergedVecItem::Left(new_index)),
						),

						MergedVecMoveItem::Right { old_index, new_index, .. } => (
							get_index(items, MergedVecItem::Right(old_index)),
							get_index(items, MergedVecItem::Right(new_index)),
						),
					};

					let old_index = old_index.unwrap_or_else(|| unreachable!("move requires pre-existing old index"));
					let new_index = new_index.unwrap_or_else(|| unreachable!("move requires pre-existing new index"));

					let value = items.remove(old_index);
					items.insert(new_index, value);

					Some(VecDiff::Move { old_index, new_index })
				},

				MergedVecDiff::PushItem(push_item) => {
					let existing = items.last();
					let value = match push_item {
						MergedVecPushItem::Left { value } => MergedVecItem::Left(value),
						MergedVecPushItem::Right { value } => MergedVecItem::Right(value),
					};

					match (existing, value) {
						// no items exist
						(None, value) => Some(push(items, value)),

						// final item comes from same signalvec as current
						(Some(MergedVecItem::Left(_)), value @ MergedVecItem::Left(_))
						| (Some(MergedVecItem::Right(_)), value @ MergedVecItem::Right(_)) => Some(push(items, value)),

						// existing item comes from left signalvec, current item
						// from right
						(Some(MergedVecItem::Left(existing)), value @ MergedVecItem::Right(_)) => {
							let mut index = items.len() - 1;

							Some(match order_fn(existing, value.as_right()) {
								Ordering::Less => loop {
									index = match index.checked_sub(1) {
										Some(index) => index,
										None => 0
									};

									let existing = &items[index];
									match existing {
										MergedVecItem::Left(existing) => match order_fn(existing, value.as_right()) {
											Ordering::Less => if index > 0 {
												continue;
											} else {
												break insert_at(items, 0, value);
											},

											Ordering::Equal => unimplemented!("Ordering::Equal has unpredictable results"),
											Ordering::Greater => break insert_at(items, index, value),
										}

										MergedVecItem::Right(_) => break insert_at(items, index + 1, value),
									}
								}

								Ordering::Equal => unimplemented!("Ordering::Equal has unpredictable results"),
								Ordering::Greater => push(items, value),
							})
						},

						// existing item comes from right signal vec, current
						// item from left
						(Some(MergedVecItem::Right(existing)), value @ MergedVecItem::Left(_)) => {
							let mut index = items.len() - 1;

							Some(match order_fn(value.as_left(), existing) {
								Ordering::Less => loop {
									index = match index.checked_sub(1) {
										Some(index) => index,
										None => 0
									};

									let existing = &items[index];
									match existing {
										MergedVecItem::Right(existing) => match order_fn(value.as_left(), existing) {
											Ordering::Less => if index > 0 {
												continue;
											} else {
												break insert_at(items, 0, value);
											},

											Ordering::Equal => unimplemented!("Ordering::Equal has unpredictable results"),
											Ordering::Greater => break insert_at(items, index, value),
										}

										MergedVecItem::Left(_) => break insert_at(items, index + 1, value),
									}
								}

								Ordering::Equal => unimplemented!("Ordering::Equal has unpredictable results"),
								Ordering::Greater => push(items, value),
							})
						}
					}
				},

				MergedVecDiff::PopItem(pop_item) => {
					let index = match pop_item {
						MergedVecPopItem::Left { .. } => get_last_index(items, MergedVecItem::Left(())),
						MergedVecPopItem::Right { .. } => get_last_index(items, MergedVecItem::Right(())),
					};

					let op = match index {
						Some(index) if index > items.len() - 1 => VecDiff::RemoveAt { index },
						Some(_) => VecDiff::Pop {},
						None => unreachable!("item is guaranteed to exist"),
					};

					// we already guaranteed above that index != None
					items.remove(index.unwrap());
					Some(op)
				},

				MergedVecDiff::Clear(clear) => {
					items.retain(|item| match item {
						MergedVecItem::Left(_) if matches!(clear, MergedVecClear::Left { .. }) => true,
						MergedVecItem::Right(_) if matches!(clear, MergedVecClear::Right { .. }) => true,
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
}

#[derive(Debug, Clone, PartialEq)]
pub enum MergedVecItem<Left, Right>
where Left: Debug,
      Right: Debug,
{
	Left(Left),
	Right(Right)
}

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

	/// Attempt to unwrap a `MergedVecItem::Left` value, panicking if `self`
	/// is `MergedVecItem::Right`.
	fn unwrap_left(self) -> Left {
		match self {
			MergedVecItem::Left(value) => value,
			MergedVecItem::Right(_) => panic!("called `MergedVecItem::unwrap_left()` on a `Right` value"),
		}
	}

	/// Attempt to unwrap a `MergedVecItem::Right` value, panicking if `self`
	/// is `MergedVecItem::Left`
	fn unwrap_right(self) -> Right {
		match self {
			MergedVecItem::Left(_) => panic!("called `MergedVecItem::unwrap_right()` on a `Left` value"),
			MergedVecItem::Right(value) => value,
		}
	}
}

/// A version of [`VecDiff`](futures_signals::signal_vec::VecDiff) that can be
/// used for processing a merge.
#[derive(Debug, Clone)]
enum MergedVecDiff<Left, Right>
where Left: Debug,
      Right: Debug,
{
	Replace(MergedVecDiffReplace<Left, Right>),
	InsertAt(MergedVecInsertAt<Left, Right>),
	UpdateAt(MergedVecUpdateAt<Left, Right>),
	RemoveAt(MergedVecRemoveAt<Left, Right>),
	MoveItem(MergedVecMoveItem<Left, Right>),
	PushItem(MergedVecPushItem<Left, Right>),
	PopItem(MergedVecPopItem<Left, Right>),
	Clear(MergedVecClear<Left, Right>),
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
				Self::Replace(MergedVecDiffReplace::Left { values }),

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
				Self::Replace(MergedVecDiffReplace::Right { values }),

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
enum MergedVecDiffReplace<Left, Right> {
	Left {
		values: Vec<Left>,
	},

	Right {
		values: Vec<Right>,
	},
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

#[derive(Debug, Clone)]
enum MergedVecPushItem<Left, Right> {
	Left {
		value: Left,
	},

	Right {
		value: Right,
	},
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

#[derive(Debug, Clone)]
enum MergedVecClear<Left, Right> {
	Left {
		_marker: PhantomData<Left>,
	},

	Right {
		_marker: PhantomData<Right>,
	},
}

/// Prioritise one operation over the other, based on the order they appear in
/// [`VecDiff<T>`](futures_signals::signal_vec::VecDiff).
fn get_priority<'op, Base, Other>(base: &'op VecDiff<Base>, other: &'op VecDiff<Other>)
-> MergedVecItem<&'op VecDiff<Base>, &'op VecDiff<Other>>
where Base: Debug + Clone,
      Other: Debug + Clone,
{
	// VecDiff might become non-exhaustive in the future
	#[allow(unreachable_patterns)]
	match (base, other) {
		// prioritise base for all items
		(VecDiff::Replace { values: _ }, VecDiff::Replace { values: _ })
		| (VecDiff::Replace { values: _ }, VecDiff::InsertAt { index: _, value: _ })
		| (VecDiff::Replace { values: _ }, VecDiff::UpdateAt { index: _, value: _ })
		| (VecDiff::Replace { values: _ }, VecDiff::RemoveAt { index: _ })
		| (VecDiff::Replace { values: _ }, VecDiff::Move { old_index: _, new_index: _ })
		| (VecDiff::Replace { values: _ }, VecDiff::Push { value: _ })
		| (VecDiff::Replace { values: _ }, VecDiff::Pop {  })
		| (VecDiff::Replace { values: _ }, VecDiff::Clear {  }) => MergedVecItem::Left(&base),

		// prioritise base for insertion, other for rest
		(VecDiff::InsertAt { index: _, value: _ }, VecDiff::Replace { values: _ })
		| (VecDiff::InsertAt { index: _, value: _ }, VecDiff::InsertAt { index: _, value: _ }) => MergedVecItem::Left(&base),
		(VecDiff::InsertAt { index: _, value: _ }, VecDiff::UpdateAt { index: _, value: _ })
		| (VecDiff::InsertAt { index: _, value: _ }, VecDiff::RemoveAt { index: _ })
		| (VecDiff::InsertAt { index: _, value: _ }, VecDiff::Move { old_index: _, new_index: _ })
		| (VecDiff::InsertAt { index: _, value: _ }, VecDiff::Push { value: _ })
		| (VecDiff::InsertAt { index: _, value: _ }, VecDiff::Pop {  })
		| (VecDiff::InsertAt { index: _, value: _ }, VecDiff::Clear {  }) => MergedVecItem::Right(&other),

		// prioritise base for update, other for rest
		(VecDiff::UpdateAt { index: _, value: _ }, VecDiff::Replace { values: _ })
		| (VecDiff::UpdateAt { index: _, value: _ }, VecDiff::InsertAt { index: _, value: _ })
		| (VecDiff::UpdateAt { index: _, value: _ }, VecDiff::UpdateAt { index: _, value: _ }) => MergedVecItem::Left(&base),
		(VecDiff::UpdateAt { index: _, value: _ }, VecDiff::RemoveAt { index: _ })
		| (VecDiff::UpdateAt { index: _, value: _ }, VecDiff::Move { old_index: _, new_index: _ })
		| (VecDiff::UpdateAt { index: _, value: _ }, VecDiff::Push { value: _ })
		| (VecDiff::UpdateAt { index: _, value: _ }, VecDiff::Pop {  })
		| (VecDiff::UpdateAt { index: _, value: _ }, VecDiff::Clear {  }) => MergedVecItem::Right(&other),

		// prioritise base for removal, other for rest
		(VecDiff::RemoveAt { index: _ }, VecDiff::Replace { values: _ })
		| (VecDiff::RemoveAt { index: _ }, VecDiff::InsertAt { index: _, value: _ })
		| (VecDiff::RemoveAt { index: _ }, VecDiff::UpdateAt { index: _, value: _ })
		| (VecDiff::RemoveAt { index: _ }, VecDiff::RemoveAt { index: _ }) => MergedVecItem::Left(&base),
		(VecDiff::RemoveAt { index: _ }, VecDiff::Move { old_index: _, new_index: _ })
		| (VecDiff::RemoveAt { index: _ }, VecDiff::Push { value: _ })
		| (VecDiff::RemoveAt { index: _ }, VecDiff::Pop {  })
		| (VecDiff::RemoveAt { index: _ }, VecDiff::Clear {  }) => MergedVecItem::Right(&other),

		// prioritise base for movement, other for rest
		(VecDiff::Move { old_index: _, new_index: _ }, VecDiff::Replace { values: _ })
		| (VecDiff::Move { old_index: _, new_index: _ }, VecDiff::InsertAt { index: _, value: _ })
		| (VecDiff::Move { old_index: _, new_index: _ }, VecDiff::UpdateAt { index: _, value: _ })
		| (VecDiff::Move { old_index: _, new_index: _ }, VecDiff::RemoveAt { index: _ })
		| (VecDiff::Move { old_index: _, new_index: _ }, VecDiff::Move { old_index: _, new_index: _ }) => MergedVecItem::Left(&base),
		(VecDiff::Move { old_index: _, new_index: _ }, VecDiff::Push { value: _ })
		| (VecDiff::Move { old_index: _, new_index: _ }, VecDiff::Pop {  })
		| (VecDiff::Move { old_index: _, new_index: _ }, VecDiff::Clear {  }) => MergedVecItem::Right(&other),

		// prioritise base for push, other for rest
		(VecDiff::Push { value: _ }, VecDiff::Replace { values: _ })
		| (VecDiff::Push { value: _ }, VecDiff::InsertAt { index: _, value: _ })
		| (VecDiff::Push { value: _ }, VecDiff::UpdateAt { index: _, value: _ })
		| (VecDiff::Push { value: _ }, VecDiff::RemoveAt { index: _ })
		| (VecDiff::Push { value: _ }, VecDiff::Move { old_index: _, new_index: _ })
		| (VecDiff::Push { value: _ }, VecDiff::Push { value: _ }) => MergedVecItem::Left(&base),
		(VecDiff::Push { value: _ }, VecDiff::Pop {  })
		| (VecDiff::Push { value: _ }, VecDiff::Clear {  }) => MergedVecItem::Right(&other),

		// prioritise base for pop, other for rest
		(VecDiff::Pop {  }, VecDiff::Replace { values: _ })
		| (VecDiff::Pop {  }, VecDiff::InsertAt { index: _, value: _ })
		| (VecDiff::Pop {  }, VecDiff::UpdateAt { index: _, value: _ })
		| (VecDiff::Pop {  }, VecDiff::RemoveAt { index: _ })
		| (VecDiff::Pop {  }, VecDiff::Move { old_index: _, new_index: _ })
		| (VecDiff::Pop {  }, VecDiff::Push { value: _ })
		| (VecDiff::Pop {  }, VecDiff::Pop {  }) => MergedVecItem::Left(&base),
		(VecDiff::Pop {  }, VecDiff::Clear {  }) => MergedVecItem::Right(&other),

		// prioritise other for all items
		(VecDiff::Clear {  }, VecDiff::Replace { values: _ })
		| (VecDiff::Clear {  }, VecDiff::InsertAt { index: _, value: _ })
		| (VecDiff::Clear {  }, VecDiff::UpdateAt { index: _, value: _ })
		| (VecDiff::Clear {  }, VecDiff::RemoveAt { index: _ })
		| (VecDiff::Clear {  }, VecDiff::Move { old_index: _, new_index: _ })
		| (VecDiff::Clear {  }, VecDiff::Push { value: _ })
		| (VecDiff::Clear {  }, VecDiff::Pop {  })
		| (VecDiff::Clear {  }, VecDiff::Clear {  }) => MergedVecItem::Right(&other),

		(_, _) => todo!("diff type {:?}", (base, other))
	}
}

fn get_index<Left, Right>(items: &[MergedVecItem<Left, Right>], index: MergedVecItem<usize, usize>) -> Option<usize>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	match index {
		MergedVecItem::Left(mut index) => {
			for (idx, item) in items.into_iter().enumerate() {
				if matches!(item, MergedVecItem::Left(_)) {
					index -= 1;
				}

				if index == 0 {
					return Some(idx);
				}
			}
		},

		MergedVecItem::Right(mut index) => {
			for (idx, item) in items.into_iter().enumerate() {
				if matches!(item, MergedVecItem::Right(_)) {
					index -= 1;
				}

				if index == 0 {
					return Some(idx);
				}
			}
		},
	};

	None
}

fn get_last_index<Left, Right>(items: &[MergedVecItem<Left, Right>], side: MergedVecItem<(), ()>) -> Option<usize>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	match side {
		MergedVecItem::Left(()) => items.into_iter()
			.enumerate()
			.rev()
			.find_map(|(idx, item)| {
				matches!(item, MergedVecItem::Left(_)).then(|| idx)
			}),

		MergedVecItem::Right(()) => items.into_iter()
			.enumerate()
			.rev()
			.find_map(|(idx, item)| {
				matches!(item, MergedVecItem::Right(_)).then(|| idx)
			}),
	}
}

fn insert_at<Left, Right>(items: &mut Vec<MergedVecItem<Left, Right>>, index: usize, value: MergedVecItem<Left, Right>)
-> VecDiff<MergedVecItem<Left, Right>>
where Left: Debug + Clone,
      Right: Debug + Clone,
{
	items.insert(index, value.clone());
	VecDiff::InsertAt { index, value }
}

fn push<Left, Right>(items: &mut Vec<MergedVecItem<Left, Right>>, value: MergedVecItem<Left, Right>)
-> VecDiff<MergedVecItem<Left, Right>>
where Left: Debug + Clone,
	  Right: Debug + Clone,
{
	items.push(value.clone());
	VecDiff::Push { value }
}
