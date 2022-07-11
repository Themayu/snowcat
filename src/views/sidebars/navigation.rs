mod channels_list;
mod conversations_list;

use crate::App;
use crate::styles::{classname, NO_USER_SELECT};
use crate::styles::icon;
use crate::views::client::{ClientScreenState, ClientView};
use crate::views::sidebars::SIDEBAR;
use crate::views::sidebars::navigation::channels_list::channels_list;
use crate::views::sidebars::navigation::conversations_list::conversations_list;
use dominator::{Dom, EventOptions, class, clone, html, svg, pseudo};
use futures_signals::signal::{Signal, SignalExt, always};
use once_cell::sync::Lazy;
use snowcat_common::state::character::CharacterStatus;
use std::rc::Rc;
use std::sync::Arc;

static CURRENT_CHARACTER: Lazy<String> = Lazy::new(|| class! {
	.style("background-color", "var(--background-secondary)")
	.style("display", "grid")
	.style("grid-template-areas", "\"avatar top\" \"avatar name\" \"avatar options\"")
	.style("grid-template-columns", "50px 1fr")
	.style("grid-template-rows", "6px 21px 19px")
	.style("padding", "4px 10px 10px 10px")
});

static CURRENT_CHARACTER_AVATAR: Lazy<String> = Lazy::new(|| class! {
	.style("display", "block")
	.style("grid-area", "avatar")
	.style("height", "100%")
	.style("width", "100%")
});

static CURRENT_CHARACTER_NAME: Lazy<String> = Lazy::new(|| class! {
	.style("color", "var(--header-primary)")
	.style("font-size", "16px")
	.style("grid-area", "name")
	.style("margin", "0")
	.style("overflow-x", "hidden")
	.style("text-overflow", "ellipsis")
	.style("white-space", "nowrap")
});

static CURRENT_CHARACTER_OPTIONS: Lazy<String> = Lazy::new(|| class! {
	.style("color", "var(--ui-inactive)")
	.style("font-size", "12px")
	.style("grid-area", "options")
	.style("margin", "0")
});

static NAVIGATION_CONTAINER: Lazy<String> = Lazy::new(|| class! {
	.style("flex", "1 0 0")
	.style("min-height", "0")
	.style("overflow-y", "auto")
});

static NAVIGATION_FUNCTION_OPTIONS: Lazy<String> = Lazy::new(|| class! {
	.style("column-gap", "17.5px")
	.style("font-size", "20px")
	.style("padding", "15px")
});

static NAVIGATION_FUNCTION_OPTION: Lazy<String> = Lazy::new(|| class! {
	.style("padding-block", "5px")
});

static NAVIGATION_LIST_OPTIONS: Lazy<String> = Lazy::new(|| class! {
	.style("font-size", "13px")
	.style("padding", "10px")
});

static NAVIGATION_LIST_OPTION: Lazy<String> = Lazy::new(|| class! {
	.style("flex", "1 1 0")
	.style("padding-block", "8px")
	.style("text-align", "center")
});

static NAVIGATION_OPTION: Lazy<String> = Lazy::new(|| class! {
	.style("color", "var(--ui-inactive)")
	.style("cursor", "pointer")
	.style("flex", "1 1 0")
	.style("text-align", "center")

	.pseudo!(":hover", {
		.style("color", "var(--header-secondary)")
	})
});

static NAVIGATION_OPTION_ACTIVE: Lazy<String> = Lazy::new(|| class! {
	.style("color", "var(--header-secondary)")
});

static NAVIGATION_SWITCHER: Lazy<String> = Lazy::new(|| class! {
	.style("background-color", "var(--background-secondary)")
	.style("display", "flex")
});

static OPTION_TEXT: Lazy<String> = Lazy::new(|| class! {
	.style("cursor", "pointer")

	.pseudo!(":hover", {
		.style("color", "var(--ui-text)")
	})
});

static VIEW_SWITCH_BUTTON: Lazy<String> = Lazy::new(|| class! {
	.style("align-items", "center")
	.style("color", "var(--ui-inactive)")
	.style("column-gap", "10px")
	.style("cursor", "pointer")
	.style("display", "flex")
	.style("flex-flow", "row nowrap")
	.style("padding", "10px")

	.pseudo!(":hover", {
		.style("background-color", "var(--background-tertiary)")
		.style("color", "var(--ui-text)")
	})
});

