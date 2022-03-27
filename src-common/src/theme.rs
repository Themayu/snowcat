use serde::{Deserialize, Serialize};
use std::convert::TryInto;
use std::env::current_exe;
use std::fmt::{Formatter, Result as FormatResult, UpperHex};
use std::io::Error as IoError;
use std::path::PathBuf;
use std::sync::Arc;

/// A colour value for use in a theme.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Color {
	red: u8,
	green: u8,
	blue: u8,
	alpha: u8,
}

impl Color {
	/// Create a solid colour from a red-green-blue triple.
	pub fn new(red: u8, green: u8, blue: u8) -> Self {
		Color { red, green, blue, alpha: 255 }
	}

	/// Create a colour from a red-green-blue triple with alpha transparency.
	pub fn new_alpha(red: u8, green: u8, blue: u8, alpha: u8) -> Self {
		Color { red, green, blue, alpha }
	}

	/// Create a colour value from a 32-bit representation.
	pub fn from_u32(color: u32) -> Self {
		let red: u8 = (color >> 24 & 0xFF).try_into().unwrap();
		let green: u8 = (color >> 16 & 0xFF).try_into().unwrap();
		let blue: u8 = (color >> 8 & 0xFF).try_into().unwrap();
		let alpha: u8 = (color & 0xFF).try_into().unwrap();

		Self::new_alpha(red, green, blue, alpha)
	}

	pub fn to_rgba(&self) -> String {
		format!(
			"rgba({}, {}, {}, {})",
			self.red, self.green, self.blue, self.alpha as f32 / 255.0
		)
	}
}

impl UpperHex for Color {
	fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
		write!(f, "{:X}", self.red)?;
		write!(f, "{:X}", self.green)?;
		write!(f, "{:X}", self.blue)?;

		if self.alpha > 0 {
			write!(f, "{:X}", self.alpha)?;
		}

		Ok(())
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Theme {
	pub name: Arc<String>,
	pub author: Arc<String>,

	pub background: BackgroundColors,
	pub header: HeaderColors,
	pub ui: UiColors,
	pub chat: ChatColors,
	pub sex: SexColors,
	pub status: StatusColors,
}

impl Theme {
	pub fn new(name: &str) -> Self {
		Theme {
			name: name.to_owned().into(),
			..Default::default()
		}
	}

	/// Returns the path to the file this theme is stored in, based on its name.
	///
	/// Theme files are stored in a `themes` directory in the executable
	/// directory. Each theme's file name is a lowercase variant of the theme
	/// name, with the file extension `.theme`.
	/// 
	/// # Errors
	/// 
	/// Since themes are stored alongside the current executable, this function
	/// can fail if the path to the executable cannot be retrieved. In this
	/// case, the error message will be that returned by [`current_exe`].
	/// 
	/// # Examples
	/// ```rust
	/// let theme = ColourTheme::new("Dark");
	/// let name = theme.file_path().and_then(|s| s.to_str()).unwrap()
	///
	/// assert!(path.ends_with("dark.theme"));
	/// ```
	pub fn file_path(&self) -> Result<PathBuf, IoError> {
		let mut path = current_exe()?;

		path.pop();
		path.push("themes");
		path.set_file_name(self.file_name());

		Ok(path)
	}

