use crate::api::channels;
use crate::api::characters;
use crate::api::remote::commands::server::helpers::command_prefix;
use serde::Deserialize;
use serde_json::Value;
use serde_with::{serde_as, DefaultOnError, DisplayFromStr};
use std::collections::HashMap;
use std::marker::PhantomData;

// -----------------------------------------------------------------------------
// COMMANDS
// -----------------------------------------------------------------------------

// CHANNEL FUNCTIONALITY

#[derive(Debug, Deserialize)]
pub struct ChannelChangeDescription<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) description: &'data str,
}

command_prefix!(ChannelChangeDescription<'_>, "CDS");

#[derive(Debug, Deserialize)]
pub struct ChannelChangeMode<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) mode: channels::ChannelMode,
}

command_prefix!(ChannelChangeMode<'_>, "RMO");

#[derive(Debug, Deserialize)]
pub struct ChannelChangeOwner<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) character: &'data str,
}

command_prefix!(ChannelChangeOwner<'_>, "CSO");

#[derive(Debug, Deserialize)]
pub struct ChannelData<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	#[serde(borrow, rename = "users")]
	pub(in crate::api) characters: Vec<data::CharacterName<'data>>,

	pub(in crate::api) mode: channels::ChannelMode,
}

command_prefix!(ChannelData<'_>, "ICH");

#[derive(Debug, Deserialize)]
pub struct ChannelDiceRoll<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) character: &'data str,
	pub(in crate::api) message: &'data str,

	#[serde(borrow, flatten)]
	pub(in crate::api) roll_data: data::ChannelRollType<'data>,
}

command_prefix!(ChannelDiceRoll<'_>, "RLL");

#[derive(Debug, Deserialize)]
pub struct ChannelInvitation<'data> {
	pub(in crate::api) sender: &'data str,
	pub(in crate::api) title: &'data str,
	pub(in crate::api) name: &'data str,
}

command_prefix!(ChannelInvitation<'_>, "CIU");

// CHANNEL MODERATION

#[derive(Debug, Deserialize)]
pub struct ChannelMemberBan<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) operator: &'data str,
	pub(in crate::api) character: &'data str,
}

command_prefix!(ChannelMemberBan<'_>, "CBU");

#[derive(Debug, Deserialize)]
pub struct ChannelMemberDemotion<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) character: &'data str,
}

command_prefix!(ChannelMemberDemotion<'_>, "COR");

#[derive(Debug, Deserialize)]
pub struct ChannelMemberKick<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) operator: &'data str,
	pub(in crate::api) character: &'data str,
}

command_prefix!(ChannelMemberKick<'_>, "CKU");

#[derive(Debug, Deserialize)]
pub struct ChannelMemberPromotion<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) character: &'data str,
}

command_prefix!(ChannelMemberPromotion<'_>, "COA");

#[derive(Debug, Deserialize)]
pub struct ChannelMemberTimeout<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) operator: &'data str,
	pub(in crate::api) character: &'data str,

	#[serde(rename = "length")]
	pub(in crate::api) timeout_duration: u8,
}

command_prefix!(ChannelMemberTimeout<'_>, "CTU");

#[derive(Debug, Deserialize)]
pub struct ChannelOpsList<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	#[serde(borrow, rename = "oplist")]
	pub(in crate::api) ops: Vec<&'data str>,
}

command_prefix!(ChannelOpsList<'_>, "COL");

// CHANNEL LIST

#[derive(Debug, Deserialize)]
pub struct ChannelsListOpen<'data> {
	#[serde(borrow)]
	pub(in crate::api) channels: Vec<data::ChannelDataOpen<'data>>,
}

command_prefix!(ChannelsListOpen<'_>, "ORS");

#[derive(Debug, Deserialize)]
pub struct ChannelsListPublic<'data> {
	#[serde(borrow)]
	pub(in crate::api) channels: Vec<data::ChannelDataPublic<'data>>,
}

command_prefix!(ChannelsListPublic<'_>, "CHA");

// CHARACTER STATUS

#[derive(Debug)]
pub struct CharacterChangeStatus<'data> {
	pub(in crate::api) character: &'data str,
	pub(in crate::api) status: &'data str,
	pub(in crate::api) message: Option<&'data str>,
}

