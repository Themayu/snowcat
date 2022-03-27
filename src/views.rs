pub mod client;
pub mod login;
pub mod overlays;
pub mod sidebars;

use crate::App;
use crate::styles::classname;
use crate::views::client::ClientScreenState;
use crate::views::sidebars::SIDEBAR_CONTAINER;
use dominator::{Dom, class, clone, html};
use once_cell::sync::Lazy;
use std::rc::Rc;
use std::sync::Arc;

static BODY: Lazy<String> = Lazy::new(|| class! {
	.style("contain", "paint")
	.style("display", "flex")
	.style("flex", "0 1 auto")
	.style("height", "100%")
	.style("min-height", "0")
});

#[derive(Debug, Clone)]
pub enum Screen {
	Client(Rc<ClientScreenState>),
	Login(Rc<LoginScreenState>),
}

#[derive(Debug, Clone)]
pub struct LoginScreenState {
	view: LoginView,
}

#[derive(Debug, Clone)]
pub enum LoginView {
	CharacterList,
	LoginForm,
}

pub fn build_client_screen(app: Arc<App>, state: Rc<ClientScreenState>) -> Dom {
	html!("div", {
		.class(classname::BODY)
		.class(&*self::BODY)

		.child(html!("div", {
			.class(&*classname::COLLAPSIBLE)
			.class(&*SIDEBAR_CONTAINER)

			.class_signal(&*classname::COLLAPSED, clone!(state => state.navigation_view_collapsed.signal()))
			.class_signal(&*classname::EXPANDED, clone!(state => state.navigation_view_expanded.signal()))

			.child(sidebars::render_navigation(app.clone(), state.clone()))
		}))

		.child_signal(state.client_view.signal_ref(
			clone!(app, state => move |view| Some(client::render(app.clone(), state.clone(), view)))
		))

		.child(html!("div", {
			.class(&*classname::COLLAPSIBLE)
			.class(&*SIDEBAR_CONTAINER)
			
			.class_signal(&*classname::COLLAPSED, clone!(state => state.actions_view_collapsed.signal()))
			.class_signal(&*classname::EXPANDED, clone!(state => state.actions_view_expanded.signal()))
		}))
	})
}

pub fn build_login_screen(app: Arc<App>, state: Rc<LoginScreenState>) -> Dom {
	login::login_view(app, state)
}

pub mod mock {
	use super::Screen;
	use crate::state::{self, PublicChannel, SystemMessage};
	use crate::views::client::{ClientScreenState, ClientView};
	use crate::views::sidebars::NavigationView;
	use chrono::{ParseResult, DateTime, Utc};
	use futures_signals::signal::Mutable;
	use futures_signals::signal_vec::MutableVec;
	use std::rc::Rc;

	pub fn get_client_screen() -> Screen {
		let channel_models: Vec<_> = state::mock::get_channels().into_iter().map(|channel| {
			PublicChannel::attach_view_state(channel)
		}).collect();
		
		let system_message_models: Vec<_> = state::mock::get_notifications().into_iter().map(|notification| {
			Rc::new(SystemMessage::new(notification, parse_datetime("2022-01-21 20:31:00 +00:00").unwrap()))
		}).collect();

		let view = {
			let channel = &channel_models[1];
			let view_state = channel.view_state();

			ClientView::PublicChannel(view_state.clone())
		};

		let navigation_view = NavigationView::Channels;

		let state = ClientScreenState {
			navigation_view: Rc::new(Mutable::new(navigation_view)),
			client_view: Rc::new(Mutable::new(view)),

			open_channels: Rc::new(MutableVec::new_with_values(channel_models)),
			open_conversations: Rc::new(MutableVec::new()),
			system_messages: Rc::new(MutableVec::new_with_values(system_message_models)),

			actions_view_collapsed: Mutable::new(false),
			actions_view_expanded: Mutable::new(false),
			navigation_view_collapsed: Mutable::new(false),
			navigation_view_expanded: Mutable::new(false),
		};

		Screen::Client(Rc::new(state))
	}

	fn parse_datetime(datetime: &str) -> ParseResult<DateTime<Utc>> {
		DateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S %z")
			.map(|datetime| datetime.with_timezone(&Utc))
	}
}
