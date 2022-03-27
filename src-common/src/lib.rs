#![warn(rust_2018_idioms)]
pub mod command;

pub mod constants;
pub mod error;
pub mod helper;
pub mod remote;
pub mod settings;
pub mod state;
pub mod theme;
pub mod user;

/// A shorthand for setting the default value on an enum.
#[macro_export]
macro_rules! default {
	($type: ty => $value: expr) => {
		impl Default for $type {
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
