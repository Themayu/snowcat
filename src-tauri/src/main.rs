#![warn(rust_2018_idioms)]
#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

mod commands;
mod state;

use crate::state::{CharacterMap, User};
use std::sync::{Arc, Mutex};

fn main() {
	let character_cache = Arc::new(Mutex::new(CharacterMap::default()));
	let user = Arc::new(Mutex::new(Option::<User<'_>>::None));

	tauri::Builder::default()
		.invoke_handler(tauri::generate_handler![
			commands::account_login,
			commands::settings_get
		])
		.manage(character_cache)
		.manage(user)
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
