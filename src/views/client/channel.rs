use crate::App;
use crate::modules::web::ResizeObserver;
use crate::state::PublicChannel;
use crate::styles::{NO_USER_SELECT, classname, component, icon};
use crate::views::client::{
	VIEW_DESCRIPTION_MINIMISED, VIEW_HEADER, VIEW_TITLE, VIEW_TITLE_BOX, VIEW_TITLE_SUBTEXT, ClientScreenState,
	group_by_date,
};
use dominator::{Dom, class, clone, html, with_node, dom_builder};
use dominator::events::Click;
use futures_signals::map_ref;
use futures_signals::signal::{Mutable, Signal, SignalExt, not};
use futures_signals::signal_vec::{SignalVecExt, MutableSignalVec, SignalVec};
use once_cell::sync::Lazy;
use snowcat_common::state::{MessageType, ChannelMessage};
use std::cell::RefCell;
use std::mem;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use web_sys::HtmlTextAreaElement;

static MESSAGE_AREA: Lazy<String> = Lazy::new(|| class! {
	.style("background-color", "var(--background-primary)")
	.style("padding", "10px")
});

static MESSAGE_BOX: Lazy<String> = Lazy::new(|| class! {
	.style("display", "flex")
	.style("flex-flow", "row nowrap")
});

static MESSAGE_FILTER: Lazy<String> = Lazy::new(|| class! {
	.style("cursor", "pointer")
});

static MESSAGE_FILTER_ACTIVE: Lazy<String> = Lazy::new(|| class! {
	.style("color", "var(--ui-text)")
});

#[derive(Debug, Clone)]
pub struct ViewState {
	channel: Weak<PublicChannel>,

	pub can_send_messages: Mutable<bool>,
	pub open_panel: Mutable<Option<ViewPanel>>,
	pub textarea: Mutable<Option<Rc<RefCell<HtmlTextAreaElement>>>>,

	pub display_ads: Mutable<bool>,
	pub display_chat: Mutable<bool>,
}

impl ViewState {
	pub fn new(channel: &PublicChannel, weak_ref: &Weak<PublicChannel>) -> ViewState {
		ViewState {
			channel: weak_ref.clone(),

			can_send_messages: Mutable::new(true),
			open_panel: Mutable::new(None),
			textarea: Mutable::new(None),

			display_ads: Mutable::new(channel.channel_settings.allows_ads.get()),
			display_chat: Mutable::new(channel.channel_settings.allows_chat.get()),
		}
	}

	pub fn channel(&self) -> Rc<PublicChannel> {
		self.channel.upgrade().expect("a surviving reference")
	}

	pub(in crate::views) fn open_panel_signal(&self, panel: ViewPanel) -> impl Signal<Item = bool> {
		self.open_panel.signal_ref(move |value| {
			matches!(value, Some(content) if mem::discriminant(content) == mem::discriminant(&panel))
		})
	}

	pub (in crate::views) fn any_open_panel_signal(&self) -> impl Signal<Item = bool> {
		self.open_panel.signal_ref(move |value| {
			matches!(value, Some(_))
		})
	}

	pub(in crate::views) fn _no_open_panel_signal(&self) -> impl Signal<Item = bool> {
		self.open_panel.signal_ref(move |value| {
			matches!(value, None)
		})
	}
}

#[derive(Debug, Clone, Copy)]
pub enum ViewPanel {
	Description,
	Report,
}

