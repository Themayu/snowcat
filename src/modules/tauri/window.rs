use crate::modules::tauri::event::EventCallback;
use std::future::Future;
use js_sys::Function;
use serde::Serialize;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub struct WebviewWindow {
	inner: window_sys::WebviewWindow,
}

impl WebviewWindow {
	/// Closes the window.
	pub async fn close(&self) -> Result<(), JsValue> {
		let promise = self.inner.close();
		JsFuture::from(promise).await?;

		Ok(())
	}

	pub async fn emit<T>(&self, event: &str, payload: Option<&T>) -> Result<(), JsValue>
	where T: Serialize {
		let promise = if let Some(payload) = payload {
			let payload = serde_json::to_string(&payload).unwrap_throw();
			self.inner.emit_payload(event, &payload)
		} else {
			self.inner.emit_none(event)
		};

		JsFuture::from(promise).await?;

		Ok(())
	}

	/// Gets the window's current decorated state.
	pub async fn is_decorated(&self) -> Result<bool, JsValue> {
		let promise = self.inner.is_decorated();
		let future = JsFuture::from(promise);

		Ok(future.await?.as_bool().unwrap_throw())
	}

	/// Listen to an event emitted by the backend that is tied to the webview
	/// window.
	pub async fn listen(&self, event: &str, handler: &EventCallback) -> Result<Function, JsValue> {
		let promise = self.inner.listen(event, handler.as_ref().dyn_ref().unwrap_throw());
		let future = JsFuture::from(promise);
		
		future.await?.dyn_into::<Function>()
	}

	/// Minimizes the window.
	pub async fn minimize(&self) -> Result<(), JsValue> {
		let promise = self.inner.minimize();
		JsFuture::from(promise).await?;

		Ok(())
	}

	/// Whether the window should have borders and bars.
	pub async fn set_decorations(&self, decorations: bool) -> Result<(), JsValue> {
		let promise = self.inner.set_decorations(decorations);
		JsFuture::from(promise).await?;

		Ok(())
	}

	/// Toggles the window maximized state.
	pub async fn toggle_maximize(&self) -> Result<(), JsValue> {
		let promise = self.inner.toggle_maximize();
		JsFuture::from(promise).await?;

		Ok(())
	}
}

impl core::fmt::Debug for WebviewWindow {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "WebviewWindow")
	}
}

pub async fn get_current() ->  Result<WebviewWindow, JsValue> {
	let value = window_sys::get_current();
	Ok(WebviewWindow { inner: value })
}

mod window_sys {
	use js_sys::{Function, Promise};
	use wasm_bindgen::prelude::*;
	
