pub mod channel;
pub mod console;
pub mod conversation;

use crate::App;
use crate::state::{Conversation, PublicChannel, SystemMessage};
use crate::styles::{NO_USER_SELECT, component};
use crate::views::sidebars::NavigationView;
use chrono::{Date, Utc};
use dominator::{Dom, class, clone, html};
use futures_signals::signal::{Mutable, Signal, SignalExt, always, Broadcaster, ReadOnlyMutable};
use futures_signals::signal_vec::{MutableVec, MutableVecLockRef, MutableVecLockMut, SignalVec, SignalVecExt};
use once_cell::sync::Lazy;
use snowcat_common::state::{character, Action, ChannelMessage, MessageType, MessageTypeDiscriminant};
use snowcat_common::state::character::Character;
use snowcat_signals::signal_vec::SnowcatSignalVecExt;
use snowcat_signals::signal_vec::group_by_key::SignalVecChunk;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

static VIEW_DESCRIPTION_MINIMISED: Lazy<String> = Lazy::new(|| class! {
	.style("-webkit-box-orient", "vertical")
	.style("-webkit-line-clamp", "2")
	.style("color", "var(--ui-inactive)")
	.style("display", "-webkit-box")
	.style("flex", "1 1 100%")
	.style("font-size", "16px")
	.style("margin-block", "0")
	.style("margin-inline", "20px 35px")
	.style("overflow", "hidden")
	.style("visibility", "visible")
});

static VIEW_HEADER: Lazy<String> = Lazy::new(|| class! {
	.style("align-items", "center")	
	.style("background-color", "var(--background-primary)")
	.style("display", "flex")
	.style("padding", "13px 13px 13px 20px")
});

static VIEW_TITLE: Lazy<String> = Lazy::new(|| class! {
	.style("color", "var(--header-primary)")
	.style("font-size", "20px")
	.style("margin", "0 0 1px 0")
	.style("max-width", "var(--title-max-width)")
	.style("overflow", "hidden")
	.style("text-overflow", "ellipsis")
	.style("white-space", "nowrap")
});

static VIEW_TITLE_BOX: Lazy<String> = Lazy::new(|| class! {
	.style("--title-max-width", "250px")
	.style("display", "flex")
	.style("flex-flow", "column nowrap")
	.style("flex", "1 0 var(--title-max-width)")
});

static VIEW_TITLE_SUBTEXT: Lazy<String> = Lazy::new(|| class! {
	.style("color", "var(--ui-inactive)")
	.style("font-size", "14px")
	.style("margin", "0")
	.style("max-width", "var(--title-max-width)")
});

#[derive(Debug, Clone)]
pub struct ClientScreenState {
	pub(in crate::views) client_view: Rc<Mutable<ClientView>>,
	pub(in crate::views) navigation_view: Rc<Mutable<NavigationView>>,

	pub(in crate::views) open_channels: Rc<MutableVec<Rc<PublicChannel>>>,
	pub(in crate::views) open_conversations: Rc<MutableVec<Rc<Conversation>>>,
	pub(in crate::views) system_messages: Rc<MutableVec<Rc<SystemMessage>>>,

	pub actions_view_collapsed: Mutable<bool>,
	pub actions_view_expanded: Mutable<bool>,
	pub navigation_view_collapsed: Mutable<bool>,
	pub navigation_view_expanded: Mutable<bool>,
}

impl ClientScreenState {
	/// Acquire a read-only reference to the open channel list,
	/// for reading entries.
	pub fn channels(&self) -> MutableVecLockRef<'_, Rc<PublicChannel>> {
		self.open_channels.lock_ref()
	}

	/// Acquire a writable reference to the open channel list, for adding and
	/// removing entries.
	pub fn channels_mut(&self) -> MutableVecLockMut<'_, Rc<PublicChannel>> {
		self.open_channels.lock_mut()
	}

	/// Acquire a read-only reference to the open private conversation list,
	/// for reading entries.
	pub fn conversations(&self) -> MutableVecLockRef<'_, Rc<Conversation>> {
		self.open_conversations.lock_ref()
	}

	/// Acquire a writable reference to the open private conversation list, for
	/// adding and removing entries.
	pub fn conversations_mut(&self) -> MutableVecLockMut<'_, Rc<Conversation>> {
		self.open_conversations.lock_mut()
	}

	/// Acquire a read-only reference to the open private conversation list,
	/// for reading entries.
	pub fn system(&self) -> MutableVecLockRef<'_, Rc<SystemMessage>> {
		self.system_messages.lock_ref()
	}

	/// Acquire a writable reference to the open private conversation list, for
	/// adding and removing entries.
	pub fn system_mut(&self) -> MutableVecLockMut<'_, Rc<SystemMessage>> {
		self.system_messages.lock_mut()
	}
}