pub(in crate::views::client) fn render(
	app: Arc<App>,
	screen_state: Rc<ClientScreenState>,
	view_state: Rc<ViewState>,
) -> Dom {
	let channel = view_state.channel();
	let settings = &channel.channel_settings;
	let input_resize_observer = ResizeObserver::new(interactions::set_textarea_size_on_resize).unwrap();

	html!("div", {
		.class(&*component::view::CONTAINER)

		// Header
		.child(html!("header", {
			.class(&*NO_USER_SELECT)
			.class(&*VIEW_HEADER)
			
			// Channel information
			.child(html!("div", {
				.class(&*VIEW_TITLE_BOX)

				// Channel name 
				.child(html!("h1", {
					.class(&*VIEW_TITLE)
					.text("# ")
					.text_signal(channel.name.signal_cloned())
				}))

				// Filter selection
				.child(html!("p", {
					.class(&*VIEW_TITLE_SUBTEXT)
					
					// Chat messages filter
					.child_signal(settings.allows_chat.signal().map(clone!(view_state => move |allows_chat| {
						match allows_chat {
							true => Some(html!("a", {
								.class(&*self::MESSAGE_FILTER)
								.class_signal(&*self::MESSAGE_FILTER_ACTIVE, view_state.display_chat.signal_cloned())
								
								.event(clone!(view_state => move |_: Click| {
									view_state.display_chat.set(!view_state.display_chat.get())
								}))
								
								.text("Chat")
							})),

							false => None,
						}
					})))
					
					// Separator
					.text_signal(map_ref! {
						let allows_ads = channel.channel_settings.allows_ads.signal(),
						let allows_chat = channel.channel_settings.allows_chat.signal() => {
							if allows_chat & allows_ads {
								" | "
							} else {
								""
							}
						}
					})
					// End separator
					
					// Ads filter
					.child_signal(settings.allows_ads.signal().map(clone!(view_state => move |allows_ads| {
						match allows_ads {
							true => Some(html!("a", {
								.class(&*self::MESSAGE_FILTER)
								.class_signal(&*self::MESSAGE_FILTER_ACTIVE, view_state.display_ads.signal_cloned())

								.event(clone!(view_state => move |_: Click| {
									view_state.display_ads.set(!view_state.display_ads.get())
								}))
								
								.text("Ads")
							})),

							false => None,
						}
					})))
				}))
			}))
			// End channel information

			// Description
			.child(html!("p", {
				.class(&*VIEW_DESCRIPTION_MINIMISED)
				.class_signal(&*component::layout::HIDDEN, view_state.open_panel_signal(ViewPanel::Description))
				.text_signal(channel.description.signal_cloned())
			}))

			// Report content button
			.child(html!("div", {
				.class(classname::ALERT_STAFF)
				.class(&*component::button::ALERT_STAFF)

				.child(html!("div", {
					.class(classname::TEXT_CONTENT)
					.class(&*component::button::BUTTON_TEXT)
					.text("Alert Staff")
				}))

				.child(html!("div", {
					.class(classname::ICON)
					.class(&*component::button::BUTTON_ICON_RIGHT)
					.text(&*icon::ERROR_GLYPH)
				}))

				.event(clone!(view_state => move |_: Click| {
					view_state.open_panel.set_if(Some(ViewPanel::Report), |old, new| {
						match (old, new) {
							(Some(old), Some(new)) => mem::discriminant(old) != mem::discriminant(new),
							_ => true
						}
					})
				}))
			}))
		}))

		// Main content view
		.child(html!("main", {
			.class(&*component::channel::CHANNEL)

			// Expanded description
			.child(channel_description(&view_state))
			
			// Expand channel description
			.child(html!("div", {
				.attr("aria-hidden", "true")
				.class(classname::ICON)
				.class(&*component::button::EXPANDER_HORIZONTAL)
				.class(&*NO_USER_SELECT)

				.text_signal(view_state.any_open_panel_signal().map(|value| match value {
					true => &*icon::UP_CHEVRON_GLYPH,
					false => &*icon::DOWN_CHEVRON_GLYPH,
				}))

				.event(clone!(view_state => move |_: Click| {
					view_state.open_panel.set(match view_state.open_panel.get() {
						Some(_) => None,
						None => Some(ViewPanel::Description)
					})
				}))
			}))
			// End expand channel description

			// Message display
			.child(html!("div", {
				.class(&*component::channel::MESSAGE_AREA)

				// Spacer
				.child(html!("div", {
					.style("flex", "1 1 auto")
				}))
				// End spacer

				.children_signal_vec(channel_messages(&app, &view_state, channel.messages.signal_vec_cloned()))
			}))
			// End message display
		}))

		// Message area
		.child(html!("div", {
			.class(&*self::MESSAGE_AREA)
			.class(&*NO_USER_SELECT)

			// Formatting tools
			.child_signal(clone!(view_state => {
				view_state.textarea.signal_ref(clone!(app, view_state => move |textarea| {
					textarea.as_ref().map(|textarea| formatting_bar(app.clone(), view_state.clone(), textarea.clone()))
				}))
			}))
			// End formatting tools

			.child(html!("div", {
				.class(&*self::MESSAGE_BOX)

				// Message textbox
				.child(match view_state.textarea.get_cloned() {
					Some(textarea) => {
						let textarea = &*textarea.borrow();
						
						dom_builder!(textarea.clone(), {
							.after_inserted(clone!(input_resize_observer => move |textarea| {
								input_resize_observer.observe(&textarea);

								focus(&textarea);
							}))

							.after_removed(clone!(input_resize_observer => move |textarea| {
								input_resize_observer.unobserve(&textarea);
							}))
						})
					},

					None => html!("textarea" => HtmlTextAreaElement, {
						.attr("rows", "2")
						.attr_signal("placeholder", channel.name.signal_ref(|name| format!("Message #{}", name)))
						.class(&*component::input::TEXTAREA)
	
						.after_inserted(clone!(view_state, input_resize_observer => move |textarea| {
							input_resize_observer.observe(&textarea);
							focus(&textarea);

							view_state.textarea.set(Some(Rc::new(RefCell::new(textarea))));
						}))

						.after_removed(clone!(input_resize_observer => move |textarea| {
							input_resize_observer.unobserve(&textarea);
						}))
	
						.with_node!(textarea => {
							.event(interactions::set_textarea_size_on_input(textarea))
						})
					})
				})
				// End message textbox

				// Send button
				.child(html!("div", {
					.class(classname::ICON)
					.class(&*component::message_box::SEND_MESSAGE)
					.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
					.text(icon::SEND_FILLED_GLYPH)

					.with_node!(button => {
						.event(clone!(view_state => move |_: Click| {
							if button.class_list().contains(classname::DISABLED) {
								log::warn!("attempted to submit data from disabled button; exiting");
								return;
							}

							let textarea = view_state.textarea.get_cloned();
							let textarea = match textarea {
								Some(ref textarea) => textarea.borrow(),
								None => return,
							};

							let value = textarea.value();
							if !value.trim().is_empty() {
								log::info!("sending value {:?}", value);
							}
						}))
					})
				}))
				// End send button
			}))
		}))
		// End message area
	})
}

