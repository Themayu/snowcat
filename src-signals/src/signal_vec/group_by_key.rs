use crate::signal_vec::wrap_poll_result;
use futures_signals::signal_vec::{MutableSignalVec, MutableVec, SignalVec, VecDiff};
use std::collections::VecDeque;
use std::fmt::Debug;
use std::pin::Pin;
use std::task::{Context, Poll};
use pin_project::pin_project;

#[pin_project(project = GroupByKeyProj)]
#[derive(Debug)]
pub struct GroupByKey<Key, KeyFn, Source>
where Key: Eq + Debug + Clone,
	  KeyFn: Fn(&Source::Item) -> Key,
	  Source: SignalVec,
	  Source::Item: Debug + Clone,
{
	chunks: Vec<MutableVec<Source::Item>>,
	key_fn: KeyFn,
	last_key: Option<Key>,
	pending_operations: VecDeque<VecDiff<Source::Item>>,
	pending_returns: VecDeque<VecDiff<SignalVecChunk<Key, Source::Item>>>,

	#[pin]
	signal: Source,
}

impl<Key, KeyFn, Source> GroupByKey<Key, KeyFn, Source>
where Key: Eq + Debug + Clone,
	  KeyFn: Fn(&Source::Item) -> Key,
	  Source: SignalVec,
	  Source::Item: Debug + Clone,
{
	pub(in crate::signal_vec) fn new(signal: Source, key_fn: KeyFn) -> GroupByKey<Key, KeyFn, Source> {
		GroupByKey {
			signal,
			key_fn,

			chunks: vec![],
			last_key: None,
			pending_operations: VecDeque::new(),
			pending_returns: VecDeque::new(),
		}
	}
}

