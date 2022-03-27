use std::ops::Deref;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use wasm_bindgen::{prelude::*, JsCast};

pub struct ResizeObserver {
	id: usize,
	observer: web_sys::ResizeObserver,
	_closure: Closure<dyn Fn(js_sys::Array, web_sys::ResizeObserver)>,
}

impl ResizeObserver {
	pub fn new(callback: impl Fn(js_sys::Array, web_sys::ResizeObserver) + 'static) -> Result<Rc<ResizeObserver>, JsValue> {
		static ID: AtomicUsize = AtomicUsize::new(0);

		let id = ID.fetch_add(1, Ordering::SeqCst);
		let closure = Closure::wrap(Box::new(callback) as Box<dyn Fn(js_sys::Array, web_sys::ResizeObserver) + 'static>);
		let observer = web_sys::ResizeObserver::new(&closure.as_ref().unchecked_ref())?;

		log::trace!("created resize observer {}", id);

		Ok(Rc::new(ResizeObserver {
			id,
			observer,
			_closure: closure,
		}))
	}
}

impl Deref for ResizeObserver {
	type Target = web_sys::ResizeObserver;

	fn deref(&self) -> &Self::Target {
		&self.observer
	}
}

impl Drop for ResizeObserver {
	fn drop(&mut self) {
		log::trace!("resize observer {} went out of scope, dropping", self.id);
	}
}