pub fn channel_description(view_state: &Rc<ViewState>) -> Dom {
	let channel = view_state.channel();

	html!("div", {
		.attr("aria-hidden", "true")
		.class(&*component::channel::DESCRIPTION_CONTAINER)
		.visible_signal(view_state.open_panel_signal(ViewPanel::Description))
		
		.child(html!("div", {
			.class(&*component::channel::DESCRIPTION)
			.text_signal(channel.description.signal_cloned())
		}))
	})
}

fn channel_messages(app: &Arc<App>, view_state: &Rc<ViewState>, messages: MutableSignalVec<Rc<ChannelMessage>>)
-> impl SignalVec<Item = Dom> {
	group_by_date(&app, messages.filter_signal_cloned(clone!(view_state => move |message| {
		clone!(message => map_ref! {
			let display_ads = view_state.display_ads.signal(),
			let display_chat = view_state.display_chat.signal() => move
			if matches!(message.content, MessageType::Ad(_)) { true } else { *display_chat } &&
			if matches!(message.content, MessageType::Ad(_)) { *display_ads } else { true }
		})
	})))
}

fn focus(textarea: &HtmlTextAreaElement) {
	textarea.focus().expect("textarea focus to be successful");
}

fn formatting_bar(
	app: Arc<App>,
	view_state: Rc<ViewState>,
	textarea: Rc<RefCell<HtmlTextAreaElement>>,
) -> Dom {
	use crate::views::client::formatting_bar;

	html!("div", {
		.class(&*component::message_box::FORMATTING_BAR)
		.children(&mut [
			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::BOLD_LABEL)
				.attr("title", formatting_bar::BOLD_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::BOLD_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::bold_listener(textarea.clone(), view_state.clone())
				}))
			}),

			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::ITALIC_LABEL)
				.attr("title", formatting_bar::ITALIC_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::ITALIC_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::italic_listener(textarea.clone(), view_state.clone())
				}))
			}),

			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::UNDERLINE_LABEL)
				.attr("title", formatting_bar::UNDERLINE_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::UNDERLINE_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::underline_listener(textarea.clone(), view_state.clone())
				}))
			}),
			
			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::STRIKETHROUGH_LABEL)
				.attr("title", formatting_bar::STRIKETHROUGH_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::STRIKETHROUGH_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::strikethrough_listener(textarea.clone(), view_state.clone())
				}))
			}),

			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::TEXT_COLOR_LABEL)
				.attr("title", formatting_bar::TEXT_COLOR_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::FONT_COLOR_GLYPH)

				.with_node!(button => {
					.event(clone!(app, view_state, textarea => move |event: Click| {
						event.stop_propagation();

						listeners::font_color_listener(
							app.clone(),
							button.clone(),
							textarea.clone(),
							view_state.clone()
						)
					}))
				})
			}),

			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::SUPERSCRIPT_LABEL)
				.attr("title", formatting_bar::SUPERSCRIPT_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::FONT_SIZE_INCREASE_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::superscript_listener(textarea.clone(), view_state.clone())
				}))
			}),

			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::SUBSCRIPT_LABEL)
				.attr("title", formatting_bar::SUBSCRIPT_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::FONT_SIZE_DECREASE_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::subscript_listener(textarea.clone(), view_state.clone())
				}))
			}),

			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::HYPERLINK_LABEL)
				.attr("title", formatting_bar::HYPERLINK_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::LINK_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::hyperlink_listener(textarea.clone(), view_state.clone())
				}))
			}),

			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::CHARACTER_LINK_LABEL)
				.attr("title", formatting_bar::CHARACTER_LINK_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::CONTACT_SOLID_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::character_link_listener(textarea.clone(), view_state.clone())
				}))
			}),
			
			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::CHARACTER_AVATAR_LABEL)
				.attr("title", formatting_bar::CHARACTER_AVATAR_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::CONTACT_FRAME_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::avatar_link_listener(textarea.clone(), view_state.clone())
				}))
			}),

			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::EICON_LABEL)
				.attr("title", formatting_bar::EICON_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::EMOJI_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::eicon_listener(textarea.clone(), view_state.clone())
				}))
			}),

			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::SPOILER_LABEL)
				.attr("title", formatting_bar::SPOILER_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::EYE_GLAZE_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::spoiler_listener(textarea.clone(), view_state.clone())
				}))
			}),
			
			html!("div", {
				.attr("aria-role", "button")
				.attr("aria-label", formatting_bar::RAW_LABEL)
				.attr("title", formatting_bar::RAW_TITLE)
				.class(classname::ICON)
				.class(&*component::message_box::FORMATTING_BUTTON)
				.class_signal(classname::DISABLED, not(view_state.can_send_messages.signal()))
				.text(&*icon::STATUS_CIRCLE_BLOCK_GLYPH)

				.event(clone!(view_state, textarea => move |_: Click| {
					listeners::noparse_listener(textarea.clone(), view_state.clone())
				}))
			}),
		])
	})
}

