use crate::api::remote::constants::images::{
	AVATAR,
	CHARIMAGE,
	CHARINLINE,
	CHARTHUMB,
	EICON,
};

/// Get the URL for the given character's avatar.
pub fn avatar_url(name: &str) -> String {
	format!("{AVATAR}/{name}.png", name = name.to_lowercase())
}

/// Get the URL for the given gallery image.
pub fn charimage_url(id: u64, ext: &str) -> String {
	format!("{CHARIMAGE}/{id}.{ext}")
}

/// Get the URL for the given inline image.
pub fn charinline_url(hash: &str, ext: &str) -> String {
	let group_1 = &hash[0..2];
	let group_2 = &hash[2..4];

	format!("{CHARINLINE}/{group_1}/{group_2}/{hash}.{ext}")
}

/// Get the thumbnail URL for the given gallery image.
pub fn charthumb_url(id: u64, ext: &str) -> String {
	format!("{CHARTHUMB}/{id}.{ext}")
}

/// Get the URL for the given eicon.
pub fn eicon_url(name: &str) -> String {
	format!("{EICON}/{name}.gif")
}

#[cfg(test)]
mod tests {
	use crate::api::remote::constants::images::{
		AVATAR,
		CHARIMAGE,
		CHARINLINE,
		CHARTHUMB,
		EICON,
	};
    use super::{
		avatar_url,
		charimage_url,
		charinline_url,
		charthumb_url,
		eicon_url,
	};
	use const_str::concat as const_concat;

	#[test]
	fn image_avatar_url_endpoint() {
		const EXPECTED: &str = const_concat!(AVATAR, "/dragon wolf.png");
		assert_eq!(avatar_url("Dragon Wolf"), EXPECTED);
	}

	#[test]
	fn image_charimage_url_endpoint() {
		const EXPECTED: &str = const_concat!(CHARIMAGE, "/404.png");
		assert_eq!(charimage_url(404, "png"), EXPECTED);
	}

	#[test]
	fn image_charinline_url_endpoint() {
		const EXPECTED: &str = const_concat!(CHARINLINE, "/10/9e/109ea1e42706053aad1d748dac3451430d9ebcce.png");
		assert_eq!(charinline_url("109ea1e42706053aad1d748dac3451430d9ebcce", "png"), EXPECTED);
	}

	#[test]
	fn image_charthumb_url_endpoint() {
		const EXPECTED: &str = const_concat!(CHARTHUMB, "/404.png");
		assert_eq!(charthumb_url(404, "png"), EXPECTED);
	}

	#[test]
	fn image_eicon_url_endpoint() {
		const EXPECTED: &str = const_concat!(EICON, "/cheese.gif");
		assert_eq!(eicon_url("cheese"), EXPECTED);
	}
}
