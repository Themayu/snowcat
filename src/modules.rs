pub mod tauri;
pub mod web;

use js_sys::Promise;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt};
use wasm_bindgen_futures::JsFuture;

async fn to_future<T>(promise: Promise) -> Result<T, JsValue> where T: JsCast {
	let value = JsFuture::from(promise).await?;
	Ok(value.dyn_into::<T>().unwrap_throw())
}

// async fn to_future_void(promise: Promise) -> Result<(), JsValue> {
// 	let value = JsFuture::from(promise).await?;
// 	Ok(())
// }