impl<Key, KeyFn, Source> SignalVec for GroupByKey<Key, KeyFn, Source>
where Key: Eq + Debug + Clone,
	  KeyFn: Fn(&Source::Item) -> Key,
	  Source: SignalVec,
	  Source::Item: Debug + Clone,
{
	type Item = SignalVecChunk<Key, Source::Item>;

	fn poll_vec_change(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<VecDiff<Self::Item>>> {
		let GroupByKeyProj {
			chunks,
			key_fn,
			last_key,
			pending_operations,
			pending_returns,
			mut signal,
		} = self.project();

		if let Some(op) = pending_returns.pop_front() {
			log::trace!("running queued op {:?}", &op);
			return wrap_poll_result(op);
		}

		loop {
			let poll_result = match pending_operations.pop_front() {
				Some(op) => Poll::Ready(Some(op)),
				None => signal.as_mut().poll_vec_change(cx),
			};

			let op = match poll_result {
				Poll::Ready(Some(op)) => op,
				Poll::Ready(None) => return Poll::Ready(None),
				Poll::Pending => return Poll::Pending,
			};

			// VecDiff might become non-exhaustive in the future
			#[allow(unreachable_patterns)]
			return Poll::Ready(match op {
				VecDiff::Replace { values } => {
					chunks.drain(..).for_each(|mut chunk| clear_chunk(&mut chunk));
					*chunks = Vec::new();

					let mut signals = Vec::new();

					if values.len() > 0 {
						let mut chunk = Vec::new();
						let mut iterator = values.iter().peekable();

						let first = iterator.peek().unwrap_or_else(|| unreachable!());
						*last_key = Some(key_fn(*first).clone());

						while let Some(value) = iterator.next() {
							log::trace!("processing key {:?}, new value {:?}", last_key, &value);

							match compare_keys(last_key, value, key_fn) {
								KeyComparison::Different { new_key } => {
									log::trace!("key {:?} compared false, splitting", &last_key.as_ref().expect("key"));
									let (mutable, signal) = SignalVecChunk::new(last_key.as_ref().expect("key").clone(), chunk);
								
									chunks.push(mutable);
									signals.push(signal);

									*last_key = Some(new_key);
									chunk = Vec::new();
								},

								KeyComparison::Equal => {},
							}

							chunk.push(value.clone());
						}

						if chunk.len() > 0 {
							let (mutable, signal) = SignalVecChunk::new(last_key.as_ref().expect("key").clone(), chunk);

							chunks.push(mutable);
							signals.push(signal);
						}
					}

					log::trace!("built list {:#?}", &chunks);
					Some(VecDiff::Replace { values: signals })
				},

				VecDiff::InsertAt { index, value } => {
					log::trace!("request to insert value at position {}: {:?}", index, value);

					// we are inserting right at the start of the list; don't bother with obtaining chunk info
					if index == 0 {
						let mut first_chunk = match chunks.get(0) {
							Some(chunk) => chunk.lock_mut(),
							None => {
								// we are inserting the very first value
								let key = key_fn(&value);
								let (chunk, signal) = SignalVecChunk::new(key.clone(), vec![value]);
								
								chunks.insert(0, chunk);
								*last_key = Some(key);

								return wrap_poll_result(VecDiff::InsertAt { index: 0, value: signal });
							},
						};

						let next_value = first_chunk.first().unwrap_or_else(|| unreachable!());
						let chunk_key = key_fn(&next_value);
						let value_key = key_fn(&value);

						if chunk_key == value_key {
							first_chunk.insert_cloned(0, value);
						} else {
							drop(chunk_key);
							drop(first_chunk);

							let (chunk, signal) = SignalVecChunk::new(value_key, vec![value]);
								
							chunks.insert(0, chunk);
							return wrap_poll_result(VecDiff::InsertAt { index: 0, value: signal });
						}

						continue;
					}

					log::trace!("chunk list: {:#?}", chunks);
					let insert_index = get_chunk_from_index(chunks, index);
					let (chunk_index, insert_index) = if let Some((chunk_index, insert_index)) = insert_index {
						(chunk_index, insert_index)
					} else if let Some(chunk) = chunks.last_mut() {
						let mut lock = chunk.lock_mut();

						let chunk_key = key_fn(lock.last().unwrap());
						let value_key = key_fn(&value);

						if value_key == chunk_key {
							lock.insert_cloned(lock.len(), value);
							continue;
						}

						drop(lock);

						let (new_chunk, signal) = SignalVecChunk::new(value_key.clone(), vec![value]);
						let index = chunks.len();

						chunks.push(new_chunk);

						*last_key = Some(value_key);
						return wrap_poll_result(VecDiff::InsertAt { index: index, value: signal });
					} else {
						// already handled, see above
						unreachable!();
					};

					log::trace!("inserting value into chunk {}, index {}", chunk_index, insert_index);
					let mut current_chunk = chunks.get(chunk_index).unwrap_or_else(|| {
						unreachable!("`chunk_index = 0 || chunk_index = chunks.len()` has already been handled")
					}).lock_mut();

					let key = key_fn(&value);
					if insert_index == 0 {
						let mut previous_chunk = chunks.get(chunk_index - 1).unwrap_or_else(|| {
							unreachable!("`chunk_index = 0, insert_index = 0` has already been handled")
						}).lock_mut();

						let previous_last_key = key_fn(previous_chunk.last().unwrap());
						let current_first_key = key_fn(current_chunk.first().unwrap());

						if key == previous_last_key {
							previous_chunk.insert_cloned(previous_chunk.len(), value);
							continue;
						} else if key == current_first_key {
							current_chunk.insert_cloned(0, value);
							continue;
						} else {
							drop(previous_chunk);
							drop(current_chunk);

							let (new_chunk, signal) = SignalVecChunk::new(key, vec![value]);
							chunks.insert(chunk_index, new_chunk);

							Some(VecDiff::InsertAt { index: chunk_index, value: signal })
						}
					} else if insert_index < current_chunk.len() {
						let current_key = key_fn(current_chunk.first().unwrap());
						if key == current_key {
							current_chunk.insert_cloned(insert_index, value);
							continue;
						}

						let new_chunk_index = chunk_index + 1;
						let tail_chunk_index = chunk_index + 2;

						// neither of these will be empty, thankfully
						let (head, tail) = current_chunk.split_at(insert_index);
						let head = head.to_owned();
						let tail = tail.to_owned();

						current_chunk.replace_cloned(head);
						let (new_chunk, new_signal) = SignalVecChunk::new(key, vec![value]);
						let (tail_chunk, tail_signal) = SignalVecChunk::new(current_key, tail);
						
						// give up the no longer needed lock
						drop(current_chunk);

						chunks.insert(new_chunk_index, new_chunk);
						chunks.insert(tail_chunk_index, tail_chunk);

						pending_returns.push_back(VecDiff::InsertAt { index: tail_chunk_index, value: tail_signal });
						return wrap_poll_result(VecDiff::InsertAt { index: new_chunk_index, value: new_signal });
					} else {
						// this might not actually exist
						let mut maybe_next_chunk = chunk_index.checked_add(1).and_then(|index| {
							chunks.get(index).map(|chunk| {
								chunk.lock_mut()
							})
						});

						let current_last_key = key_fn(current_chunk.first().unwrap());
						if key == current_last_key {
							current_chunk.insert_cloned(insert_index, value);
							continue;
						} else if let Some(ref mut next_chunk) = maybe_next_chunk {
							let next_first_key = key_fn(next_chunk.first().unwrap());
							if key == next_first_key {
								next_chunk.insert_cloned(next_chunk.len(), value);
								continue;
							} else {
								drop(current_chunk);

								// the chunk needs to be dropped separately here
								drop(maybe_next_chunk);

								let new_chunk_index = chunk_index + 1;
								let (new_chunk, signal) = SignalVecChunk::new(key.clone(), vec![value]);

								chunks.insert(new_chunk_index, new_chunk);

								Some(VecDiff::InsertAt { index: new_chunk_index, value: signal })
							}
						} else {
							drop(current_chunk);
							drop(maybe_next_chunk);

							let (new_chunk, signal) = SignalVecChunk::new(key.clone(), vec![value]);

							*last_key = Some(key);
							chunks.push(new_chunk);

							Some(VecDiff::InsertAt { index: chunks.len(), value: signal })
						}
					}
				},

				VecDiff::UpdateAt { index, value } => {
					// `index` is guaranteed to be within the bounds of the source
					let (chunk_index, index) = get_chunk_from_index(chunks, index).unwrap_or_else(|| {
						unreachable!();
					});

					// unreachable as `chunk_index` is guaranteed to be within the bounds of self.chunks
					let mut chunk = chunks.get(chunk_index).unwrap_or_else(|| unreachable!()).lock_mut();
					chunk.set_cloned(index, value);
					continue;
				}

				VecDiff::RemoveAt { index } => {
					log::trace!("request to remove value from position {}", index);

					// `index` is guaranteed to be within the bounds of the source
					let (chunk_index, index) = get_chunk_from_index(chunks, index).unwrap_or_else(|| {
						unreachable!();
					});

					// unreachable as `chunk_index` is guaranteed to be within the bounds of self.chunks
					let current_chunk = chunks.get(chunk_index).unwrap_or_else(|| unreachable!());
					log::trace!("current chunk: {:?}", &current_chunk);

					let mut chunk = current_chunk.lock_mut();
					chunk.remove(index);

					if chunk.len() > 0 {
						continue;
					}
					
					// drop chunk borrow so we can alter the chunk list
					drop(chunk);

					log::trace!("removing chunk {}", chunk_index);
					chunks.remove(chunk_index);
					
					let previous_chunk = chunk_index.checked_sub(1).and_then(|index| chunks.get(index)).map(|chunk| chunk.lock_mut());
					let next_chunk = chunks.get(chunk_index).map(|chunk| chunk.lock_mut());
					
					let remove_index = previous_chunk.zip(next_chunk).and_then(|(mut previous_chunk, mut next_chunk)| {
						// temporary borrows to check keys
						let previous_last = previous_chunk.last().unwrap();
						let next_first = next_chunk.first().unwrap();

						if key_fn(&previous_last) != key_fn(next_first) {
							return None;
						}

						// try to move the least amount of elements as possible
						if previous_chunk.len() >= next_chunk.len() {
							log::trace!("moving items to previous chunk: {:?}", AsRef::<[<Source as SignalVec>::Item]>::as_ref(&next_chunk));

							next_chunk.drain(..).for_each(|value| {
								previous_chunk.push_cloned(value);
							});

							Some(chunk_index)
						} else {
							log::trace!("moving items to next chunk: {:?}", AsRef::<[<Source as SignalVec>::Item]>::as_ref(&previous_chunk));

							previous_chunk.drain(..).rev().for_each(|value| {
								next_chunk.insert_cloned(0, value);
							});

							Some(chunk_index - 1)
						}
					});

					if let Some(remove_index) = remove_index {
						chunks.remove(remove_index);
						pending_returns.push_back(VecDiff::RemoveAt { index: remove_index });
					}

					// update the final key
					*last_key = chunks.last().and_then(|chunk| {
						chunk.lock_mut().last().map(|value| {
							key_fn(&value).clone()
						})
					});

					log::trace!("chunk list after change: {:#?}", chunks);
					Some(VecDiff::RemoveAt { index: chunk_index })
				},

				// // This doesn't actually exist yet
				// VecDiff::Batch { changes } => {
				// 	// for every change, queue it up for evaluation. This unfortunately loses the semantics of Batch,
				// 	// but it's just a quick hack until I can properly split this up.
				// 	for change in changes {
				// 		pending_operations.push_back(change);
				// 	}

				// 	continue;
				// },

				// VecDiff::Swap { old_index, new_index } => {
					// if both are inside one chunk: swap inside chunk
					// if both are inside different chunks:
						// remove old_index from chunk 1
						// remove new_index from chunk 2
						// insert old_index into chunk 2
						// insert new_index into chunk 1
				// },

				VecDiff::Move { old_index: from_index, new_index: to_index } => {
					// why would anyone do this...?
					if from_index == to_index {
						continue;
					}

					// old chunk is guaranteed to be within source bounds
					let (old_chunk_index, old_index) = get_chunk_from_index(chunks, from_index).unwrap_or_else(|| {
						unreachable!();
					});

					// future sanity check: is new chunk guaranteed to be within source bounds?
					// of course it is, I don't even know what I was thinking here...
					let (new_chunk_index, new_index) = get_chunk_from_index(chunks, to_index).unwrap_or_else(|| {
						// let chunk_index = chunks.len() - 1;
						// let index = chunks.last().unwrap_or_else(|| unreachable!()).lock_ref().len();
						// (chunk_index, index)

						unreachable!();
					});

					// move a value between locations inside the same chunk
					if old_chunk_index == new_chunk_index {
						let mut chunk = chunks.get(old_chunk_index).unwrap_or_else(|| unreachable!()).lock_mut();
						
						chunk.move_from_to(old_index, new_index);
						continue;
					}

					let old_chunk = chunks.get(old_chunk_index).unwrap_or_else(|| unreachable!()).lock_ref();
					let value = old_chunk.get(old_index).unwrap_or_else(|| unreachable!()).clone();

					// queue up a RemoveAt for the old index, then an InsertAt for the new index
					let to_index = if from_index < to_index {
						to_index - 1
					} else {
						to_index
					};

					pending_operations.push_back(VecDiff::RemoveAt { index: from_index });
					pending_operations.push_back(VecDiff::InsertAt { index: to_index, value });
					
					continue;
				},

				VecDiff::Push { value } => {
					log::trace!("processing key {:?}, new value {:?}", last_key, &value);
					let signal = match compare_keys(last_key, &value, key_fn) {
						KeyComparison::Different { new_key } => {
							log::trace!("key {:?} compared false, splitting", &last_key.as_ref().expect("key"));
							let (mutable, signal) = SignalVecChunk::new(new_key.clone(), vec![value]);
							
							*last_key = Some(new_key);
							chunks.push(mutable);
							
							signal
						},

						KeyComparison::Equal => {
							log::trace!("key {:?} compared true, pushing {:?}", &last_key.as_ref().expect("key"), &value);
							let mut chunk = chunks.last_mut().unwrap_or_else(|| unreachable!()).lock_mut();

							chunk.push_cloned(value.clone());
							continue;
						},
					};

					Some(VecDiff::Push { value: signal })
				},

				VecDiff::Pop {} => {
					// popping a value is only possible if there are values on the source to be popped
					let mut chunk = chunks.last().unwrap_or_else(|| unreachable!()).lock_mut();

					chunk.pop();
					
					// if there are no values left, remove the whole chunk
					if chunk.len() > 0 {
						continue;
					}
					
					drop(chunk);
					chunks.pop();

					Some(VecDiff::Pop {})
				},

				VecDiff::Clear {} => {
					chunks.drain(..).for_each(|mut chunk| clear_chunk(&mut chunk));
					Some(VecDiff::Clear {})
				},

				_ => todo!("diff type {:?}", op)
			})
		}
	}
}

#[derive(Debug)]
pub struct SignalVecChunk<Key, Item>
where Key: Debug + Clone,
	  Item: Debug + Clone,
{
	pub key: Key,
	signal: MutableSignalVec<Item>,
}

impl<Key, Item> SignalVecChunk<Key, Item>
where Key: Debug + Clone,
	  Item: Debug + Clone,
{
	fn new(key: Key, values: Vec<Item>) -> (MutableVec<Item>, SignalVecChunk<Key, Item>) {
		let chunk = MutableVec::new_with_values(values);
		let signal = SignalVecChunk {
			key,
			signal: chunk.signal_vec_cloned(),
		};

		(chunk, signal)
	}
}

impl<Key, Item> Unpin for SignalVecChunk<Key, Item>
where Key: Debug + Clone,
	  Item: Debug + Clone, {}

impl<Key, Item> SignalVec for SignalVecChunk<Key, Item>
where Key: Debug + Clone,
	  Item: Debug + Clone,
{
	type Item = Item;

	#[inline]
	fn poll_vec_change(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<VecDiff<Self::Item>>> {
		Pin::new(&mut self.signal).poll_vec_change(cx)
	}
}

enum KeyComparison<Key> {
	Equal,
	Different { new_key: Key },
}

fn clear_chunk<Item>(chunk: &mut MutableVec<Item>) {
	chunk.lock_mut().clear();
}

fn compare_keys<Key, KeyFn, Item>(last_key: &Option<Key>, item: &Item, key_fn: &KeyFn) -> KeyComparison<Key>
where Key: Eq + Debug + Clone,
	  KeyFn: Fn(&Item) -> Key,
	  Item: Debug,
{
	match (&last_key, key_fn(item).clone()) {
		(Some(last_key), key) if (last_key == &key) => KeyComparison::Equal,
		(Some(_), key) | (None, key) => KeyComparison::Different { new_key: key },
	}
}

fn get_chunk_from_index<Item>(chunks: &[MutableVec<Item>], index: usize) -> Option<(usize, usize)> {
	let mut end = 0;
	
	for (chunk_index, chunk) in chunks.into_iter().enumerate() {
		let start = end;
		end += chunk.lock_ref().len();

		log::trace!("current chunk index: {}", chunk_index);
		if (index >= start) && (index < end) {
			log::trace!("-- returning chunk index {}", chunk_index);
			return Some((chunk_index, index - start));
		}
	}

	None
}

#[cfg(tests)]
mod tests {
	// TODO: FIGURE OUT HOW I AM GOING TO TEST THIS
}
