use std::borrow::{Cow, Borrow};
use std::convert::TryInto;
use std::ops::ControlFlow;
use std::{fmt, slice, str};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(feat_array_chunks)]
use std::slice::ArrayChunks;

const TABLE: [(u8, u8); 16] = [
	(b'0', 0),
	(b'1', 1),
	(b'2', 2),
	(b'3', 3),
	(b'4', 4),
	(b'5', 5),
	(b'6', 6),
	(b'7', 7),
	(b'8', 8),
	(b'9', 9),
	(b'a', 10),
	(b'b', 11),
	(b'c', 12),
	(b'd', 13),
	(b'e', 14),
	(b'f', 15),
];

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct HexByte(u8);

impl HexByte {
	pub fn to_byte(self) -> u8 {
		return self.0;
	}

	pub fn from_byte(byte: u8) -> HexByte {
		HexByte(byte)
	}

	fn to_high(self) -> char {
		let byte = (self.0 & 0xF0) >> 4;

		// SAFETY: We are only working with four of the eight bits in our u8.
		unsafe {
			Self::get_char_unchecked(byte)
		}
	}

	fn to_low(self) -> char {
		let byte = self.0 & 0x0F;

		// SAFETY: We are only working with four of the eight bits in our u8.
		unsafe {
			Self::get_char_unchecked(byte)
		}
	}

	unsafe fn get_char_unchecked(byte: u8) -> char {
		let byte: usize = byte.into();
		let value = TABLE.get_unchecked(byte).0;

		char::from_u32_unchecked(value as u32)
	}
}

impl Default for HexByte {
	fn default() -> Self {
		HexByte(0)
	}
}

impl fmt::Debug for HexByte {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", self.0)
	}
}

impl fmt::Display for HexByte {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let high = self.to_high();
		let low = self.to_low();

		write!(f, "{high}{low}")
	}
}

impl<'de> Deserialize<'de> for HexByte {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		use serde::de::{Error, Unexpected, Visitor};

		struct ByteVisitor;
		impl<'de> Visitor<'de> for ByteVisitor {
			type Value = HexByte;

			fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "a value between 0-255, or a hexadecimal pair")
			}

			fn visit_u8<E>(self, byte: u8) -> Result<Self::Value, E>
			where
				E: Error,
			{
				Ok(HexByte(byte))
			}

			fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
			where
				E: Error,
			{
				let mut string = Cow::Borrowed(string);

				if string.len() < 1 || string.len() > 2 {
					return Err(Error::invalid_value(
						Unexpected::Str(string.borrow()),
						&"one or two hexadecimal digits"
					));
				}

				if !string.is_ascii() {
					return Err(Error::invalid_value(Unexpected::Str(string.borrow()), &"a valid ascii string"));
				}

				fn find_byte<E>(codepoint: u8) -> Result<u8, E>
				where
					E: Error,
				{
					TABLE.iter()
						.find_map(|&(c, byte)| (c == codepoint.to_ascii_lowercase()).then_some(byte))
						.ok_or_else(|| Error::invalid_value(
							Unexpected::Char(unsafe { char::from_u32_unchecked(codepoint.into()) }),
							&"a hexadecimal digit",
						))
				}

				if string.len() == 1 {
					string.to_mut().insert(0, '0');
				}

				let [high, low] = match string.as_bytes() {
					[high, low] => [*high, *low],
					_ => unreachable!("string is always ascii of length 2"),
				};

				let high = find_byte(high)? << 4;
				let low = find_byte(low)?;

				let byte = high | low;

				Ok(HexByte(byte))
			}
		}

		deserializer.deserialize_any(ByteVisitor)
	}
}

impl Serialize for HexByte {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_u8(self.to_byte())
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hex<const BYTES: usize>([HexByte; BYTES]);

impl<const N: usize> Hex<N> {
	pub fn as_bytes(&self) -> &[u8] {
		let ptr_start = self.0.as_ptr() as *const u8;

		// SAFETY: HexByte is repr(transparent), so it has the exact same
		// memory representation as u8. In addition, the pointer has provenance
		// for the entire array, as it comes from the array.
		unsafe {
			slice::from_raw_parts(ptr_start, N)
		}
	}

	pub fn as_bytes_mut(&mut self) -> &mut [u8] {
		let ptr_start = self.0.as_mut_ptr() as *mut u8;

		// SAFETY: HexByte is repr(transparent), so it has the exact same
		// memory representation as u8. In addition, the pointer has provenance
		// for the entire array, as it comes from the array. Finally, `Hex` is
		// borrowed as mutable, so we cannot obtain multiple references through
		// this function.
		unsafe {
			slice::from_raw_parts_mut(ptr_start, N)
		}
	}
}

impl<const N: usize> Default for Hex<N> {
	fn default() -> Self {
		Hex([HexByte::default(); N])
	}
}

impl<const N: usize> fmt::Display for Hex<N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if f.alternate() {
			write!(f, "0x")?;
		}

		if let Some(target_width) = f.width() {
			let output_width = self.0.len() * 2;

			if let Some(padding) = target_width.checked_sub(output_width) {
				// round to next multiple of 2
				let padding = match padding % 2 {
					0 => padding,
					_ => padding + 1,
				};

				write!(f, "{}", str::repeat("0", padding))?;
			}
		}

