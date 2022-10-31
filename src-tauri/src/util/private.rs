use std::fmt;
use std::ops;

pub struct Private<T>(T);

impl<T> fmt::Debug for Private<T>
where
	T: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct(std::any::type_name::<T>())
			.finish_non_exhaustive()
	}
}

impl<T> fmt::Display for Private<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", std::any::type_name::<T>())
	}
}

impl<T> ops::Deref for Private<T> {
	type Target = T;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> ops::DerefMut for Private<T> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}
