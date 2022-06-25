use snowcat_common::error;
use snowcat_common::state::settings::Settings;
use tauri;

#[tauri::command]
pub fn settings_get() -> Result<Settings, error::CommandError> {
	Ok(Settings::default())
}
