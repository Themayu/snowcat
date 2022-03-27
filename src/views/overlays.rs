pub mod menus;
pub mod notifications;

// use dominator::{DomHandle, Dom, append_dom, class};
// use once_cell::sync::Lazy;
// use web_sys::HtmlElement;

// static OVERLAY_HOST: Lazy<String> = Lazy::new(|| class! {
// 	.style("background-color", "transparent")
// 	.style("position", "absolute")
// 	.style("bottom", "0px")
// 	.style("left", "0px")
// 	.style("right", "0px")
// 	.style("top", "0px")
// });

// #[derive(Debug)]
// pub struct OverlayHost {
// 	host: HtmlElement,

// 	pub bottom: i32,
// 	pub left: i32,
// 	pub right: i32,
// 	pub top: i32,

// 	pub x: i32,
// 	pub y: i32,
// }

// impl OverlayHost {
// 	pub fn new<'dom>(host: &'dom HtmlElement) -> OverlayHost {
// 		let host = host.clone();
// 		let bounding_box = host.get_bounding_client_rect();

// 		OverlayHost {
// 			host,

// 			bottom: bounding_box.bottom() as i32,
// 			left: bounding_box.left() as i32,
// 			right: bounding_box.right() as i32,
// 			top: bounding_box.top() as i32,

// 			x: bounding_box.x() as i32,
// 			y: bounding_box.y() as i32,
// 		}
// 	}

// 	pub fn show(&self, overlay: Dom) -> DomHandle {
// 		append_dom(&self.host, overlay)
// 	}
// }