mod interactions {
	use dominator::{class, pseudo};
    use dominator::events::Input;
	use js_sys::Array;
	use lazy_static::lazy_static;
	use wasm_bindgen::JsCast;  
	use web_sys::{HtmlTextAreaElement, ResizeObserver, ResizeObserverEntry};

	pub fn set_textarea_size_on_input(textarea: HtmlTextAreaElement) -> impl FnMut(Input) {
		move |_: Input| {
			textarea.set_rows(calculate_rows(&textarea));
		}
	}

	pub fn set_textarea_size_on_resize(entries: Array, _observer: ResizeObserver) {
		let entries = entries.to_vec();
	
		entries.iter().map(|entry| entry.clone().unchecked_into::<ResizeObserverEntry>()).for_each(|entry| {
			let textarea = entry.target().unchecked_into::<HtmlTextAreaElement>();
	
			textarea.set_rows(calculate_rows(&textarea));
		});
	}

	fn calculate_rows(textarea: &HtmlTextAreaElement) -> u32 {
		lazy_static! {
			static ref NO_SCROLLBAR: String = class! {
				.pseudo!("::-webkit-scrollbar", {
					.style("display", "none")
				})
			};
		}

		const BASE_HEIGHT: u32 = 50;
		const DIFFERENCE: u32 = 21;
		const MAX_ROWS: u32 = 8;
		const MIN_ROWS: u32 = 2;

		let current_row_count = textarea.rows();
		
		match textarea.get_attribute("no-autoresize") {
			Some(_) => current_row_count,

			None => {
				// hide scrollbar to prevent errors when calculating row count
				textarea.class_list().add_1(&*NO_SCROLLBAR).unwrap_or_else(|_| unreachable!());
				textarea.set_rows(MIN_ROWS);

				let scroll_height = textarea.scroll_height();
				let overhang = (scroll_height as u32 - BASE_HEIGHT) / DIFFERENCE;
				let new_row_count = (overhang + MIN_ROWS).min(MAX_ROWS);

				if new_row_count != current_row_count {
					log::trace!("setting row count to {}", new_row_count);
				}

				// re-add scrollbar
				textarea.class_list().remove_1(&*NO_SCROLLBAR).unwrap_or_else(|_| unreachable!());
				new_row_count
			},
		}
	}
}

mod listeners {
	use crate::App;
	// use crate::views::overlays::menus::{ContextMenuPosition, context_menu};
	use super::ViewState;
	// use dominator::{clone, html};
	// use dominator::events::Click;
    use std::cell::{RefCell, Ref};
    use std::rc::Rc;
    use std::sync::Arc;
    use log::error;
    use web_sys::{HtmlTextAreaElement, HtmlElement};

	struct TextboxSelection {
		after: Vec<u16>,
		before: Vec<u16>,
		content: Vec<u16>,
	}

