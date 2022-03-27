#![feature(arc_new_cyclic)]
#![feature(bool_to_option)]
#![warn(rust_2018_idioms)]

// FIXME: Move class declarations to CSS, as apart from the root element no
//        class declaration uses `style_signal`.

mod commands;
mod components;
mod io;
mod modules;
mod state;
mod styles;
mod views;

use crate::commands::GetSettingsCommand;
// use crate::components::Titlebar;
use crate::modules::tauri::{event, window};
use crate::state::AppState;
use crate::state::settings::AppSettings;
use crate::styles::classname;
use crate::views::overlays::menus::ContextMenuContainer;
use crate::views::{
	Screen,
	build_client_screen,
	build_login_screen,
	mock::get_client_screen,
};
use dominator::{Dom, class, clone, html, stylesheet};
use futures_signals::signal::{Mutable, SignalExt};
use js_sys::Function;
use once_cell::sync::Lazy;
use snowcat_common::theme::Theme;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::prelude::*;

const LOGGING_PREFIX: &str = "snowcat_ui";

#[derive(Debug)]
pub struct App {
	data: Rc<AppState>,
	overlays: OverlaySet,
	screen: Arc<Mutable<Screen>>,
	theme: Mutable<Theme>,
	window: Arc<window::WebviewWindow>,
}

impl App {
	async fn new() -> Result<Arc<Self>, JsValue> {
		let window = Arc::new(window::get_current().await?);
		let screen = Arc::new(Mutable::new(get_client_screen()));

		let app_settings = GetSettingsCommand.invoke().await.unwrap_throw();

		let app = Self {
			data: AppState::new(app_settings),

			overlays: OverlaySet {
				context_menu: ContextMenuContainer::new(),
			},

			theme: Mutable::default(),
			screen,
			window,
		};

		log::debug!("initialising app with state {:#?}", &app.data);
		Ok(app.into())
	}

	async fn setup(app: Arc<Self>) -> Result<(), JsValue> {
		clone!(app => app.setup_listeners()).await?;
		clone!(app => app.setup_styles());

		Ok(())
	}

	async fn setup_listeners(self: Arc<Self>) -> Result<Vec<Function>, JsValue> {
		let window = self.window.clone();
		let theme_set_handler = move |value| {
			let mut value = event::Event::<Theme>::new(value);

			self.theme.set(value.payload().unwrap().clone());
		};

		let theme_set_handler = Box::new(theme_set_handler) as Box<dyn Fn(event::EventSys)>;
		let theme_set_handler = Closure::wrap(theme_set_handler);

		let theme_set_unlisten = window
			.listen("theme-set", &theme_set_handler).await?;

		Ok(vec![theme_set_unlisten])
	}

