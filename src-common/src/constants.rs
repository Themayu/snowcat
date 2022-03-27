/// Events to be sent back and forth between Tauri and Dominator.
pub mod event {
	/// Make an attempt at logging in. Transfers a [`user::Credentials`] object.
	pub const LOGIN_ATTEMPT: &'static str = "login-attempt";

	/// The login attempt failed. Transfers an Error signifying the reason.
	pub const LOGIN_FAILED: &'static str = "login-failed";

	/// The login attempt succeeded. Transfers a User containing the default
	/// character, the list of characters, the list of bookmarks, and the list
	/// of friends.
	pub const LOGIN_SUCCESS: &'static str = "login-success";
}

pub mod default {
	pub const DEFAULT_SERVER: &'static str = "wss://chat.f-list.net/chat2";
}

pub mod http {
	pub mod endpoints {
		pub const GET_TICKET: &'static str = "https://www.f-list.net/json/api/getApiTicket.php";
	}

	pub const USER_AGENT_HEADER: &'static str = "User-Agent";

	pub const USER_AGENT_VALUE: &'static str = concat!(
		"Snowcat/", env!("CARGO_PKG_VERSION"), " by ", env!("CARGO_PKG_AUTHORS")
	);
}
