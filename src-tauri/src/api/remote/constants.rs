use const_str::{
	format as const_format,
	replace as const_replace,
};

pub mod api {
	use const_str::{
		concat as const_concat,
		replace as const_replace,
	};

	/// API URL base.
	const API_BASE: &str = "https://www.f-list.net/json/api";

	/// **`POST`** Acquire an API ticket. Tickets last for 30 minutes, and
	/// invalidate all previously acquired tickets for the account when issued.
	/// 
	/// Parameters:
	/// - `account`: the account to acquire a ticket for.
	/// - `password`: the account's current password.
	/// - *optional* `no_characters`: don't include the list of characters
	///   in the response.
	/// - *optional* `no_friends`: don't include the list of friends in the
	///   response.
	/// - *optional* `no_bookmarks`: don't include the list of bookmarks in the
	///   response.
	/// - *optional* `new_character_list`: use an alternative format for the
	///   character list.
	pub const GET_API_TICKET: &str = const_replace!(API_BASE, "/api", "/getApiTicket.php");

	/// **`POST`** Get the global list of all F-List groups.
	/// 
	/// Parameters:
	/// - `account`: the account to issue the request from.
	/// - `ticket`: a valid API ticket for the account.
	pub const GET_GROUP_LIST: &str = const_concat!(API_BASE, "/group-list.php");

	/// **`POST`** Get a list of all profiles the account has on chat-ignore.
	/// 
	/// Parameters:
	/// - `account`: the account to get the list from.
	/// - `ticket`: a valid API ticket for the account.
	pub const GET_IGNORE_LIST: &str = const_concat!(API_BASE, "/ignore-list.php");

	/// **`POST`** Get the global list of profile info fields.
	/// 
	/// Parameters: None
	pub const GET_INFO_LIST: &str = const_concat!(API_BASE, "/info-list.php");

	/// **`POST`** Get the global list of kinks.
	/// 
	/// Parameters: None
	pub const GET_KINK_LIST: &str = const_concat!(API_BASE, "/kink-list.php");

	/// **`POST`** Get the global list of kinks, infotags, infotag groups, and
	/// list items.
	/// 
	/// Parameters: None
	pub const GET_MAPPING_LIST: &str = const_concat!(API_BASE, "/mapping-list.php");

	/// **`POST`** Bookmark a profile.
	/// 
	/// Parameters:
	/// - `name`: the character to bookmark.
	/// - `account`: the account to add the bookmark to.
	/// - `ticket`: a valid API ticket for the account.
	pub const BOOKMARK_ADD: &str = const_concat!(API_BASE, "/bookmark-add.php");

	/// **`POST`** List all bookmarked profiles.
	/// 
	/// Parameters:
	/// - `account`: the account to retrieve the list from.
	/// - `ticket`: a valid API ticket for the account.
	pub const BOOKMARK_LIST: &str = const_concat!(API_BASE, "/bookmark-list.php");

	/// **`POST`** Remove a bookmark.
	/// 
	/// Parameters:
	/// - `name`: the character to remove.
	/// - `account`: the account to remove the bookmark from.
	/// - `ticket`: a valid API ticket for the account.
	pub const BOOKMARK_REMOVE: &str = const_concat!(API_BASE, "/bookmark-remove.php");

	/// **`POST`** Get a character's information.
	/// 
	/// Parameters:
	/// - `name`: the character to get information for.
	/// - `account`: the account to issue the request from.
	/// - `ticket`: a valid API ticket for the account.
	/// 
	/// Errors:
	/// - Will return an error if called on a character that is banned, timed
	///   out, blocked, or deleted.
	pub const CHARACTER_DATA: &str = const_concat!(API_BASE, "/character-data.php");

	/// **`POST`** Get a list of all the account's characters.
	/// 
	/// Parameters
	/// - `account`: the account to list characters from.
	/// - `ticket`: a valid API ticket for the account.
	pub const CHARACTER_LIST: &str = const_concat!(API_BASE, "/character-list.php");