command_prefix!(CharacterChangeStatus<'_>, "STA");

// we can't deserialize `"" => None, value => Some(value)` using a helper
// due to lifetime issues.
impl<'data, 'de: 'data> Deserialize<'de> for CharacterChangeStatus<'data> {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'de>,
	{
		use serde::de::{self, Error};
		use std::fmt;

		#[derive(Deserialize)]
		#[serde(field_identifier, rename_all = "lowercase")]
		enum Field {
			Character,
			#[serde(rename = "statusmsg")] Message,
			Status,
		}

		struct CommandVisitor<'data, 'de: 'data> {
			_deserializer: PhantomData<&'de ()>,
			_command: PhantomData<CharacterChangeStatus<'data>>,
		}

		impl<'data, 'de: 'data> de::Visitor<'de> for CommandVisitor<'data, 'de> {
			type Value = CharacterChangeStatus<'data>;

			fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
				write!(f, "struct CharacterStatusChange")
			}

			#[inline]
			fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
			where
				A: de::SeqAccess<'de>,
			{
				const ERR: &str = "struct CharacterStatusChange with 3 elements";

				let status: &'data str = seq.next_element()?
					.ok_or_else(|| Error::invalid_length(0, &ERR))?;

				let character: &'data str = seq.next_element()?
					.ok_or_else(|| Error::invalid_length(1, &ERR))?;

				let message: &'data str = seq.next_element()?
					.ok_or_else(|| Error::invalid_length(2, &ERR))?;

				Ok(CharacterChangeStatus {
					character,
					status,

					message: match message {
						"" => None,
						value => Some(value),
					},
				})
			}

			#[inline]
			fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
			where
				A: de::MapAccess<'de>,
			{
				let mut character: Option<&'data str> = None;
				let mut status: Option<&'data str> = None;
				let mut message: Option<&'data str> = None;

				while let Some(key) = map.next_key()? {
					#[allow(unreachable_patterns)]
					match key {
						Field::Character => helpers::assign_field!(map => character),
						Field::Status => helpers::assign_field!(map => status),
						Field::Message => helpers::assign_field!(map => message),

						// ignore unrecognised keys
						_ => {
							let _: de::IgnoredAny = map.next_value()?;
						}
					}
				}

				helpers::unwrap_field!(character);
				helpers::unwrap_field!(status);
				helpers::unwrap_field!(message);

				Ok(CharacterChangeStatus {
					character,
					status,

					message: match message {
						"" => None,
						value => Some(value),
					},
				})
			}
		}

		const FIELDS: [&str; 3] = ["channel", "character", "message"];
		deserializer.deserialize_struct("CharacterStatusChange", &FIELDS, CommandVisitor {
			_command: PhantomData,
			_deserializer: PhantomData,
		})
	}
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct CharacterChangeTypingStatus<'data> {
	pub(in crate::api) character: &'data str,

	#[serde_as(deserialize_as = "DefaultOnError")]
	pub(in crate::api) status: super::CharacterTypingStatus,
}

command_prefix!(CharacterChangeTypingStatus<'_>, "TPN");

#[derive(Debug, Deserialize)]
pub struct CharacterJoinedChannel<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	#[serde(rename = "title")]
	pub(in crate::api) channel_name: &'data str,

	#[serde(borrow)]
	pub(in crate::api) character: data::CharacterName<'data>
}

command_prefix!(CharacterJoinedChannel<'_>, "JCH");

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CharacterKinksList<'data> {
	Start {
		message: &'data str,
	},

	Custom {
		key: &'data str,
		value: &'data str,
	},

	End {
		message: &'data str,
	}
}

command_prefix!(CharacterKinksList<'_>, "KID");

#[derive(Debug, Deserialize)]
pub struct CharacterLeftChannel<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) character: &'data str,
}

command_prefix!(CharacterLeftChannel<'_>, "LCH");

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct CharacterLoggedIn<'data> {
	#[serde(rename = "identity")]
	pub(in crate::api) character: &'data str,

	#[serde_as(deserialize_as = "DefaultOnError")]
	pub(in crate::api) gender: characters::CharacterGender,

	pub(in crate::api) status: &'data str,
}

