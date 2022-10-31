use snowcat::api::Account;
use snowcat::api::characters::CharacterInfo;
use snowcat::api::error::Result as ApiResult;
use snowcat::client::Client;
use snowcat::tauri::commands::account::AuthenticateReturn;
use tauri::State;

#[tauri::command]
pub async fn authenticate(
	client: State<'_, Client>,
	username: String,
	password: String,
) -> ApiResult<AuthenticateReturn> {
	client.authenticate(&username, &password).await?;

	let Account { default_character, characters_list, .. } = &*client.account_info().await;

	let characters_list = characters_list.into_iter()
		.map(|(_, name)| CharacterInfo::default_for(name))
		.collect();

	Ok(AuthenticateReturn::new(characters_list, *default_character))
}
