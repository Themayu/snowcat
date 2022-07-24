use futures::executor::block_on;
use futures::future::poll_fn;
use futures::task::{ArcWake, waker};
use futures::pin_mut;
use futures_signals::signal::Signal;
use futures_signals::signal_map::{MapDiff, SignalMap};
use futures_signals::signal_vec::{SignalVec, VecDiff};
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;
use std::iter::FromIterator;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::thread;
use std::time::Duration;

// -----------------------------------------------------------------------------
// SIGNALS
// -----------------------------------------------------------------------------

pub struct Source<T> {
	changes: VecDeque<Poll<T>>,
}

impl<T> Unpin for Source<T> {}

impl<T> Source<T>
where T: Debug {
	pub fn new(changes: Vec<Poll<T>>) -> Self {
		log::debug!("[{module}::Source<T>::new] initialising source with {changes:?}", module = module_path!());

		Source {
			changes: VecDeque::from(changes)
		}
	}

	fn poll(&mut self, cx: &mut Context) -> Poll<Option<T>> {
		let poll_result = match self.changes.pop_front() {
			Some(Poll::Pending) => {
				cx.waker().wake_by_ref();
				Poll::Pending
			},

			Some(Poll::Ready(change)) =>  Poll::Ready(Some(change)),
			None => Poll::Ready(None),
		};

		log::trace!("[{module}::Source<T>::poll] returning {poll_result:?}", module = module_path!());
		poll_result
	}
}

impl<T> Signal for Source<T>
where T: Debug,
{
	type Item = T;

	fn poll_change(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
		self.poll(cx)
	}
}