#[derive(Debug, Clone)]
pub enum ClientView {
	Console,
	Conversation(Rc<Conversation>, Rc<Character>),
	PublicChannel(Rc<channel::ViewState>),
}

pub fn render(
	app: Arc<App>,
	screen_state: Rc<ClientScreenState>,
	client_view: &ClientView,
) -> Dom {
	match client_view {
		ClientView::Console => clone!(app, screen_state => console::render(app, screen_state)),
		ClientView::Conversation(channel, other_character) => clone!(app, screen_state, other_character, channel => todo!("private conversation rendering")),
		ClientView::PublicChannel(view_state) => channel::render(app.clone(), screen_state.clone(), view_state.clone()),
	}
}

fn clickable_channel_link(channel: &Rc<PublicChannel>) -> Dom {
	html!("a", {
		.child(html!("img", {
			
		}))
	})
}

fn clickable_profile_link(character: &Rc<Character>, show_profile_picture: ReadOnlyMutable<bool>) -> Dom {
	html!("a", {
		.child_signal(show_profile_picture.signal().map(clone!(character => move |show_profile_picture| {
			// UNSTABLE: feature(bool_to_option)
			show_profile_picture.then_some(html!("img", {
				.attr("aria-hidden", "true")
				.attr_signal("src", profile_picture_url(&character).map(Some))
				.class(&*component::channel::character::PROFILE_PICTURE_INLINE)
				.class(&*NO_USER_SELECT)
			}))
		})))

		// UNSTABLE: feature(bool_to_option)
		.text_signal(show_profile_picture.signal().map(|show_profile_picture| show_profile_picture.then_some(" ").unwrap_or("")))
		.text_signal(always(character.name.clone()))
	})
}

fn group_ads_by_author(
	app: &Arc<App>,
	date: &Rc<str>,
	groups: impl SignalVec<Item = SignalVecChunk<Rc<character::Character>, Rc<ChannelMessage>>> + 'static,
) -> Pin<Box<dyn SignalVec<Item = Dom>>> {
	let groups = clone!(date => groups.map(move |messages| {
		html!("div", {
			.class(&*component::channel::message::AD_GROUP_WRAPPER)

			.child(html!("div", {
				.class(&*component::channel::message::AD_GROUP_BACKGROUND)
			}))

			.child(html!("div", {
				.class(&*component::channel::message::GROUP)
				
				.child(html!("div", {
					.class(&*component::channel::message::HEAD)

					.child(html!("img", {
						.attr_signal("src", profile_picture_url(&*messages.key))
						.class(&*component::channel::character::PROFILE_PICTURE)
						.class(&*NO_USER_SELECT)
					}))

					.child(html!("p", {
						.class(&*component::channel::message::DATE)
						.class(&*NO_USER_SELECT)
						.text(&date)
					}))

					.child(html!("h2", {
						.class(&*component::channel::message::AUTHOR)
						.text(&messages.key.name)
					}))
				}))

				.children_signal_vec(show_messages(messages))
			}))
		})
	}));

	Box::pin(groups)
}

fn group_messages_by_author(
	app: &Arc<App>,
	date: &Rc<str>,
	groups: impl SignalVec<Item = SignalVecChunk<Rc<character::Character>, Rc<ChannelMessage>>> + 'static,
) -> Pin<Box<dyn SignalVec<Item = Dom>>> {
	let groups = clone!(date => groups.map(move |messages| {
		html!("div", {
			.class(&*component::channel::message::GROUP)
			
			.child(html!("div", {
				.class(&*component::channel::message::HEAD)

				.child(html!("img", {
					.attr_signal("src", profile_picture_url(&*messages.key))
					.class(&*component::channel::character::PROFILE_PICTURE)
					.class(&*NO_USER_SELECT)
				}))

				.child(html!("p", {
					.class(&*component::channel::message::DATE)
					.class(&*NO_USER_SELECT)
					.text(&date)
				}))

				.child(html!("h2", {
					.class(&*component::channel::message::AUTHOR)
					.text(&messages.key.name)
				}))
			}))

			.children_signal_vec(show_messages(messages))
		})
	}));

	Box::pin(groups)
}