	/// Returns the file name this theme is stored in, based on its name.
	/// 
	/// A theme's file name is created by converting the theme name to
	/// lowercase and giving it the `.theme` file extension.
	/// 
	/// # Examples
	/// ```rust
	/// let theme = ColourTheme::new("Dark");
	/// let name = theme.file_name();
	/// 
	/// assert_eq!(name, "dark.theme");
	/// ```
	fn file_name(&self) -> String {
		format!("{}.theme", &self.name.to_lowercase())
	}
}

impl Default for Theme {
	fn default() -> Self {
		Theme {
			name: String::from("Test Theme").into(),
			author: String::from("Auto-generated").into(),

			background: BackgroundColors::default(),
			header: HeaderColors::default(),
			ui: UiColors::default(),
			chat: ChatColors::default(),
			sex: SexColors::default(),
			status: StatusColors::default(),
		}
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct BackgroundColors {
	pub base: Color,
	pub primary: Color,
	pub secondary: Color,
	pub secondary_alt: Color,
	pub tertiary: Color,
	pub floating: Color,
}

impl Default for BackgroundColors {
	fn default() -> Self {
		BackgroundColors {
			base: Color::from_u32(0x18191CFF),
			primary: Color::from_u32(0x36393FFF),
			secondary: Color::from_u32(0x2F3136FF),
			secondary_alt: Color::from_u32(0x292B2FFF),
			tertiary: Color::from_u32(0x202225FF),
			floating: Color::from_u32(0x36393FFF),
		}
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HeaderColors {
	// Headers
	pub primary: Color,
	pub secondary: Color,
}

impl Default for HeaderColors {
	fn default() -> Self {
		Self {
			primary: Color::from_u32(0xFFFFFFFF),
			secondary: Color::from_u32(0xB9BBBEFF),
		}
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UiColors {
	// App Text Colours
	pub text: Color,
	pub inactive: Color,
	pub muted: Color,
	pub positive: Color,
	pub warning: Color,
	pub danger: Color,
	pub info: Color,
	pub selection: Color,
}

impl Default for UiColors {
	fn default() -> Self {
		Self {
			text: Color::from_u32(0xDCDDDEFF),
			inactive: Color::from_u32(0x8E9297FF),
			muted: Color::from_u32(0x72767DFF),
			positive: Color::from_u32(0x4FDC7CFF),
			warning: Color::from_u32(0xFAA81AFF),
			danger: Color::from_u32(0xED4245FF),
			info: Color::from_u32(0x00AFF4FF),
			selection: Color::from_u32(0x0B68D9FF),
		}
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChatColors {
	pub blue: Color,
	pub brown: Color,
	pub cyan: Color,
	pub green: Color,
	pub orange: Color,
	pub pink: Color,
	pub purple: Color,
	pub red: Color,
	pub yellow: Color,
	pub black: Color,
	pub grey: Color,
	pub white: Color,
}

impl Default for ChatColors {
	fn default() -> Self {
		Self {
			blue: Color::from_u32(0x0066FFFF),
			brown: Color::from_u32(0x8A6D3BFF),
			cyan: Color::from_u32(0x00FFFFFF),
			green: Color::from_u32(0x00FF00FF),
			orange: Color::from_u32(0xFF6600FF),
			pink: Color::from_u32(0xFFB6C1FF),
			purple: Color::from_u32(0x9370DBFF),
			red: Color::from_u32(0xFF0000FF),
			yellow: Color::from_u32(0xFFFF00FF),
			black: Color::from_u32(0x000000FF),
			grey: Color::from_u32(0xCCCCCCFF),
			white: Color::from_u32(0xFFFFFFFF),
		}
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SexColors {
	pub cunt_boy: Color,
	pub female: Color,
	pub hermaphrodite: Color,
	pub male: Color,
	pub male_herm: Color,
	pub shemale: Color,
	pub transgender: Color,
	pub none_set: Color,
}

impl Default for SexColors {
	fn default() -> Self {
		Self {
			cunt_boy: Color::from_u32(0x00CC66FF),
			female: Color::from_u32(0xFF6699FF),
			hermaphrodite: Color::from_u32(0x9B30FFFF),
			male: Color::from_u32(0x6699FFFF),
			male_herm: Color::from_u32(0x007FFFFF),
			shemale: Color::from_u32(0xCC66FFFF),
			transgender: Color::from_u32(0xEE8822FF),
			none_set: Color::from_u32(0xDCDDDEFF),
		}
	}
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StatusColors {
	pub looking: Color,
	pub online: Color,
	pub away: Color,
	pub idle: Color,
	pub busy: Color,
	pub dnd: Color,
	pub offline: Color,
}

impl Default for StatusColors {
	fn default() -> Self {
		Self {
			looking: Color::from_u32(0x0FBBFFFF),
			online: Color::from_u32(0x4FDC7CFF),
			away: Color::from_u32(0xFAA81AFF),
			idle: Color::from_u32(0xFA811AFF),
			busy: Color::from_u32(0xED4245FF),
			dnd: Color::from_u32(0xED4245FF),
			offline: Color::from_u32(0x72767DFF),
		}
	}
}