	#[wasm_bindgen]
	extern "C" {
		// #[derive(Debug)]
		pub type WebviewWindow;

		// /// Centers the window
		// #[wasm_bindgen(method)]
		// pub fn center(this: &WebviewWindow) -> Promise;

		/// Closes the window
		#[wasm_bindgen(method)]
		pub fn close(this: &WebviewWindow) -> Promise;

		/// Emits an event to the backend, tied to the webview window, with no
		/// payload.
		#[wasm_bindgen(method, js_name = emit)]
		pub fn emit_none(this: &WebviewWindow, event: &str) -> Promise;

		/// Emits an event to thhe backend, tied to the webview window, with a
		/// given string payload.
		#[wasm_bindgen(method, js_name = emit)]
		pub fn emit_payload(this: &WebviewWindow, event: &str, payload: &str) -> Promise;

		// Implementation details: returl = Promise<null | WebviewWindow>
		// /// Gets the WebviewWindow for the webview associated with the given
		// /// label.
		// #[wasm_bindgen(method, js_name = getByLabel)]
		// pub fn get_by_label(this: &WebviewWindow, title: &str) -> Promise;

		// /// Sets the window visibility to false.
		// #[wasm_bindgen(method)]
		// pub fn hide(this: &WebviewWindow) -> Promise;

		// /// The position of the top-left hand corner of the window's client area
		// /// relative to the top-left hand corner of the desktop.
		// #[wasm_bindgen(method, js_name = innerPosition)]
		// pub fn inner_position(this: &WebviewWindow) -> Promise;

		// /// The physical size of the window's client area. The client area is the
		// /// content of the window, excluding the title bar and borders.
		// #[wasm_bindgen(method, js_name = innerSize)]
		// pub fn inner_size(this: &WebviewWindow) -> Promise;

		/// Gets the window's current decorated state.
		#[wasm_bindgen(method, js_name = isDecorated)]
		pub fn is_decorated(this: &WebviewWindow) -> Promise;

		// /// Gets the window's current fullscreen state.
		// #[wasm_bindgen(method, js_name = isFullscreen)]
		// pub fn is_fullscreen(this: &WebviewWindow) -> Promise;

		// /// Gets the window's current maximized state.
		// #[wasm_bindgen(method, js_name = isMaximized)]
		// pub fn is_maximized(this: &WebviewWindow) -> Promise;

		// /// Gets the window's current resizable state.
		// #[wasm_bindgen(method, js_name = isResizable)]
		// pub fn is_resizable(this: &WebviewWindow) -> Promise;

		// /// Gets the window's current visible state.
		// #[wasm_bindgen(method, js_name = isVisible)]
		// pub fn is_visible(this: &WebviewWindow) -> Promise;

		/// Listen to an event emitted by the backend that is tied to the webview
		/// window.
		#[wasm_bindgen(method)]
		pub fn listen(this: &WebviewWindow, event: &str, handler: &Function) -> Promise;

		// /// Maximizes the window.
		// #[wasm_bindgen(method)]
		// pub fn maximize(this: &WebviewWindow) -> Promise;

		/// Minimizes the window.
		#[wasm_bindgen(method)]
		pub fn minimize(this: &WebviewWindow) -> Promise;

		// /// Listen to an one-off event emitted by the backend that is tied to the
		// /// webview window.
		// #[wasm_bindgen(method)]
		// pub fn once(this: &WebviewWindow, event: &str, handler: &Function) -> Promise;

		// /// The position of the top-left hand corner of the window relative to the
		// /// top-left hand corner of the desktop.
		// #[wasm_bindgen(method, js_name = outerPosition)]
		// pub fn outer_position(this: &WebviewWindow) -> Promise;

		// /// The physical size of the entire window. These dimensions include the
		// /// title bar and borders. If you don't want that (and you usually don't),
		// /// use [`inner_size`] instead.
		// #[wasm_bindgen(method, js_name = outerSize)]
		// pub fn outer_size(this: &WebviewWindow) -> Promise;

		// /// Requests user attention to the window, this has no effect if the
		// /// application is already focused. How requesting for user attention
		// /// manifests is platform dependent, see `UserAttentionType` for
		// /// details.
		// /// 
		// /// Providing `null` will unset the request for user attention.
		// /// Unsetting the request for user attention might not be done
		// /// automatically by the WM when the window receives input.
		// /// 
		// /// # Platform-specific
		// /// 
		// /// - macOS: `null` has no effect.
		// #[wasm_bindgen(method, js_name = requestUserAttention)]
		// pub fn request_user_attention(this: &WebviewWindow, request_type: Option<u32>) -> Promise;

		// /// The scale factor can be used to map physical pixels to logical
		// /// pixels.
		// #[wasm_bindgen(method, js_name = scaleFactor)]
		// pub fn scale_factor(this: &WebviewWindow) -> Promise;

		// /// Whether the window should always be on top of other windows.
		// #[wasm_bindgen(method, js_name = setAlwaysOnTop)]
		// pub fn set_always_on_top(this: &WebviewWindow, always_on_top: bool) -> Promise;

		/// Whether the window should have borders and bars.
		#[wasm_bindgen(method, js_name = setDecorations)]
		pub fn set_decorations(this: &WebviewWindow, decorations: bool) -> Promise;

		// /// Bring the window to front and focus.
		// #[wasm_bindgen(method, js_name = setFocus)]
		// pub fn set_focus(this: &WebviewWindow) -> Promise;

		// /// Sets the window fullscreen state.
		// #[wasm_bindgen(method, js_name = setFullscreen)]
		// pub fn set_fullscreen(this: &WebviewWindow, fullscreen: bool) -> Promise;

		// /// Sets the window icon to a binary buffer.
		// #[wasm_bindgen(method, js_name = setIcon)]
		// pub fn set_icon_bytes(this: &WebviewWindow, bytes: &[u8]) -> Promise;

		// /// Sets the path to the window icon.
		// #[wasm_bindgen(method, js_name = setIcon)]
		// pub fn set_icon_path(this: &WebviewWindow, path: &str) -> Promise;

		// Implementation details: size = PhysicalSize | LogicalSize | undefined
		// /// Sets the window max size. If the size argument is not provided, the
		// /// max size is unset.
		// #[wasm_bindgen(method, js_name = setMaxSize)]
		// pub fn set_max_size(this: &WebviewWindow, size) -> Promise

		// Implementation details: size = PhysicalSize | LogicalSize | undefined
		// /// Sets the window min size. If the size argument is not provided, the
		// /// min size is unset.
		// #[wasm_bindgen(method, js_name = setMinSize)]
		// pub fn set_min_size(this: &WebviewWindow, size) -> Promise;

		// Implementation details: position = PhysicalPosition | LogicalPosition
		// /// Sets the window position.
		// #[wasm_bindgen(method, js_name = setPosition)]
		// pub fn set_position(this: &WebviewWindow, position) -> Promise;

		// /// Updates the window resizable flag.
		// #[wasm_bindgen(method, js_name = setResizable)]
		// pub fn set_resizable(this: &WebviewWindow, resizable: bool) -> Promise;

		// Implementation details: size = PhysicalSize | LogicalSize
		// /// Resizes the window.
		// #[wasm_bindgen(method, js_name = setSize)]
		// pub fn set_size(this: &WebviewWindow, size) -> Promise;

		// /// Whether to show the window icon in the task bar or not.
		// #[wasm_bindgen(method, js_name = setSkipTaskbar)]
		// pub fn set_skip_taskbar(this: &WebviewWindow, skip: bool) -> Promise;

		// /// Sets the window title.
		// #[wasm_bindgen(method, js_name = setTitle)]
		// pub fn set_title(this: &WebviewWindow, title: &str) -> Promise;

		// /// Sets the window visibility to true.
		// #[wasm_bindgen(method)]
		// pub fn show(this: &WebviewWindow) -> Promise;

		// /// Starts dragging the window.
		// #[wasm_bindgen(method, js_name = startDragging)]
		// pub fn start_dragging(this: &WebviewWindow) -> Promise;

		/// Toggles the window maximize state.
		#[wasm_bindgen(method, js_name = toggleMaximize)]
		pub fn toggle_maximize(this: &WebviewWindow) -> Promise;

		// /// Unmaximizes the window
		// #[wasm_bindgen(method)]
		// pub fn unmaximize(this: &WebviewWindow) -> Promise;

		// /// Unminimizes the window
		// #[wasm_bindgen(method)]
		// pub fn unminimize(this: &WebviewWindow) -> Promise;
	}

	#[wasm_bindgen(module = "@tauri-apps/api/window")]
	extern "C" {
		#[wasm_bindgen(js_name = getCurrent)]
		pub fn get_current() -> WebviewWindow;
	}
}