	pub fn avatar_link_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "icon", "icon");
	}

	pub fn bold_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "b", "b");
	}

	pub fn character_link_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "user", "user");
	}

	pub fn eicon_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "eicon", "eicon");
	}

	pub fn font_color_listener(
		_app: Arc<App>,
		_button: HtmlElement,
		textarea: Rc<RefCell<HtmlTextAreaElement>>,
		view_state: Rc<ViewState>,
	) {
		if !view_state.can_send_messages.get() {
			return;
		}
		
		// context_menu(app, ContextMenuPosition::AboveTarget((*button).clone()), clone!(textarea => move |menu| {
		// 	html!("div", {
		// 		.text("context menu")
		// 		.event(clone!(textarea => move |event: Click| {
		// 			event.stop_propagation();

		// 			let textarea = textarea.borrow();
		// 			let fragments = get_fragments(&textarea);

		// 			apply_transform(&textarea, fragments, "color=", "color");
		// 			menu.close();
		// 		}))
		// 	})
		// }));

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "color=", "color");
	}

	pub fn hyperlink_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "url=", "url");
	}

	pub fn italic_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "i", "i");
	}

	pub fn noparse_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "noparse", "noparse");
	}

	pub fn spoiler_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "spoiler", "spoiler");
	}

	pub fn strikethrough_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "s", "s");
	}

	pub fn subscript_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "sub", "sub")
	}

	pub fn superscript_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "sup", "sup")
	}

	pub fn underline_listener(textarea: Rc<RefCell<HtmlTextAreaElement>>, view_state: Rc<ViewState>) {
		if !view_state.can_send_messages.get() {
			return;
		}

		let textarea = textarea.borrow();
		let selection = get_selection(&textarea);
		apply_transform(&textarea, selection, "u", "u");
	}

	fn apply_transform(
		textarea: &Ref<'_, HtmlTextAreaElement>,
		fragments: TextboxSelection,
		tag_open: &str,
		tag_close: &str,
	) {
		let (selection, selection_length) = (String::from_utf16_lossy(&fragments.content), fragments.content.len());
		let (before, before_length) = (String::from_utf16_lossy(&fragments.before), fragments.before.len());
		let after = String::from_utf16_lossy(&fragments.after);
		
		let tag_open_length = tag_open.encode_utf16().count() + 2;
		let tag_close_length = tag_close.encode_utf16().count() + 3;
		let wrapped = format!("[{}]{}[/{}]", tag_open, &selection, tag_close);

		let selection_start = if selection.is_empty() {
			before_length + tag_open_length
		} else {
			before_length
		} as u32;

		let selection_end = if selection.is_empty() {
			before_length + tag_open_length
		} else {
			before_length + tag_open_length + selection_length + tag_close_length
		} as u32;

		let value = before + &wrapped + &after;
		let selection_direction = textarea.selection_direction().unwrap_or_else(|_| {
			unreachable!("selection is allowed on <textarea>")
		}).map(|direction| match direction.as_str() {
			// You gotta love the DOM API. Instead of being undefined - and therefore mapped to None - an unknown
			// direction is the string "none"
			"none" => String::from("forward"), 
			_ => direction
		}).unwrap_or_else(|| String::from("forward"));

		textarea.set_value(&value);
		textarea.set_selection_range_with_direction(selection_start, selection_end, &selection_direction).unwrap();
	}

	fn get_selection(textarea: &Ref<'_, HtmlTextAreaElement>) -> TextboxSelection {
		if let Err(err) = textarea.focus() {
			let err = err.as_string().expect("a value thrown by the JavaScript runtime");
			error!("an error occurred while trying to focus the textbox: {}", &err);
		};

		let value = textarea.value();
		let codepoints: Vec<_> = value.encode_utf16().collect();

		let selection_start = textarea.selection_start().unwrap_or_else(|_| {
			unreachable!("selection is allowed on <textarea>")
		});

		let selection_end = textarea.selection_end().unwrap_or_else(|_| {
			unreachable!("selection is allowed on <textarea>")
		});

		let (selection_start, selection_end) = selection_start.zip(selection_end).map(|(start, end)| {
			let (start, end) = (end.min(start), start.max(end));
			(start as usize, end as usize)
		}).unwrap_or_else(|| {
			let len = textarea.text_length();
			(len as usize, len as usize)
		});

		TextboxSelection {
			before: codepoints[0..selection_start].to_vec(),
			content: codepoints[selection_start..selection_end].to_vec(),
			after: codepoints[selection_end..].to_vec(),
		}
	}
}
