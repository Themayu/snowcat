use snowcat_common::error;
use tauri;

#[tauri::command]
pub fn account_login(_window: tauri::Window) -> Result<(), error::Error> {
	Err(error::ApiError::InvalidCredentials.into())
}
