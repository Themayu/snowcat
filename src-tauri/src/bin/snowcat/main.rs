#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

use snowcat::{
	api,
	client,
	socket,
	state,
	util,
};
use snowcat::state::tauri as tauri_state;
use tauri::async_runtime::RwLock;

fn main() {
	let State { channels, characters } = create_state();

	tauri::Builder::default()
		.manage(channels)
		.manage(characters)
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}

struct State {
	channels: RwLock<tauri_state::ChannelList>,
	characters: RwLock<tauri_state::CharacterList>,
}

fn create_state() -> State {
	State {
		channels: RwLock::default(),
		characters: RwLock::default(),
	}
}