command_prefix!(CharacterLoggedIn<'_>, "NLN");

#[derive(Debug, Deserialize)]
pub struct CharacterLoggedOut<'data> {
	pub(in crate::api) character: &'data str,
}

command_prefix!(CharacterLoggedOut<'_>, "FLN");

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum CharacterProfileData<'data> {
	Start {
		message: &'data str,
	},

	Info {
		key: &'data str,
		value: &'data str,
	},

	Select {
		key: &'data str,
		value: &'data str,
	},

	End {
		message: &'data str,
	},
}

command_prefix!(CharacterProfileData<'_>, "PRD");

// MESSAGING FUNCTIONALITY

#[derive(Debug)]
pub struct ClientHeartbeatRequest;
command_prefix!(ClientHeartbeatRequest, "PIN");

#[derive(Debug, Deserialize)]
pub struct RealTimeBridgeMessage<'data> {
	#[serde(rename = "type")]
	pub(in crate::api) kind: data::RealTimeBridgeMessageKind,
	pub(in crate::api) character: &'data str,
}

command_prefix!(RealTimeBridgeMessage<'_>, "RTB");

#[derive(Debug, Deserialize)]
pub struct ReceiveAd<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) character: &'data str,
	pub(in crate::api) message: &'data str,
}

command_prefix!(ReceiveAd<'_>, "LRP");

#[derive(Debug, Deserialize)]
pub struct ReceiveMessage<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) character: &'data str,
	pub(in crate::api) message: &'data str,
}

command_prefix!(ReceiveMessage<'_>, "MSG");

#[derive(Debug, Deserialize)]
pub struct ReceivePrivateMessage<'data> {
	pub(in crate::api) character: &'data str,
	pub(in crate::api) message: &'data str,
}

command_prefix!(ReceivePrivateMessage<'_>, "PRI");

// SERVER MODERATION

#[derive(Debug, Deserialize)]
pub struct ServerBroadcast<'data> {
	pub(in crate::api) message: &'data str,
}

command_prefix!(ServerBroadcast<'_>, "BRO");

#[derive(Debug, Deserialize)]
pub struct ServerMemberDemotion<'data> {
	pub(in crate::api) character: &'data str,
}

command_prefix!(ServerMemberDemotion<'_>, "DOP");

#[derive(Debug, Deserialize)]
pub struct ServerMemberPromotion<'data> {
	pub(in crate::api) character: &'data str,
}

command_prefix!(ServerMemberPromotion<'_>, "PRO");

#[derive(Debug, Deserialize)]
pub struct ServerOpsList<'data> {
	#[serde(borrow)]
	pub(in crate::api) ops: Vec<&'data str>,
}