		for c in self.0 {
			write!(f, "{c}")?;
		}

		Ok(())
	}
}

impl<'de, const BYTES: usize> Deserialize<'de> for Hex<BYTES> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		use serde::de::{Error, SeqAccess, Unexpected, Visitor};

		struct HexVisitor<const BYTES: usize>;
		impl<'de, const BYTES: usize> Visitor<'de> for HexVisitor<BYTES> {
			type Value = Hex<BYTES>;

			fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
				write!(f, "an array of bytes or a hexadecimal string")
			}

			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: SeqAccess<'de>,
			{
				let err_text = format!("less than or equal to {BYTES} elements");
				let err_text = err_text.as_str();

				if let Some(size) = seq.size_hint() {
					if size > BYTES {
						return Err(Error::invalid_length(size, &err_text));
					}
				}

				let mut bytes = [0; BYTES];
				let res: ControlFlow<Result<(), A::Error>> = bytes.iter_mut().rev().try_for_each(|place| {
					match seq.next_element::<u8>() {
						Ok(value) => match value {
							Some(value) => {
								*place = value;
								ControlFlow::Continue(())
							},

							None => ControlFlow::Break(Ok(())),
						},

						Err(err) => ControlFlow::Break(Err(err)),
					}
				});

				if let Some(_) = seq.next_element::<u8>()? {
					return Err(Error::invalid_length(bytes.len() + 1, &err_text))
				}

				match res {
					ControlFlow::Continue(_)
					| ControlFlow::Break(Ok(_)) => Ok(bytes.into()),

					ControlFlow::Break(Err(err)) => Err(err),
				}
			}

			fn visit_str<E>(self, string: &str) -> Result<Self::Value, E>
			where
				E: Error,
			{
				let err_text = format!("less than or equal to {} hexadecimal characters", BYTES * 2);
				let err_text = err_text.as_str();

				string.parse().map_err(|err| match err {
					HexFromStrError::InputTooLong { actual, .. } => Error::invalid_length(actual, &err_text),
					HexFromStrError::NotAscii => Error::invalid_type(Unexpected::Str(string), &"a valid ascii string"),

					HexFromStrError::UnexpectedCharacter { codepoint } => Error::invalid_value(
						Unexpected::Char(unsafe { char::from_u32_unchecked(codepoint.into()) }),
						&"a hexadecimal digit",
					),
				})
			}
		}

		deserializer.deserialize_any(HexVisitor)
	}
}

impl<const BYTES: usize> Serialize for Hex<BYTES> {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<const BYTES: usize> From<Hex<BYTES>> for [u8; BYTES] {
	fn from(hex: Hex<BYTES>) -> Self {
		hex.0.map(HexByte::to_byte)
	}
}

impl<const BYTES: usize> From<[u8; BYTES]> for Hex<BYTES> {
	fn from(bytes: [u8; BYTES]) -> Self {
		Hex(bytes.map(HexByte::from_byte))
	}
}

impl<const BYTES: usize> str::FromStr for Hex<BYTES> {
	type Err = HexFromStrError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let mut s = Cow::Borrowed(s);

		if !s.is_ascii() {
			return Err(HexFromStrError::NotAscii);
		}

		if s.len() > BYTES * 2 {
			return Err(HexFromStrError::InputTooLong {
				actual: s.len(),
				max: BYTES,
			});
		}

		if s.len() % 2 != 0 {
			s.to_mut().insert(0, '0');
		}

		let s = s.borrow();

		let bytes = as_chunks(s)
			.enumerate()
			.try_rfold([0u8; BYTES], |mut acc, (index, &chunk)| {
				let [high, low] = chunk;

				fn find_byte(codepoint: u8) -> Result<u8, HexFromStrError> {
					TABLE.iter()
						.find_map(|&(c, byte)| (c == codepoint.to_ascii_lowercase()).then_some(byte))
						.ok_or_else(|| HexFromStrError::UnexpectedCharacter { codepoint })
				}
				
				let high = find_byte(high.into())? << 4;
				let low = find_byte(low.into())?;

				acc[index] = high | low;

				Ok(acc)
			})?;

		Ok(bytes.into())
	}
}

#[derive(Debug, Clone, Copy, Error)]
pub enum HexFromStrError {
	#[error("input string was not ASCII")]
	NotAscii,

	#[error("input string was too long (expected len <= {max}, got {actual})")]
	InputTooLong {
		actual: usize,
		max: usize,
	},

	#[error("encountered non-hexadecimal character {code:?}", code = char::from_u32(*codepoint as u32))]
	UnexpectedCharacter {
		codepoint: u8,
	},
}

#[cfg(feat_array_chunks)]
#[inline]
fn as_chunks<'str>(s: &'str str) -> ArrayChunks<'str, u8, 2> {
	s.as_bytes().array_chunks::<2>()
}

#[cfg(not(feat_array_chunks))]
#[inline]
fn as_chunks<'str>(s: &'str str) -> impl Iterator<Item = &'str [u8; 2]> + DoubleEndedIterator + ExactSizeIterator {
	s.as_bytes().chunks_exact(2).map(|window| window.try_into().expect("a two-element slice"))
}