	fn setup_styles(&self) {
		stylesheet!(":root", {
			.style_signal("--background-base", self.theme.signal_ref(|t| t.background.base.to_rgba()))
			.style_signal("--background-primary", self.theme.signal_ref(|t| t.background.primary.to_rgba()))
			.style_signal("--background-secondary", self.theme.signal_ref(|t| t.background.secondary.to_rgba()))
			.style_signal("--background-secondary-alt", self.theme.signal_ref(|t| t.background.secondary_alt.to_rgba()))
			.style_signal("--background-tertiary", self.theme.signal_ref(|t| t.background.tertiary.to_rgba()))
			.style_signal("--background-floating", self.theme.signal_ref(|t| t.background.floating.to_rgba()))

			.style_signal("--header-primary", self.theme.signal_ref(|t| t.header.primary.to_rgba()))
			.style_signal("--header-secondary", self.theme.signal_ref(|t| t.header.secondary.to_rgba()))

			.style_signal("--ui-text", self.theme.signal_ref(|t| t.ui.text.to_rgba()))
			.style_signal("--ui-inactive", self.theme.signal_ref(|t| t.ui.inactive.to_rgba()))
			.style_signal("--ui-muted", self.theme.signal_ref(|t| t.ui.muted.to_rgba()))
			.style_signal("--ui-positive", self.theme.signal_ref(|t| t.ui.positive.to_rgba()))
			.style_signal("--ui-warning", self.theme.signal_ref(|t| t.ui.warning.to_rgba()))
			.style_signal("--ui-danger", self.theme.signal_ref(|t| t.ui.danger.to_rgba()))
			.style_signal("--ui-info", self.theme.signal_ref(|t| t.ui.info.to_rgba()))
			.style_signal("--ui-selection", self.theme.signal_ref(|t| t.ui.selection.to_rgba()))

			.style_signal("--chat-blue", self.theme.signal_ref(|t| t.chat.blue.to_rgba()))
			.style_signal("--chat-brown", self.theme.signal_ref(|t| t.chat.brown.to_rgba()))
			.style_signal("--chat-cyan", self.theme.signal_ref(|t| t.chat.cyan.to_rgba()))
			.style_signal("--chat-green", self.theme.signal_ref(|t| t.chat.green.to_rgba()))
			.style_signal("--chat-orange", self.theme.signal_ref(|t| t.chat.orange.to_rgba()))
			.style_signal("--chat-pink", self.theme.signal_ref(|t| t.chat.pink.to_rgba()))
			.style_signal("--chat-purple", self.theme.signal_ref(|t| t.chat.purple.to_rgba()))
			.style_signal("--chat-red", self.theme.signal_ref(|t| t.chat.red.to_rgba()))
			.style_signal("--chat-yellow", self.theme.signal_ref(|t| t.chat.yellow.to_rgba()))
			.style_signal("--chat-black", self.theme.signal_ref(|t| t.chat.black.to_rgba()))
			.style_signal("--chat-grey", self.theme.signal_ref(|t| t.chat.grey.to_rgba()))
			.style_signal("--chat-white", self.theme.signal_ref(|t| t.chat.white.to_rgba()))

			.style_signal("--sex-cunt-boy", self.theme.signal_ref(|t| t.sex.cunt_boy.to_rgba()))
			.style_signal("--sex-female", self.theme.signal_ref(|t| t.sex.female.to_rgba()))
			.style_signal("--sex-hermaphrodite", self.theme.signal_ref(|t| t.sex.hermaphrodite.to_rgba()))
			.style_signal("--sex-male", self.theme.signal_ref(|t| t.sex.male.to_rgba()))
			.style_signal("--sex-male-herm", self.theme.signal_ref(|t| t.sex.male_herm.to_rgba()))
			.style_signal("--sex-shemale", self.theme.signal_ref(|t| t.sex.shemale.to_rgba()))
			.style_signal("--sex-transgender", self.theme.signal_ref(|t| t.sex.transgender.to_rgba()))
			.style_signal("--sex-none-set", self.theme.signal_ref(|t| t.sex.none_set.to_rgba()))

			.style_signal("--status-looking", self.theme.signal_ref(|t| t.status.looking.to_rgba()))
			.style_signal("--status-online", self.theme.signal_ref(|t| t.status.online.to_rgba()))
			.style_signal("--status-away", self.theme.signal_ref(|t| t.status.away.to_rgba()))
			.style_signal("--status-idle", self.theme.signal_ref(|t| t.status.idle.to_rgba()))
			.style_signal("--status-busy", self.theme.signal_ref(|t| t.status.busy.to_rgba()))
			.style_signal("--status-dnd", self.theme.signal_ref(|t| t.status.dnd.to_rgba()))
			.style_signal("--status-offline", self.theme.signal_ref(|t| t.status.offline.to_rgba()))
		});

		stylesheet!("*", {
			.style(["-webkit-box-sizing", "-moz-box-sizing", "box-sizing"],
				"border-box")
			.style("font-family", r#"-apple-system, "Segoe UI""#)
		});
	}

	async fn render(app: Arc<Self>) -> Result<Dom, JsValue> {
		static APP_CONTAINER: Lazy<String> = Lazy::new(|| class! {
			.style("display", "flex")
			.style("flex-flow", "column nowrap")
			.style("height", "100%")
		});

		let dom = html!("div", {
			.class(classname::CONTAINER)
			.class(&*APP_CONTAINER)
			// .class_signal(classname::FRAMELESS, always(true))
			// .child(clone!(app => Titlebar::render(app)))

			.child_signal(app.screen.signal_cloned().map(clone!(app => move |screen| match screen {
				Screen::Client(state) => clone!(app => Some(build_client_screen(app, state))),
				Screen::Login(state) => clone!(app => Some(build_login_screen(app, state))),
			})))

			.child(app.overlays.context_menu.render())
		});

		Ok(dom)
	}

	// fn render(app: Arc<Self>) -> Dom {
	//     // Define CSS styles
	//     static CLASS: Lazy<String> = Lazy::new(|| class! {
	//         .style("font-size", "20px")
	//         .style("color", "hsl(110, 70%, 70%)")
	//     });

	//     // Create the DOM nodes
	//     html!("div", {
	//         .class(&*CLASS)

	//         .text_signal(app.message.signal_cloned().map(|message| {
	//             format!("Message: {}", message)
	//         }))

	//         .event(clone!(app => move |_: events::Click| {
	//             app.message.set_neq("Goodbye!".to_string());
	//         }))
	//     })
	// }
}

#[derive(Debug)]
pub struct OverlaySet {
	pub context_menu: Rc<ContextMenuContainer>,
}

#[wasm_bindgen(start)]
pub async fn main_js() -> Result<(), JsValue> {
	#[cfg(debug_assertions)]
	console_error_panic_hook::set_once();

	wasm_logger::init(wasm_logger::Config::new(log::Level::Trace).module_prefix(self::LOGGING_PREFIX));

	let app = App::new().await?;

	App::setup(app.clone()).await?;
	dominator::append_dom(&dominator::body(), App::render(app).await?);

	Ok(())
}
