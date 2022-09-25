use std::borrow::{Cow, Borrow};
use std::convert::TryInto;
use std::fmt;
use std::str::FromStr;
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

#[derive(Clone, Copy)]
pub struct HexByte(u8);

impl HexByte {
	pub fn to_byte(self) -> u8 {
		return self.0;
	}

	pub fn from_byte(byte: u8) -> HexByte {
		HexByte(byte)
	}
	
	unsafe fn get_char_unchecked(byte: u8) -> char {
		let byte: usize = byte.into();
		let value = TABLE.get_unchecked(byte).0;

		char::from_u32_unchecked(value as u32)
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

#[derive(Debug, Clone, Copy)]
pub struct Hex<const BYTES: usize>([HexByte; BYTES]);

impl<const N: usize> Hex<N> {
	pub fn from_bytes(bytes: [u8; N]) -> Self {
		Hex(bytes.map(HexByte::from_byte))
	}

	pub fn to_bytes(self) -> [u8; N] {
		self.0.map(HexByte::to_byte)
	}
}

impl<const N: usize> Default for Hex<N> {
	fn default() -> Self {
		Hex([HexByte::default(); N])
	}
}

impl<const N: usize> fmt::Display for Hex<N> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "0x")?;

		for c in self.0 {
			write!(f, "{c}")?
		}

		Ok(())
	}
}

impl<const BYTES: usize> FromStr for Hex<BYTES> {
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

		Ok(Self::from_bytes(bytes))
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
