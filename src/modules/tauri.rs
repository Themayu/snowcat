pub mod event;
pub mod window;

use serde::Serialize;
use serde::de::DeserializeOwned;
use wasm_bindgen::{JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;

pub async fn invoke<Input, Output>(cmd: &str, args: Option<&Input>) -> Result<Option<Output>, JsValue>
where Input: Serialize,
	  Output: DeserializeOwned,
{
	let promise = if let Some(args) = args {
		tauri_sys::invoke_parameters(cmd, JsValue::from_serde(&args).unwrap_throw())
	} else {
		tauri_sys::invoke_none(cmd)
	};

	let result = JsFuture::from(promise).await?;
	let value = if !result.is_null() & !result.is_undefined() {
		Some(result.into_serde::<Output>().unwrap_throw())
	} else {
		None
	};

	Ok(value)
}

pub async fn try_invoke<Input, Output, Error>(cmd: &str, args: Option<&Input>) -> Result<Option<Output>, Error>
where Input: Serialize,
	  Output: DeserializeOwned,
	  Error: DeserializeOwned,
{
	let promise = if let Some(args) = args {
		tauri_sys::invoke_parameters(cmd, JsValue::from_serde(&args).unwrap_throw())
	} else {
		tauri_sys::invoke_none(cmd)
	};

	let result = JsFuture::from(promise).await;
	
	match result {
		Ok(value) => {
			let value = if !value.is_null() & !value.is_undefined() {
				Some(value.into_serde::<Output>().unwrap_throw())
			} else {
				None
			};

			Ok(value)
		},

		Err(error) => Err(error.into_serde::<Error>().unwrap_throw()),
	}
}

mod tauri_sys {
	use js_sys::Promise;
	use wasm_bindgen::prelude::*;

	#[wasm_bindgen(module = "@tauri-apps/api/tauri")]
	extern "C" {
		/// Sends a message to the backend with no parameters.
		#[wasm_bindgen(js_name = invoke)]
		pub fn invoke_none(cmd: &str) -> Promise;

		/// Sends a message to the backend with parameters.
		#[wasm_bindgen(js_name = invoke)]
		pub fn invoke_parameters(cmd: &str, args: JsValue) -> Promise;
	}
}