static VIEW_SWITCH_BUTTON_ACTIVE: Lazy<String> = Lazy::new(|| class! {
	.style("color", "var(--ui-text)")
});

static VIEW_SWITCH_BUTTON_ICON: Lazy<String> = Lazy::new(|| class! {
	.style("--container-size", "40px")
	.style("--icon-size", "20pt")

	.style("align-items", "center")
	.style("background-color", "var(--background-primary)")
	.style("border-radius", "calc(var(--container-size) / 2)")
	.style("display", "flex")
	.style("flex", "0 0 auto")
	.style("font-size", "var(--icon-size)")
	.style("height", "var(--container-size)")
	.style("justify-content", "center")
	.style("width", "var(--container-size)")
});

static VIEW_SWITCH_BUTTON_ICON_SYMBOL: Lazy<String> = Lazy::new(|| class! {
	.style("--icon-size", "20px")
});

static VIEW_SWITCH_BUTTON_OPTIONS_BOX: Lazy<String> = Lazy::new(|| class! {
	.style("align-self", "stretch")
	.style("color", "var(--ui-muted)")
	.style("display", "flex")
	.style("flex", "0 0 0")
	.style("flex-flow", "column nowrap")
	.style("font-size", "14px")
	.style("justify-content", "space-between")
});

static VIEW_SWITCH_BUTTON_OPTION: Lazy<String> = Lazy::new(|| class! {
	.pseudo!(":hover", {
		.style("color", "var(--ui-text)")
	})
});

static VIEW_SWITCH_BUTTON_SUBTEXT: Lazy<String> = Lazy::new(|| class! {
	.style("color", "var(--ui-muted)")
	.style("display", "block")
	.style("font-size", "12px")
	.style("margin-top", "-2px")
});

static VIEW_SWITCH_BUTTON_TEXT_CONTAINER: Lazy<String> = Lazy::new(|| class! {
	.style("flex", "1 0 0")
	.style("min-width", "0")
});

static VIEW_SWITCH_BUTTON_TEXT: Lazy<String> = Lazy::new(|| class! {
	.style("display", "block")
	.style("font-size", "16px")
	.style("line-height", "20px")
	.style("overflow", "hidden")
	.style("text-overflow", "ellipsis")
	.style("white-space", "nowrap")
});

#[derive(Debug, Clone, PartialEq)]
pub enum NavigationView {
	Channels,
	CharacterSearch,
	ConversationHistory,
	MessageLogs,
	PrivateMessages,
	PublicChannelList,
	Settings,
}

impl NavigationView {
	pub fn next(&self) -> NavigationView {
		use NavigationView::*;

		match self {
			PrivateMessages => Channels,
			Channels => PublicChannelList,
			PublicChannelList => CharacterSearch,
			CharacterSearch => ConversationHistory,
			ConversationHistory => Settings,
			Settings => MessageLogs,
			MessageLogs => PrivateMessages,
		}
	}

	pub fn previous(&self) -> NavigationView {
		use NavigationView::*;

		match self {
			PrivateMessages => MessageLogs,
			Channels => PrivateMessages,
			PublicChannelList => Channels,
			CharacterSearch => PublicChannelList,
			ConversationHistory => CharacterSearch,
			Settings => ConversationHistory,
			MessageLogs => Settings,
		}
	}
}

