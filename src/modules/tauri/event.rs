pub use crate::modules::tauri::event::event_sys::Event as EventSys;
use serde::Deserialize;
use wasm_bindgen::prelude::*;

pub type EventCallback = Closure<dyn Fn(EventSys)>;

pub struct Event<T> {
	inner: EventSys,
	serialized: Option<T>,
}

impl<T> Event<T>
where T: for<'de> Deserialize<'de> {
	pub fn new(inner: event_sys::Event) -> Self {
		Event { inner, serialized: None }
	}

	pub fn payload(&mut self) -> Option<&T> {
		let inner = self.inner.payload();

		if inner.is_undefined() | inner.is_null() {
			return None;
		}
		
		if matches!(self.serialized, None) {
			self.serialized = Some(inner.into_serde().unwrap_throw());
		}

		self.serialized.as_ref()
	}
}

mod event_sys {
	use wasm_bindgen::prelude::*;

	#[wasm_bindgen]
	extern "C" {
		#[derive(Debug, Clone)]
		pub type Event;
		
		#[wasm_bindgen(method, getter)]
		pub fn payload(this: &Event) -> JsValue;
	}
}
