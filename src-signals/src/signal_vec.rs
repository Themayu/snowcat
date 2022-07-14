pub mod group_by_key;

use futures_signals::signal_vec::{SignalVec, VecDiff};
use std::fmt::Debug;
use std::task::Poll;

pub trait SnowcatSignalVecExt: SignalVec + Sized {
	fn group_by_key<Key, KeyFn>(self, key_fn: KeyFn) -> group_by_key::GroupByKey<Key, KeyFn, Self>
	where Key: Eq + Debug + Clone,
	      KeyFn: Fn(&Self::Item) -> Key,
		  Self::Item: Debug + Clone,
	{
		group_by_key::GroupByKey::new(self, key_fn)
	}
}

impl<T> SnowcatSignalVecExt for T where T: SignalVec + Sized {}

pub(crate) fn wrap_poll_result<T>(op: VecDiff<T>) -> Poll<Option<VecDiff<T>>> {
	Poll::Ready(Some(op))
}