pub fn render(app: Arc<App>, state: Rc<ClientScreenState>) -> Dom {
	html!("div", {
		.class(&*SIDEBAR)
		.class(&*NO_USER_SELECT)

		// Character display
		.child(html!("div", {
			.class(&*self::CURRENT_CHARACTER)
			
			.child(profile_avatar(
				always(String::from("https://cdn.pixabay.com/photo/2016/08/08/09/17/avatar-1577909_960_720.png")),
				always(CharacterStatus::Busy),
			))

			.child(html!("p", {
				.attr_signal("title", always("Sarah Blitz Garrison"))
				.class(&*self::CURRENT_CHARACTER_NAME)
				.text_signal(always("Sarah Blitz Garrison"))
			}))

			.child(html!("p", {
				.class(&*self::CURRENT_CHARACTER_OPTIONS)
				
				.child(html!("span", {
					.class(&*self::OPTION_TEXT)
					.text("Change Status")
				}))
				
				.text(" | ")

				.child(html!("span", {
					.class(&*self::OPTION_TEXT)
					.text("Log Out")
				}))
			}))
		}))
		// End character display

		// Channel list switcher
		.child(html!("div", {
			.class(&*self::NAVIGATION_SWITCHER)
			.class(&*self::NAVIGATION_LIST_OPTIONS)

			.child(html!("a", {
				.apply(mixins::navigation_list_option(
					&state,
					NavigationView::PrivateMessages,
					"Private Messages",
				))
			}))

			.child(html!("a", {
				.apply(mixins::navigation_list_option(
					&state,
					NavigationView::Channels,
					"Channels",
				))
			}))
		}))
		// End channel list switcher

		// Screen switcher
		.child_signal(state.navigation_view.signal_ref(clone!(app, state => move |view| match view {
			NavigationView::Channels => Some(channels_list(app.clone(), state.clone())),
			NavigationView::PrivateMessages => Some(conversations_list(app.clone(), state.clone())),
			view => todo!("view: {:?}", view),
		})))
		// End screen switcher

		// Client function switcher
		.child(html!("div", {
			.class(&*self::NAVIGATION_SWITCHER)
			.class(&*self::NAVIGATION_FUNCTION_OPTIONS)

			.child(html!("a", {
				.apply(mixins::navigation_function_option(
					&state,
					NavigationView::PublicChannelList,
					icon::BULLETED_LIST_GLYPH,
				))
			}))

			.child(html!("a", {
				.apply(mixins::navigation_function_option(
					&state,
					NavigationView::CharacterSearch, 
					icon::SEARCH_GLYPH,
				))
			}))

			.child(html!("a", {
				.apply(mixins::navigation_function_option(
					&state,
					NavigationView::ConversationHistory,
					icon::HISTORY_GLYPH,
				))
			}))

			.child(html!("a", {
				.apply(mixins::navigation_function_option(
					&state,
					NavigationView::Settings,
					icon::SETTINGS_GLYPH,
				))
			}))

			.child(html!("a", {
				.apply(mixins::navigation_function_option(
					&state,
					NavigationView::MessageLogs,
					icon::KNOWLEDGE_ARTICLE_GLYPH
				))
			}))
		}))
		// End client function switcher
	})
}

fn profile_avatar<Url, Status>(url: Url, status: Status) -> Dom
where Url: 'static + Signal<Item = String>,
      Status: 'static + Signal<Item = CharacterStatus> {
	svg!("svg", {
		.attr("aria-hidden", "true")
		.attr("viewbox", "0 0 50 46")
		.class(&*self::CURRENT_CHARACTER_AVATAR)

		.child(svg!("mask", {
			.attr("id", "svg-mask-avatar-status-round")
			.attr("height", "46")
			.attr("width", "45")

			.child(svg!("rect", {
				.attr("height", "40")
				.attr("width", "40")
				.attr("x", "0")
				.attr("y", "6")
				.attr("fill", "white")
			}))

			.child(svg!("circle", {
				.attr("r", "9")
				.attr("cx", "40")
				.attr("cy", "6")
				.attr("fill", "black")
			}))
		}))

		.child(svg!("foreignObject", {
			.attr("height", "40")
			.attr("mask", "url(#svg-mask-avatar-status-round)")
			.attr("width", "40")
			.attr("x", "0")
			.attr("y", "6")

			.child(html!("img", {
				.attr("alt", "")
				.attr("aria-hidden", "true")
				.attr_signal("src", url)
				.style("height", "100%")
				.style("width", "100%")
			}))
		}))

		.child(svg!("circle", {
			.attr("r", "6")
			.attr("cx", "40")
			.attr("cy", "6")

			.attr_signal("fill", status.map(|status| match status {
				CharacterStatus::Looking => "var(--status-looking)",
				CharacterStatus::Online => "var(--status-online)",
				CharacterStatus::Away => "var(--status-away)",
				CharacterStatus::Idle => "var(--status-idle)",
				CharacterStatus::Busy => "var(--status-busy)",
				CharacterStatus::DoNotDisturb => "var(--status-dnd)",
				CharacterStatus::Offline => "var(--status-offline)",
			}))
		}))
	})
}