fn group_by_date(
	app: &Arc<App>,
	messages: impl SignalVec<Item = Rc<ChannelMessage>>
) -> impl SignalVec<Item = Dom> {
	group_by_type(app, messages.group_by_key(|message| message.timestamp.date()))
}

fn group_by_type(
	app: &Arc<App>,
	groups: impl SignalVec<Item = SignalVecChunk<Date<Utc>, Rc<ChannelMessage>>>
) -> impl SignalVec<Item = Dom> {
	clone!(app => groups.map(move |messages| {
		let date: Rc<str> = Rc::from(messages.key.format("%A, %B %d, %Y").to_string());
		log::debug!(r#"[group_by_type] processing date "{:?}""#, &*date);

		// By-date container
		html!("div", {
			.children_signal_vec(make_containers(&app, &date,
				messages.group_by_key(|message| message.content.discriminant())
			))
		})
	}))
}

fn make_containers(
	app: &Arc<App>,
	date: &Rc<str>,
	groups: impl SignalVec<Item = SignalVecChunk<MessageTypeDiscriminant, Rc<ChannelMessage>>>,
) -> impl SignalVec<Item = Dom> {
	clone!(app, date => groups.map(move |messages| {
		let message_type = &messages.key;
		let children = match message_type {
			MessageTypeDiscriminant::Action => show_actions(&app, messages),
			MessageTypeDiscriminant::Ad => {
				group_ads_by_author(&app, &date, messages.group_by_key(|message| message.author.clone()))
			},
			
			MessageTypeDiscriminant::Message => {
				group_messages_by_author(&app, &date, messages.group_by_key(|message| message.author.clone()))
			},
		};

		html!("div", {
			.children_signal_vec(children)
		})
	}))
}

fn profile_picture_url(_character: &character::Character) -> impl Signal<Item = String> {
	always(String::from("https://cdn.pixabay.com/photo/2016/08/08/09/17/avatar-1577909_960_720.png"))
}

fn show_actions(
	app: &Arc<App>,
	actions: SignalVecChunk<MessageTypeDiscriminant, Rc<ChannelMessage>>
) -> Pin<Box<dyn SignalVec<Item = Dom>>> {
	fn action_text(action: &Action, author: &character::Character) -> Dom {
		match action {
			Action::Bottle { choice } => html!("div", {
				.class(&*component::channel::message::CONTENT)

				.child(html!("h2", {
					.class(&*component::channel::message::AUTHOR)
					.class(&*component::layout::INLINE_BLOCK)
					.text_signal(always(author.name.clone()))
				}))

				.text(" spins the bottle: ")
				
				.child(html!("span", {
					.class(&*component::channel::message::AUTHOR)
					.text_signal(always(choice.name.clone()))
				}))
			}),

			Action::Post(post) => html!("div", {
				.class(&*component::channel::message::CONTENT)
				.class(&*component::channel::action::POST)

				.child(html!("h2", {
					.class(&*component::channel::message::AUTHOR)
					.class(&*component::layout::INLINE_BLOCK)
					.text_signal(always(author.name.clone()))
				}))

				.text_signal(always(post.clone()))
			}),

			action_type => todo!("action type {}", action_type.discriminant())
		}
	}

	// even though it cannot be statically checked, we *know* that these
	// are MessageType::Action(Action) instances
	let actions = actions.map(move |message| {
		let action = match &message.content {
			MessageType::Action(action) => action,
			message_type => unreachable!("message type should be {correct}, got {current}",
				correct=MessageTypeDiscriminant::Action,
				current=message_type.discriminant())
		};
		
		html!("div", {
			.class(&*component::channel::action::ACTION)

			.child(html!("img", {
				.attr_signal("src", profile_picture_url(&message.author))
				.class(&*component::channel::character::PROFILE_PICTURE)
				.class(&*NO_USER_SELECT)
			}))

			.child(html!("time", {
				.attr("datetime", &message.timestamp.format("%+").to_string())
				.class(&*component::channel::message::DATE)
				.class(&*NO_USER_SELECT)
				.text(&message.timestamp.format("%A, %B %d, %Y").to_string())

				.child(html!("span", {
					.class(&*component::channel::message::DATE_INNER_TIMESTAMP)
					.text(&message.timestamp.format(" at %R").to_string())
				}))
			}))

			.child(action_text(&action, &message.author))
		})
	});

	Box::pin(actions)
}

fn show_messages(messages: impl SignalVec<Item = Rc<ChannelMessage>>) -> impl SignalVec<Item = Dom> {
	// even though it cannot be statically checked, we *know* that these
	// are MessageType::Message(String) or MessageType::Ad(String)
	// instances.
	messages.map(|message| {
		let content = match &message.content {
			MessageType::Ad(ad) => ad,
			MessageType::Message(message) => message,
			message_type => unreachable!("message type should be {correct1} or {correct2}, got {current}",
				correct1=MessageTypeDiscriminant::Ad,
				correct2=MessageTypeDiscriminant::Message,
				current=message_type.discriminant())
		};

		html!("div", {
			.class(&*component::channel::message::MESSAGE)

			.child(html!("time", {
				.attr("datetime", &message.timestamp.format("%+").to_string())
				.class(&*component::channel::message::TIME)
				.class(&*NO_USER_SELECT)
				.text(&message.timestamp.format("%R").to_string())
			}))

			.child(html!("div", {
				.class(&*component::channel::message::CONTENT)
				.text(content.as_ref())
			}))
		})
	})
}

mod formatting_bar {
	pub(super) const BOLD_LABEL: &str = "Insert bold text.";
	pub(super) const BOLD_TITLE: &str = "Insert bold text.";

	pub(super) const CHARACTER_AVATAR_LABEL: &str = "Insert character avatar.";
	pub(super) const CHARACTER_AVATAR_TITLE: &str = "Insert a character's avatar as a link to their profile.";

	pub(super) const CHARACTER_LINK_LABEL: &str = "Link to character.";
	pub(super) const CHARACTER_LINK_TITLE: &str = "Insert a link to a character profile.";

	pub(super) const EICON_LABEL: &str = "Insert eicon.";
	pub(super) const EICON_TITLE: &str = "Insert a named icon into chat. The default avatar will be rendered if no eicon has the given name.";

	pub(super) const HYPERLINK_LABEL: &str = "Insert hyperlink.";
	pub(super) const HYPERLINK_TITLE: &str = "Insert a hyperlink to a webpage.";

	pub(super) const ITALIC_LABEL: &str = "Insert italicised text.";
	pub(super) const ITALIC_TITLE: &str = "Insert italicised text.";

	pub(super) const RAW_LABEL: &str = "Insert raw text.";
	pub(super) const RAW_TITLE: &str = "Insert a block of text where any BBcode present will not be parsed, instead showing up exactly as written.";

	pub(super) const SPOILER_LABEL: &str = "Insert spoiler text.";
	pub(super) const SPOILER_TITLE: &str = "Insert a block of text that will be hidden until clicked on.";
	
	pub(super) const STRIKETHROUGH_LABEL: &str = "Insert strikethrough text.";
	pub(super) const STRIKETHROUGH_TITLE: &str = "Insert strikethrough text.";

	pub(super) const SUBSCRIPT_LABEL: &str = "Insert subscript text.";
	pub(super) const SUBSCRIPT_TITLE: &str = "Push text below the baseline. Cannot be nested.";

	pub(super) const SUPERSCRIPT_LABEL: &str = "Insert superscript text.";
	pub(super) const SUPERSCRIPT_TITLE: &str = "Raise text above the baseline. Cannot be nested.";

	pub(super) const TEXT_COLOR_LABEL: &str = "Set text color.";
	pub(super) const TEXT_COLOR_TITLE: &str = "Set text color.";

	pub(super) const UNDERLINE_LABEL: &str = "Insert underlined text.";
	pub(super) const UNDERLINE_TITLE: &str = "Insert underlined text.";
}