command_prefix!(ServerOpsList<'_>, "ADL");

// SERVER STATUS

#[derive(Debug, Deserialize)]
pub struct ServerCharacterCount {
	#[serde(rename = "count")]
	pub(in crate::api) character_count: u64,
}

command_prefix!(ServerCharacterCount, "CON");

#[derive(Debug, Deserialize)]
pub struct ServerCharactersList<'data> {
	#[serde(borrow)]
	pub(in crate::api) characters: Vec<[&'data str; 4]>,
}

command_prefix!(ServerCharactersList<'_>, "LIS");

#[derive(Debug, Deserialize)]
pub struct ServerError<'data> {
	#[serde(rename = "number")]
	pub(in crate::api) code: u64,

	pub(in crate::api) message: &'data str,
}

command_prefix!(ServerError<'_>, "ERR");

#[derive(Debug, Deserialize)]
pub struct ServerMessage<'data> {
	#[serde(rename = "channel")]
	pub(in crate::api) channel_id: &'data str,

	pub(in crate::api) message: &'data str,
}

command_prefix!(ServerMessage<'_>, "SYS");

#[derive(Debug, Deserialize)]
pub struct ServerUptime<'data> {
	#[serde(rename = "time")]
	pub(in crate::api) uptime: u64,

	#[serde(rename = "starttime")]
	pub(in crate::api) boot_timestamp: u64,

	#[serde(rename = "startstring")]
	pub(in crate::api) boot_datetime: &'data str,

	#[serde(rename = "accepted")]
	pub(in crate::api) total_accepted_connections_count: u64,

	#[serde(rename = "users")]
	pub(in crate::api) current_user_count: u64,

	#[serde(rename = "maxusers")]
	pub(in crate::api) peak_user_count: u64,
}

command_prefix!(ServerUptime<'_>, "UPT");

#[derive(Debug, Deserialize)]
#[serde(tag = "variable")]
pub enum ServerVariable<'data> {
	#[serde(rename = "chat_max")]
	MaxChannelMessageLength {
		value: u64,
	},

	#[serde(rename = "priv_max")]
	MaxPrivateMessageLength {
		value: u64
	},

	#[serde(rename = "lfrp_max")]
	MaxAdLength {
		value: u64,
	},

	#[serde(rename = "lfrp_flood")]
	GlobalAdTimeout {
		value: f64,
	},

	#[serde(rename = "msg_flood")]
	GlobalMessageTimeout {
		value: f64,
	},

	#[serde(rename = "icon_blacklist")]
	IconBlacklist {
		#[serde(borrow)]
		value: Vec<&'data str>,
	},

	#[serde(rename = "permissions")]
	CharacterPermisions {
		value: data::UserPermissionFlags,
	},
}

command_prefix!(ServerVariable<'_>, "VAR");

#[derive(Debug, Deserialize)]
pub struct ServerWelcome<'data> {
	pub(in crate::api) message: &'data str,
}

command_prefix!(ServerWelcome<'_>, "HLO");

// OWN USER STATUS

#[derive(Debug, Deserialize)]
pub struct UserFriendsList<'data> {
	#[serde(borrow)]
	pub(in crate::api) characters: Vec<&'data str>,
}

command_prefix!(UserFriendsList<'_>, "FRL");

#[derive(Debug, Deserialize)]
pub struct UserIdentificationSuccessful<'data> {
	#[serde(rename = "character")]
	pub(in crate::api) character_name: &'data str,
}

command_prefix!(UserIdentificationSuccessful<'_>, "IDN");

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UserIgnoreListAction<'data> {
	Init {
		#[serde(borrow)]
		characters: Vec<&'data str>
	},

	Add {
		character: &'data str,
	},

	Delete {
		character: &'data str,
	},

	List {
		#[serde(flatten)]
		_fields: HashMap<String, Value>,
	},

	Notify {
		#[serde(flatten)]
		_fields: HashMap<String, Value>,
	}
}

command_prefix!(UserIgnoreListAction<'_>, "IGN");

// TODO: Figure out how to deserialize SFC (may require assistance from server
// developer? try to find one - alternatively look at public backend code)
//
// https://wiki.f-list.net/F-Chat_Server_Commands#SFC
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct UserSupportRequest<'data> {
	#[serde(rename = "callid")]
	pub(in crate::api) alert_id: u64,

	#[serde(rename = "character")]
	pub(in crate::api) issuer_name: &'data str,

	#[serde(borrow, flatten)]
	pub(in crate::api) data: data::UserSupportRequestData<'data>,

	#[serde_as(as = "DisplayFromStr")]
	pub(in crate::api) timestamp: u64,
}

command_prefix!(UserSupportRequest<'_>, "SFC");

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct UserSearchResults<'data> {
	#[serde(borrow)]
	pub(in crate::api) characters: Vec<&'data str>,

	#[serde_as(as = "Vec<DisplayFromStr>")]
	pub(in crate::api) kinks: Vec<u64>,
}

command_prefix!(UserSearchResults<'_>, "FKS");

// -----------------------------------------------------------------------------
// DATA
// -----------------------------------------------------------------------------
mod data {
	use crate::api::channels;
    use bitflags::bitflags;
    use serde::Deserialize;
    use serde_with::DeserializeFromStr;
	use std::str;

	#[derive(Debug, Deserialize)]
	pub struct ChannelDataOpen<'data> {
		#[serde(rename = "name")]
		pub(in crate::api) channel_id: &'data str,

		#[serde(rename = "title")]
		pub(in crate::api) channel_name: channels::ChannelMode,

		#[serde(rename = "characters")]
		pub(in crate::api) member_count: u64,
	}

	#[derive(Debug, Deserialize)]
	pub struct ChannelDataPublic<'data> {
		#[serde(rename = "name")]
		pub(in crate::api) channel_id: &'data str,
		pub(in crate::api) mode: channels::ChannelMode,

		#[serde(rename = "characters")]
		pub(in crate::api) member_count: u64,
	}

	#[derive(Debug, Deserialize)]
	#[serde(tag = "type", rename_all = "snake_case")]
	pub enum ChannelRollType<'data> {
		Dice {
			results: Vec<u64>,

			#[serde(borrow)]
			rolls: Vec<&'data str>,

			#[serde(rename = "endresult")]
			total: u64,
		},

		Bottle {
			target: &'data str,
		}
	}

	#[derive(Debug, Deserialize)]
	pub struct CharacterName<'data> {
		#[serde(rename = "identity")]
		pub(in crate::api) name: &'data str,
	}

	#[derive(Debug, Deserialize)]
	#[serde(rename_all = "lowercase")]
	pub enum RealTimeBridgeMessageKind {
		Message,
		Note,
	}

	bitflags! {
		#[repr(transparent)]
		#[derive(DeserializeFromStr)]
		pub struct UserPermissionFlags: u32 {
			const ADMIN = 0x00000001; // 1
			const CHAT_CHAT_OP = 0x00000002; // 2
			const CHAT_CHAN_OP = 0x00000004; // 4
			const HELPDESK_CHAT = 0x00000008; // 8
			const HELPDESK_GENERAL = 0x00000010; // 16
			const MODERATION_SITE = 0x000000020; // 32
			const RESERVED = 0x00000040; // 64
			const MISC_GROUP_REQUESTS = 0x00000080; // 128
			const MISC_NEWS_POSTS = 0x00000100; // 256
			const MISC_CHANGELOG = 0x00000200; // 512
			const MISC_FEATURE_REQUESTS = 0x00000400; // 1024
			const DEV_BUG_REPORTS = 0x00000800; // 2048
			const DEV_TAGS = 0x00001000; // 4096
			const DEV_KINKS = 0x00002000; // 8192
			const DEVELOPER = 0x00004000; // 16384
			const TESTER = 0x00008000; // 32768
			const SUBSCRIPTIONS = 0x00010000; // 65536
			const FORMER_STAFF = 0x00020000; // 131072
		}
	}

	impl str::FromStr for UserPermissionFlags {
		type Err = <u32 as str::FromStr>::Err;

		fn from_str(value: &str) -> Result<Self, Self::Err> {
			let value: u32 = value.parse()?;

			Ok(UserPermissionFlags::from_bits_truncate(value))
		}
	}

	#[derive(Debug, Deserialize)]
	#[serde(tag = "action")]
	pub enum UserSupportRequestData<'data> {
		#[serde(rename = "report")]
		RequestReceived {
			#[serde(rename = "report")]
			report_text: &'data str,

			#[serde(rename = "logid")]
			log_id: u64,
		},

		#[serde(rename = "confirm")]
		RequestClaimed {
			#[serde(rename = "moderator")]
			handler_name: &'data str,
		},
	}
}

// -----------------------------------------------------------------------------
// HELPERS
// -----------------------------------------------------------------------------
mod helpers {
	macro_rules! assign_field {
		($map: ident => $field: ident) => {{
			if $field.is_some() {
				return Err(::serde::de::Error::duplicate_field(stringify!($field)));
			}

			$field = Some($map.next_value()?);
		}};
	}

	pub(super) use assign_field;

	macro_rules! command_prefix {
		($struct: ty, $command: literal) => {
			impl $struct {
				pub const COMMAND: &'static str = $command;
			}
		};
	}

	pub(super) use command_prefix;

	macro_rules! unwrap_field {
		($field: ident => $dest: ident) => {
			let $dest = match $field {
				Some($field) => $field,
				None => return Err(::serde::de::Error::missing_field(stringify!($field))),
			};
		};

		($field: ident) => {
			$crate::api::remote::commands::server::helpers::unwrap_field!($field => $field)
		};
	}

	pub(super) use unwrap_field;
}