pub(super) fn view_console_button(state: &Rc<ClientScreenState>) -> Dom {
	fn is_active_view_signal<View>(view: View) -> impl Signal<Item = bool>
	where View: Signal<Item = ClientView>, {
		view.map(|view| matches!(view, ClientView::Console))
	}

	html!("div", {
		.attr("title", "Console")
		.class(&*VIEW_SWITCH_BUTTON)
		.class_signal(&*VIEW_SWITCH_BUTTON_ACTIVE, is_active_view_signal(state.client_view.signal_cloned()))

		.child(html!("div", {
			.class(&*VIEW_SWITCH_BUTTON_ICON)
			.class(&*VIEW_SWITCH_BUTTON_ICON_SYMBOL)
			.class(&*classname::ICON)
			.text(&*icon::CONSOLE_VIEW_GLYPH)
		}))

		.child(html!("div", {
			.class(&*VIEW_SWITCH_BUTTON_TEXT_CONTAINER)
			
			.child(html!("span", {
				.class(&*VIEW_SWITCH_BUTTON_TEXT)
				.text("Console")
			}))

			.child(html!("span", {
				.class(&*VIEW_SWITCH_BUTTON_SUBTEXT)
			}))
		}))

		.event_with_options(&EventOptions::bubbles(), interactions::set_console_view_on_click(state))
	})
}

mod interactions {
	use crate::views::client::{ClientScreenState, ClientView};
	use crate::views::sidebars::NavigationView;
	use dominator::clone;
	use dominator::events::Click;
	use std::rc::Rc;

	pub(super) fn set_navigation_menu_on_click(state: &Rc<ClientScreenState>, value: NavigationView)
	-> impl FnMut(Click) {
		clone!(state, value => move |_: Click| {
			state.navigation_view.set_neq(value.clone());
		})
	}

	pub(super) fn set_console_view_on_click(
		state: &Rc<ClientScreenState>,
	) -> impl FnMut(Click) {
		clone!(state => move |_: Click| {
			state.client_view.set_if(
				ClientView::Console,
				|old, new| match (old, new) {
					(ClientView::Console, ClientView::Console) => false,
					_ => true,
				},
			);
		})
	}
}

mod mixins {
	use super::interactions;
	use crate::styles::classname;
    use crate::views::client::ClientScreenState;
    use crate::views::sidebars::NavigationView;
	use dominator::{DomBuilder, apply_methods, clone};
    use std::rc::Rc;
	use web_sys::HtmlElement;

	pub fn navigation_function_option(state: &Rc<ClientScreenState>, option: NavigationView, text: impl AsRef<str>)
	-> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
		clone!(state => move |dom| {
			apply_methods!(dom, {
				.apply(navigation_option_shared(&state, option, text))
				.class(&*super::NAVIGATION_FUNCTION_OPTION)
				.class(&*classname::ICON)
			})
		})
	}

	pub fn navigation_list_option(state: &Rc<ClientScreenState>, option: NavigationView, text: impl AsRef<str>)
	-> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
		clone!(state => move |dom| {
			apply_methods!(dom, {
				.apply(navigation_option_shared(&state, option, text))
				.class(&*super::NAVIGATION_LIST_OPTION)
			})
		})
	}

	fn navigation_option_shared(state: &Rc<ClientScreenState>, option: NavigationView, text: impl AsRef<str>)
	-> impl FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> {
		clone!(state => move |dom| {
			apply_methods!(dom, {
				.class(&*super::NAVIGATION_OPTION)
				.class_signal(&*super::NAVIGATION_OPTION_ACTIVE, state.navigation_view.signal_ref(clone!(option => move |view| *view == option)))
				.event(interactions::set_navigation_menu_on_click(&state, option.clone()))
				.text(text.as_ref())
			})
		})
	}
}
