pub mod bookmarks;
pub mod characters;
pub mod friends;
pub mod lists;
pub mod ticket;

mod mock;

use crate::api::AccountCredentials;
use serde::ser::SerializeStruct;

/// Serialize an account object into a struct serializer, then end the
/// serializer.
fn serialize_account<S>(mut serializer: S, account: &AccountCredentials) -> Result<S::Ok, S::Error>
where
	S: SerializeStruct,
{
	serializer.serialize_field("account", &account.username)?;
	serializer.serialize_field("ticket", &account.ticket)?;
	serializer.end()
}
