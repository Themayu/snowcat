use crate::App;
use crate::styles::{NO_USER_SELECT, classname, icon};
use dominator::{Dom, clone, html};
use dominator::events::Click;
use std::sync::Arc;
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::spawn_local;

pub struct Titlebar;
impl Titlebar {
	pub fn render(app: Arc<App>) -> Dom {
		// const APP_ICON: &'static str = "app-icon";
		const BAR: &'static str = "titlebar";
		const BUTTON: &'static str = "button";
		const BUTTON_CLOSE: &'static str = "close";
		const TITLE: &'static str = "title";

		html!("div", {
			.attr("aria-hidden", "true")
			.attr("data-tauri-drag-region", "true")
			.class([BAR, &*NO_USER_SELECT])

			.children(&mut [
				// html!("img", {
				// 	.class(APP_ICON)
				// 	.attr("src", "https://static-cdn.jtvnw.net/jtv_user_pictures/bbb1b59c-e112-4d42-b666-8749c1ed297b-profile_image-70x70.png")
				// }),

				html!("p", {
					.class(TITLE)
					.text("Snowcat")
				}),
				
				html!("div", {
					.class(BUTTON)
					.event(clone!(app => move |_: Click| {
						spawn_local(clone!(app => async move {
							app.window.minimize().await.unwrap_throw();
						}))
					}))
		
					.child(
						html!("a", {
							.class(classname::ICON)
							.text(icon::MINIMISE_GLYPH)
						})
					)
				}),

				html!("div", {
					.class(BUTTON)
					.event(clone!(app => move |_: Click| {
						spawn_local(clone!(app => async move {
							app.window.toggle_maximize().await.unwrap_throw();
						}))
					}))

					.child(
						html!("a", {
							.class(classname::ICON)
							.text(icon::MAXIMISE_GLYPH)
						})
					)
				}),

				html!("div", {
					.class([BUTTON, BUTTON_CLOSE])
					.event(clone!(app => move |_: Click| {
						spawn_local(clone!(app => async move {
							app.window.close().await.unwrap_throw();
						}))
					}))

					.child(
						html!("a", {
							.class(classname::ICON)
							.text(icon::CLOSE_GLYPH)
						})
					)
				})
			])
		})
	}
}
