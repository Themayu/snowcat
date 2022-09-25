use crate::api::remote::characters;
use crate::api::remote::commands::client::helpers::command_prefix;
use serde::Serialize;
use serde::ser::SerializeStruct;
use serde_with::{serde_as, DisplayFromStr};

// -----------------------------------------------------------------------------
// COMMANDS
// -----------------------------------------------------------------------------

#[derive(Debug, Serialize)]
pub struct AdminUserBan {
	character: String,
}

command_prefix!(AdminUserBan, "ACB");

#[derive(Debug, Serialize)]
pub struct AdminUserPromote {
	character: String,
}

command_prefix!(AdminUserPromote, "AOP");

#[derive(Debug, Serialize)]
pub struct AdminUserListAlts {
	character: String,
}

command_prefix!(AdminUserListAlts, "AWC");

#[derive(Debug, Serialize)]
pub struct AdminBroadcastServerMessage {
	message: String,
}

command_prefix!(AdminBroadcastServerMessage, "BRO");

#[derive(Debug, Serialize)]
pub struct ChannelOpGetBanList {
	#[serde(rename = "channel")]
	channel_id: String,
}

command_prefix!(ChannelOpGetBanList, "CBL");

#[derive(Debug, Serialize)]
pub struct ChannelOpUserBan {
	#[serde(rename = "channel")]
	channel_id: String,

	character: String,
}

command_prefix!(ChannelOpUserBan, "CBU");

#[derive(Debug, Serialize)]
pub struct ChannelCreate {
	#[serde(rename = "channel")]
	channel_name: String,
}

command_prefix!(ChannelCreate, "CCR");

#[derive(Debug, Serialize)]
pub struct ChannelChangeDescription {
	#[serde(rename = "channel")]
	channel_id: String,

	description: String,
}

command_prefix!(ChannelChangeDescription, "CDS");

#[derive(Debug)]
pub struct ServerListPublicChannels;

command_prefix!(ServerListPublicChannels, "CHA");

#[derive(Debug, Serialize)]
pub struct ChannelOpUserInvite {
	#[serde(rename = "channel")]
	channel_id: String,

	character: String,
}

command_prefix!(ChannelOpUserInvite, "CIU");

#[derive(Debug, Serialize)]
pub struct ChannelOpUserKick {
	#[serde(rename = "channel")]
	channel_id: String,

	character: String,
}

command_prefix!(ChannelOpUserKick, "CKU");

#[derive(Debug, Serialize)]
pub struct ChannelGetOpList {
	#[serde(rename = "channel")]
	channel_id: String,
}

command_prefix!(ChannelGetOpList, "COL");

#[derive(Debug, Serialize)]
pub struct ChannelUserDemote {
	#[serde(rename = "channel")]
	channel_id: String,

	character: String,
}

command_prefix!(ChannelUserDemote, "COR");

#[derive(Debug, Serialize)]
pub struct AdminChannelCreateOfficial {
	#[serde(rename = "channel")]
	channel_name: String,
}

command_prefix!(AdminChannelCreateOfficial, "CRC");

#[derive(Debug, Serialize)]
pub struct ChannelTransferOwnership {
	#[serde(rename = "channel")]
	channel_id: String,

	#[serde(rename = "character")]
	new_owner: String,
}

command_prefix!(ChannelTransferOwnership, "CSO");

#[serde_as]
#[derive(Debug, Serialize)]
pub struct ChannelOpUserTimeout {
	#[serde(rename = "channel")]
	channel_id: String,

	character: String,

	#[serde_as(serialize_as = "DisplayFromStr")]
	length: u64,
}

command_prefix!(ChannelOpUserTimeout, "CTU");

#[derive(Debug, Serialize)]
pub struct ChannelOpUserRevokeBan {
	#[serde(rename = "channel")]
	channel_id: String,

	character: String,
}

command_prefix!(ChannelOpUserRevokeBan, "CUB");