impl<T> SignalVec for Source<VecDiff<T>>
where T: Debug,
{
	type Item = T;

	fn poll_vec_change(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<futures_signals::signal_vec::VecDiff<Self::Item>>> {
		self.poll(cx)
	}
}

impl<K, V> SignalMap for Source<MapDiff<K, V>>
where K: Debug,
      V: Debug,
{
	type Key = K;
	type Value = V;

	fn poll_map_change(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<MapDiff<Self::Key, Self::Value>>> {
		self.poll(cx)
	}
}

#[allow(dead_code)]
pub fn delay() {
	thread::sleep(Duration::from_millis(50));
}

// -----------------------------------------------------------------------------
// ASSERTIONS
// -----------------------------------------------------------------------------
#[allow(dead_code)]
pub fn assert_signal_eq<S, T>(signal: S, expected: Vec<Poll<Option<T>>>) -> Option<T>
where S: Signal<Item = T>,
      T: Debug + PartialEq,
{
	let mut output = None;
	let actual = get_all_polls(signal, (), |_, _| ());

	assert_eq!(expected, actual);

	actual.into_iter()
		.filter_map(|value| if let Poll::Ready(Some(value)) = value { Some(value) } else { None })
		.for_each(|value| output = Some(value));

	output
}

#[allow(dead_code)]
pub fn assert_signal_map_eq<S, K, V>(signal: S, expected: Vec<Poll<Option<MapDiff<K, V>>>>) -> HashMap<K, V>
where S: SignalMap<Key = K, Value = V>,
      K: Debug + Hash + PartialEq + Eq,
      V: Debug + PartialEq,
{
	let mut output = HashMap::new();
	let actual = map_poll_map(signal, |_output, change| change);

	assert_eq!(expected, actual);

	actual.into_iter()
		.filter_map(|value| if let Poll::Ready(Some(value)) = value { Some(value) } else { None })
		.for_each(|diff| match diff {
			MapDiff::Replace { entries } => { output = HashMap::from_iter(entries); },
			MapDiff::Insert { key, value } => { output.insert(key, value); },
			MapDiff::Update { key, value } => { output.insert(key, value); },
			MapDiff::Remove { key } => { output.remove(&key); },
			MapDiff::Clear {} => { output.clear(); },
		});

	output
}

#[allow(dead_code)]
pub fn assert_signal_vec_eq<S, T>(signal: S, expected: Vec<Poll<Option<VecDiff<T>>>>) -> Vec<T>
where S: SignalVec<Item = T>,
      T: Debug + PartialEq,
{
	let mut output = Vec::new();
	let actual = map_poll_vec(signal, |_output, change| change);

	assert_eq!(expected, actual);

	actual.into_iter()
		.filter_map(|value| if let Poll::Ready(Some(value)) = value { Some(value) } else { None })
		.for_each(|value| value.apply_to_vec(&mut output));

	output
}

// -----------------------------------------------------------------------------
// READERS
// -----------------------------------------------------------------------------

#[allow(dead_code)]
pub fn get_all_polls<S, Acc, Fn>(signal: S, mut initial: Acc, mut f: Fn) -> Vec<Poll<Option<S::Item>>>
where S: Signal,
      Fn: FnMut(&Acc, &mut Context) -> Acc,
{
	let mut output = vec![];

	pin_mut!(signal);

	block_on(poll_fn(|cx| loop {
		initial = f(&initial, cx);

		let poll_item = Pin::as_mut(&mut signal).poll_change(cx);
		let poll_result = match poll_item {
			Poll::Ready(Some(_)) => {
				output.push(poll_item);
				continue;
			},

			Poll::Ready(None) => {
				output.push(poll_item);
				Poll::Ready(())
			},

			Poll::Pending => {
				output.push(poll_item);
				Poll::Pending
			}
		};

		return poll_result;
	}));

	output
}

#[allow(dead_code)]
pub fn get_signal_polls<S, Fn>(signal: S, f: Fn) -> Vec<Poll<Option<S::Item>>>
where S: Signal,
      Fn: FnOnce(),
{
	pin_mut!(signal);

	get_polls(f, |cx| Pin::as_mut(&mut signal).poll_change(cx))
}

#[allow(dead_code)]
pub fn get_signal_map_polls<S, Fn>(signal: S, f: Fn) -> Vec<Poll<Option<MapDiff<S::Key, S::Value>>>>
where S: SignalMap,
      Fn: FnOnce(),
{
	pin_mut!(signal);

	get_polls(f, |cx| Pin::as_mut(&mut signal).poll_map_change(cx))
}

#[allow(dead_code)]
pub fn get_signal_vec_polls<S, Fn>(signal: S, f: Fn) -> Vec<Poll<Option<VecDiff<S::Item>>>>
where S: SignalVec,
      Fn: FnOnce(),
{
	pin_mut!(signal);

	get_polls(f, |cx| Pin::as_mut(&mut signal).poll_vec_change(cx))
}

// -----------------------------------------------------------------------------
// TRANSFORMERS
// -----------------------------------------------------------------------------

#[allow(dead_code)]
pub fn map_poll_vec<S, U, MapFn>(signal: S, mut map: MapFn) -> Vec<U>
where S: SignalVec,
      MapFn: FnMut(&S, Poll<Option<VecDiff<S::Item>>>) -> U,
{
	let mut changes = vec![];

	pin_mut!(signal);

	block_on(poll_fn(|cx| loop {
		let poll_op = Pin::as_mut(&mut signal).poll_vec_change(cx);

		return match poll_op {
			Poll::Ready(Some(_)) => {
				changes.push(map(&signal, poll_op));
				continue;
			},

			Poll::Ready(None) => {
				changes.push(map(&signal, poll_op));
				Poll::Ready(())
			},

			Poll::Pending => {
				changes.push(map(&signal, poll_op));
				Poll::Pending
			},
		};
	}));


	changes
}

#[allow(dead_code)]
pub fn map_poll_map<S, U, MapFn>(signal: S, mut map: MapFn) -> Vec<U>
where S: SignalMap,
      MapFn: FnMut(&S, Poll<Option<MapDiff<S::Key, S::Value>>>) -> U,
{
	let mut changes = vec![];

	pin_mut!(signal);

	block_on(poll_fn(|cx| loop {
		let poll_op = Pin::as_mut(&mut signal).poll_map_change(cx);

		return match poll_op {
			Poll::Ready(Some(_)) => {
				changes.push(map(&signal, poll_op));
				continue;
			},

			Poll::Ready(None) => {
				changes.push(map(&signal, poll_op));
				Poll::Ready(())
			},

			Poll::Pending => {
				changes.push(map(&signal, poll_op));
				Poll::Pending
			},
		}
	}));

	changes
}

// -----------------------------------------------------------------------------
// SCOPE HELPERS
// -----------------------------------------------------------------------------

#[allow(dead_code)]
pub fn with_noop_context<U, F>(f: F) -> U
where F: FnOnce(&mut Context) -> U {
	struct Noop;

	impl ArcWake for Noop {
		fn wake_by_ref(_: &Arc<Self>) {}
	}

	let waker = waker(Arc::new(Noop));
	let mut context = Context::from_waker(&waker);

	f(&mut context)
}

// -----------------------------------------------------------------------------
// INTERNAL HELPERS
// -----------------------------------------------------------------------------

fn get_polls<T, F, PollFn>(f: F, mut poll: PollFn) -> Vec<Poll<Option<T>>>
where F: FnOnce(),
      PollFn: FnMut(&mut Context) -> Poll<Option<T>>,
{
	let mut f = Some(f);
	let mut output = vec![];

	block_on(poll_fn(|cx| loop {
		let poll_item = poll(cx);

		let poll_result = match poll_item {
			Poll::Ready(Some(_)) => {
				output.push(poll_item);
				continue;
			},
			Poll::Ready(None) => Poll::Ready(()),
			Poll::Pending => Poll::Pending,
		};

		output.push(poll_item);

		if let Some(f) = f.take() {
			f();
		}

		return poll_result;
	}));

	output
}
