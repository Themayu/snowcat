use crate::modules::tauri;
use crate::state::settings::AppSettings;
use futures::TryFutureExt;
use snowcat_common::error;
use snowcat_common::state::settings::{self, Settings as CommonSettings};

pub struct GetSettingsCommand;
impl GetSettingsCommand {
	pub async fn invoke(&self) -> Result<AppSettings, error::CommandError> {
		tauri::try_invoke("settings_get", None::<&()>).map_ok(|value: Option<CommonSettings>| {
			AppSettings::from(value.unwrap_or_else(|| {
				unimplemented!("if this actually returns null one day I am going to throw myself off a cliff")
			}))
		}).await
	}
}