	/// **`POST`** Get a list of all the account's friends.
	/// 
	/// Parameters:
	/// - `account`: the account to retrieve the friend list from.
	/// - `ticket`: a valid API ticket for the account.
	pub const FRIEND_LIST: &str = const_concat!(API_BASE, "/friend-list.php");

	/// **`POST`** Remove a profile from the account's friends list.
	/// 
	/// Parameters:
	/// - `source_name`: the character to remove the friend from.
	/// - `dest_name`: the character to remove from the friends list.
	/// - `account`: the account to remove the friend from.
	/// - `ticket`: a valid API ticket for the account.
	pub const FRIEND_REMOVE: &str = const_concat!(API_BASE, "/friend-remove.php");

	/// **`POST`** Accept an incoming friend request.
	/// 
	/// Parameters:
	/// - `request_id`: the friend request to accept.
	/// - `account`: the account to accept the friend request from.
	/// - `ticket`: a valid API ticket for the account.
	pub const FRIEND_REQUEST_ACCEPT: &str = const_concat!(API_BASE, "/request-accept.php");

	/// **`POST`** Cancel an outgoing friend request.
	/// 
	/// Parameters:
	/// - `request_id`: the friend request to cancel.
	/// - `account`: the account to cancel the friend request from.
	/// - `ticket`: a valid API ticket for the account.
	pub const FRIEND_REQUEST_CANCEL: &str = const_concat!(API_BASE, "/request-cancel.php");

	/// **`POST`** Deny a friend request.
	/// 
	/// Parameters:
	/// - `request_id`: the friend request to deny.
	/// - `account`: the account to deny the friend request from.
	/// - `ticket`: a valid API ticket for the account.
	pub const FRIEND_REQUEST_DENY: &str = const_concat!(API_BASE, "/request-deny.php");

	/// **`POST`** Get all incoming friend requests.
	/// 
	/// Parameters:
	/// - `account`: the account to retrieve the list from.
	/// - `ticket`: a valid API ticket for the account.
	pub const FRIEND_REQUEST_LIST_INCOMING: &str = const_concat!(API_BASE, "/request-list.php");

	/// **`POST`** Get all outgoing friend requests.
	/// 
	/// Parameters:
	/// - `account`: the account to retrieve the list from.
	/// - `ticket`: a valid API ticket for the account.
	pub const FRIEND_REQUEST_LIST_OUTGOING: &str = const_concat!(API_BASE, "/request-pending.php");

	/// **`POST`** Send a friend request.
	/// 
	/// Parameters:
	/// - `source_name`: the character to send the friend request from.
	/// - `dest_name`: the character to send the friend request to.
	/// - `account`: the account to send the friend request from.
	/// - `ticket`: a valid API ticket for the account.
	pub const SEND_FRIEND_REQUEST: &str = const_concat!(API_BASE, "/request-send.php");
}

pub mod character_info {
	pub mod furrypref {
		pub const FURRYPREF_HUMANS_ONLY: &str = "No furry characters, just humans";
		pub const FURRYPREF_HUMANS_PREFERRED: &str = "Furries ok, Humans Preferred";
		pub const FURRYPREF_NONE: &str = "Furs and / or humans";
		pub const FURRYPREF_FURRIES_PREFERRED: &str = "Humans ok, Furries Preferred";
		pub const FURRYPREF_FURRIES_ONLY: &str = "No humans, just furry characters";
	}

	pub mod gender {
		pub const GENDER_MALE: &str = "Male";
		pub const GENDER_FEMALE: &str = "Female";
		pub const GENDER_CUNT_BOY: &str = "Cunt-boy";
		pub const GENDER_HERM: &str = "Herm";
		pub const GENDER_MALE_HERM: &str = "Male-Herm";
		pub const GENDER_SHEMALE: &str = "Shemale";
		pub const GENDER_TRANS: &str = "Transgender";
		pub const GENDER_NONE: &str = "None";
	}

