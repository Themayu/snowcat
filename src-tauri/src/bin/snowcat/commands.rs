mod account;

use tauri::{Invoke, Wry};

pub fn command_handler() -> impl Fn(Invoke<Wry>) + Send + Sync + 'static {
	tauri::generate_handler![
		account::authenticate,
	]
}
