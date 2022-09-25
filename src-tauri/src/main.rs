#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

#![cfg_attr(feat_array_chunks, feature(array_chunks))]

mod api;
mod util;

fn main() {
	tauri::Builder::default()
		.run(tauri::generate_context!())
		.expect("error while running tauri application");
}