#[derive(Debug, Serialize)]
pub struct AdminUserDemote {
	#[serde(rename = "channel")]
	channel_id: String,
}

command_prefix!(AdminUserDemote, "DOP");

#[serde_as]
#[derive(Debug, Serialize)]
pub struct UserSearch {
	kinks: Vec<u64>,

	#[serde_as(deserialize_as = "DefaultOnError")]
	genders: Vec<characters::CharacterGender>,

	#[serde_as(deserialize_as = "DefaultOnError")]
	orientations: Vec<characters::CharacterOrientation>,

	#[serde_as(deserialize_as = "DefaultOnError")]
	#[serde(rename = "furryprefs")]
	preferences: Vec<characters::CharacterPreference>,

	#[serde(rename = "languages")]
	language_preferences: Vec<characters::CharacterLanguagePreference>,

	#[serde_as(deserialize_as = "DefaultOnError")]
	roles: Vec<characters::CharacterRole>,
}

command_prefix!(UserSearch, "FKS");

#[serde_as]
#[derive(Debug, Serialize)]
pub struct UserIdentify {
	account: String,

	#[serde(rename = "cname")]
	client_name: String,

	#[serde_as(serialize_as = "DisplayFromStr")]
	#[serde(rename = "cversion")]
	client_version: super::ClientVersion,

	#[serde(flatten)]
	method: data::UserIdentificationData,
}

command_prefix!(UserIdentify, "IDN");

#[derive(Debug, Serialize)]
#[serde(tag = "action")]
pub enum UserIgnoreListAction {
	#[serde(rename = "add")]
	AddEntry {
		character: String,
	},

	#[serde(rename = "list")]
	GetEntries,

	#[serde(rename = "notify")]
	NotifyIgnored {
		character: String,
	},

	#[serde(rename = "delete")]
	RemoveEntry {
		character: String,
	},
}

command_prefix!(UserIgnoreListAction, "IGN");

#[derive(Debug, Serialize)]
pub struct ChannelJoin {
	#[serde(rename = "channel")]
	channel_id: String,
}

command_prefix!(ChannelJoin, "JCH");

#[derive(Debug, Serialize)]
pub struct ChannelDelete {
	#[serde(rename = "channel")]
	channel_id: String,
}

command_prefix!(ChannelDelete, "KIC");

#[derive(Debug, Serialize)]
pub struct AdminKickCharacter {
	character: String,
}

command_prefix!(AdminKickCharacter, "KIK");

#[derive(Debug, Serialize)]
pub struct CharacterGetKinksList {
	character: String,
}

command_prefix!(CharacterGetKinksList, "KIN");

#[derive(Debug, Serialize)]
pub struct ChannelLeave {
	#[serde(rename = "channel")]
	channel_id: String,
}

command_prefix!(ChannelLeave, "LCH");

#[derive(Debug, Serialize)]
pub struct ChannelSendAd {
	#[serde(rename = "channel")]
	channel_id: String,

	message: String,
}

command_prefix!(ChannelSendAd, "LRP");

#[derive(Debug, Serialize)]
pub struct ChannelSendMessage {
	#[serde(rename = "channel")]
	channel_id: String,

	message: String,
}

command_prefix!(ChannelSendMessage, "MSG");

#[derive(Debug)]
pub struct ServerListOpenChannels;

command_prefix!(ServerListOpenChannels, "ORS");

#[derive(Debug)]
pub struct ClientHeartbeatResponse;

command_prefix!(ClientHeartbeatResponse, "PIN");

#[derive(Debug, Serialize)]
pub struct CharacterSendMessage {
	#[serde(rename = "recipient")]
	character: String,

	message: String,
}

command_prefix!(CharacterSendMessage, "PRI");

#[derive(Debug, Serialize)]
pub struct CharacterGetProfileData {
	character: String,
}

command_prefix!(CharacterGetProfileData, "PRO");