	pub mod language {
		pub const LANGUAGE_ARABIC: &str = "Arabic";
		pub const LANGUAGE_CHINESE: &str = "Chinese";
		pub const LANGUAGE_DUTCH: &str = "Dutch";
		pub const LANGUAGE_ENGLISH: &str = "English";
		pub const LANGUAGE_FRENCH: &str = "French";
		pub const LANGUAGE_GERMAN: &str = "German";
		pub const LANGUAGE_ITALIAN: &str = "Italian";
		pub const LANGUAGE_JAPANESE: &str = "Japanese";
		pub const LANGUAGE_KOREAN: &str = "Korean";
		pub const LANGUAGE_PORTUGUESE: &str = "Portuguese";
		pub const LANGUAGE_RUSSIAN: &str = "Russian";
		pub const LANGUAGE_SPANISH: &str = "Spanish";
		pub const LANGUAGE_SWEDISH: &str = "Swedish";
		pub const LANGUAGE_OTHER: &str = "Other";
	}

	pub mod orientation {
		pub const ORIENTATION_ASEXUAL: &str = "Asexual";
		pub const ORIENTATION_BI_CURIOUS: &str = "Bi-curious";
		pub const ORIENTATION_BI_FEMALE_PREFERENCE: &str = "Bi - female preference";
		pub const ORIENTATION_BI_MALE_PREFERENCE: &str = "Bi - male preference";
		pub const ORIENTATION_BISEXUAL: &str = "Bisexual";
		pub const ORIENTATION_GAY: &str = "Gay";
		pub const ORIENTATION_PANSEXUAL: &str = "Pansexual";
		pub const ORIENTATION_STRAIGHT: &str = "Straight";
		pub const ORIENTATION_UNSURE: &str = "Unsure";
	}

	pub mod role {
		pub const ROLE_DOM_ONLY: &str = "Always dominant";
		pub const ROLE_DOM_PREFERENCE: &str = "Usually dominant";
		pub const ROLE_SWITCH: &str = "Switch";
		pub const ROLE_SUB_PREFERENCE: &str = "Usually submissive";
		pub const ROLE_SUB_ONLY: &str = "Always submissive";
	}

	pub mod status {
		pub const STATUS_CROWN: &str = "crown";
		pub const STATUS_ONLINE: &str = "online";
		pub const STATUS_LOOKING: &str = "looking";
		pub const STATUS_IDLE: &str = "idle";
		pub const STATUS_AWAY: &str = "away";
		pub const STATUS_BUSY: &str = "busy";
		pub const STATUS_DO_NOT_DISTURB: &str = "dnd";
	}
}

pub mod headers {
	pub const USER_AGENT: &str = "User-Agent";
	pub const USER_AGENT_VALUE: &str = concat!("Snowcat/", env!("CARGO_PKG_VERSION"));
}

pub mod images {
	use const_str::concat as const_concat;

	/// Static image content URL base.
	const IMAGES_BASE: &str = "https://static.f-list.net/images";

	/// Character avatar URL base.
	pub const AVATAR: &str = const_concat!(IMAGES_BASE, "/avatar");
	
	/// Character gallery image URL base.
	pub const CHARIMAGE: &str = const_concat!(IMAGES_BASE, "/charimage");
	
	/// Character gallery thumbnail URL base.
	pub const CHARTHUMB: &str = const_concat!(IMAGES_BASE, "/charthumb");
	
	/// EIcon URL base.
	pub const EICON: &str = const_concat!(IMAGES_BASE, "/eicon");
}

pub const CLIENT_NAME: &str = const_format!(
	"Snowcat/{version} by {authors}",
	version = env!("CARGO_PKG_VERSION"),
	authors = const_replace!(env!("CARGO_PKG_AUTHORS"), ":", ", "),
);
