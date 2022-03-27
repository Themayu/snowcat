use std::task::Poll;

pub(crate) fn wrap_poll_result<T>(value: T) -> Poll<Option<T>> {
	Poll::Ready(Some(value))
}
