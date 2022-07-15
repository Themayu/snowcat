pub mod group_by_key;
pub mod merge;

use futures_signals::signal_vec::{SignalVec, VecDiff};
use std::cmp::Ordering;
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

	fn merge<Other, OrderFn>(self, other: Other, order_fn: OrderFn) -> merge::Merge2<Self, Other, OrderFn>
	where Self: SignalVec,
	      Other: SignalVec,
	      Self::Item: Debug + Clone,
	      Other::Item: Debug + Clone,
	      OrderFn: Fn(&Self::Item, &Other::Item) -> Ordering,
	{
		merge::Merge2::new(self, other, order_fn)
	}
}

impl<T> SnowcatSignalVecExt for T where T: SignalVec + Sized {}

pub(crate) fn wrap_poll_result<T>(op: VecDiff<T>) -> Poll<Option<VecDiff<T>>> {
	Poll::Ready(Some(op))
}