#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ChannelRollDice {
	InChannel {
		#[serde(rename = "channel")]
		channel_id: String,

		dice: String,
	},

	InPrivate {
		#[serde(rename = "recipient")]
		character: String,

		dice: String,
	},
}

command_prefix!(ChannelRollDice, "RLL");

// TODO: Ask one of the developers how this should be serialized, and if there
// is a response.
//
// https://wiki.f-list.net/F-Chat_Client_Commands#RLD
// #[derive(Debug, Serialize)]
// pub struct AdminReloadServerConfig {
//
// }
//
// command_prefix!(AdminReloadServerConfig, "RLD");

#[derive(Debug, Serialize)]
pub struct ChannelSetMode {
	#[serde(rename = "channel")]
	channel_id: String,

	mode: super::ChannelMode,
}

command_prefix!(ChannelSetMode, "RMO");

#[derive(Debug, Serialize)]
pub struct ChannelSetVisibility {
	#[serde(rename = "channel")]
	channel_id: String,

	status: data::ChannelVisibility,
}

command_prefix!(ChannelSetVisibility, "RST");

#[derive(Debug, Serialize)]
pub struct AdminRewardCharacter {
	character: String,
}

command_prefix!(AdminRewardCharacter, "RWD");

// TODO: Figure out what's even going on with this command's documentation.
//
// https://wiki.f-list.net/F-Chat_Client_Commands#SFC
#[derive(Debug, Serialize)]
#[serde(tag = "action", rename_all = "lowercase")]
pub enum UserRequestSupport {
	Report {
		// reports 
		#[serde(rename = "report")]
		report_text: String,
	
		#[serde(rename = "logid")]
		log_id: i64,
		
		#[serde(rename = "tab")]
		channel_id: String,
	}
}

command_prefix!(UserRequestSupport, "SFC");

#[derive(Debug)]
pub struct UserChangeStatus {
	status: characters::CharacterStatusKind,
	message: Option<String>,
}

command_prefix!(UserChangeStatus, "STA");

impl Serialize for UserChangeStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
	{
        let mut struct_ = serializer.serialize_struct("UserChangeStatus", 2)?;
		struct_.serialize_field("status", &self.status)?;
		struct_.serialize_field("statusmsg", match self.message.as_deref() {
			Some(value) => value,
			None => "",
		})?;

		struct_.end()
    }
}

#[derive(Debug, Serialize)]
pub struct AdminUserTimeout {
	character: String,
	time: u64,
	reason: String,
}

command_prefix!(AdminUserTimeout, "TPN");

#[derive(Debug, Serialize)]
pub struct CharacterNotifyTypingStatus {
	character: String,
	status: super::CharacterTypingStatus,
}

command_prefix!(CharacterNotifyTypingStatus, "TPN");

#[derive(Debug, Serialize)]
pub struct AdminUserRevokeBan {
	character: String,
}

command_prefix!(AdminUserRevokeBan, "UNB");

#[derive(Debug, Serialize)]
pub struct ServerGetUptime;

command_prefix!(ServerGetUptime, "UPT");

// -----------------------------------------------------------------------------
// DATA
// -----------------------------------------------------------------------------
mod data {
    use serde::Serialize;

	#[derive(Debug, Serialize)]
	#[serde(rename_all = "lowercase")]
	pub enum ChannelVisibility {
		Private,
		Public,
	}

	#[derive(Debug, Serialize)]
	#[serde(tag = "method", rename_all = "lowercase")]
	pub enum UserIdentificationData {
		Ticket {
			ticket: String,
		}
	}
}

// -----------------------------------------------------------------------------
// HELPERS
// -----------------------------------------------------------------------------
mod helpers {
	macro_rules! command_prefix {
		($struct: ty, $command: literal) => {
			impl $struct {
				pub const COMMAND: &'static str = $command;
			}
		};
	}

	pub(super) use command_prefix;
}
