#![cfg_attr(feat_future_join, feature(future_join, future_poll_fn))]

pub mod state;

/// A shorthand for setting the default value on a struct or enum.
#[macro_export]
macro_rules! default {
	($type: ty => $value: expr) => {
		impl ::std::default::Default for $type {
			fn default() -> Self {
				$value
			}
		}
	};
}

/// A helper macro for different forms of object deserialization.
#[macro_export]
macro_rules! deserialize {
	($value: ident => $remote: ty $(=> $local: ty)*, raw_errors) => {
		{
			let out = ::serde_json::from_str::<$remote>($value);
			$(
				let out = { out.map(|out| <$local>::from(&out)) };
			)*
			out
		}
	};

	($value: ident => $remote: ty $(=> $local: ty)*) => {
		{
			$crate::deserialize!($value => $remote $(=> $local)*, raw_errors)
				.map_err(|_| $crate::error::ResponseError::DeserializationFailed.into())
		}
	};
}
