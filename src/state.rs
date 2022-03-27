use crate::state::settings::AppSettings;
use crate::views::client::{channel, conversation};
use chrono::{DateTime, Utc};
use futures_signals::signal::Mutable;
use futures_signals::signal_vec::{MutableVec, MutableVecLockRef, MutableVecLockMut};
use snowcat_common::state::ConversationStateChange;
use snowcat_common::state::{ChannelStateChange, ChannelMessage, character::Character};
use std::rc::Rc;

/// Stores general application information, such as the set of available
/// channels and characters.
#[derive(Debug)]
pub struct AppState {
	channel_cache: MutableVec<Rc<PublicChannel>>,
	character_cache: MutableVec<Rc<Character>>,

	pub settings: Rc<AppSettings>,
	pub user: Option<Rc<Character>>,
}

impl AppState {
	pub fn new(settings: AppSettings) -> Rc<Self> {
		let channels: Vec<_> = mock::get_channels().into_iter().map(|channel| {
			PublicChannel::attach_view_state(channel)
		}).collect();

		let characters = mock::get_characters().to_vec();

		Rc::new(AppState {
			settings: Rc::new(settings),
			user: Some(characters[3].clone()),

			channel_cache: MutableVec::new_with_values(channels),
			character_cache: MutableVec::new_with_values(characters),
		})
	}

	/// Acquire a read-only reference to the channel cache, for reading entries.
	pub fn channels(&self) -> MutableVecLockRef<'_, Rc<PublicChannel>> {
		self.channel_cache.lock_ref()
	}

	/// Acquire a writable reference to the channel cache, for adding and
	/// removing entries.
	pub fn channels_mut(&self) -> MutableVecLockMut<'_, Rc<PublicChannel>> {
		self.channel_cache.lock_mut()
	}

	/// Acquire a read-only reference to the character cache, for reading
	/// entries.
	pub fn characters(&self) -> MutableVecLockRef<'_, Rc<Character>> {
		self.character_cache.lock_ref()
	}


	/// Acquire a writable reference to the character cache, for adding and
	/// removing entries.
	pub fn characters_mut(&self) -> MutableVecLockMut<'_, Rc<Character>> {
		self.character_cache.lock_mut()
	}

	/// Acquire a reference to the currently logged in user.
	pub fn user(&self) -> Option<Rc<Character>> {
		self.user.clone()
	}
}

#[derive(Debug, Clone)]
pub struct Conversation {
	pub character: Character,
	pub messages: Rc<MutableVec<Rc<ChannelMessage>>>,

	view_state: Option<Rc<conversation::ViewState>>,
}

impl Conversation {
	pub fn _process_change(&self, _change: ConversationStateChange) {
		todo!("processing changes to conversation state is not yet implemented")
	}

	pub fn attach_view_state(mut conversation: Conversation) -> Rc<Conversation> {
		use crate::views::client::conversation::ViewState;

		// UNSTABLE: feature(arc_new_cyclic)
		Rc::new_cyclic(|self_ref| {
			let view_state = ViewState::new(&conversation, self_ref);
			conversation.view_state = Some(Rc::new(view_state));

			conversation
		})
	}

	pub fn view_state(&self) -> &Rc<conversation::ViewState> {
		self.view_state.as_ref().expect("a view state object")
	}
}

impl PartialEq<Conversation> for Conversation {
	fn eq(&self, other: &Conversation) -> bool {
		self.character.id == other.character.id
	}
}

#[derive(Debug, Clone)]
pub enum Notification {
	Broadcast {
		content: String,
		sender: Rc<Character>,
	},

	Mention {
		channel: Rc<PublicChannel>,
		message: Rc<ChannelMessage>,
	},
}

impl Notification {
	pub fn broadcast(content: &str, sender: &Rc<Character>) -> Notification {
		Notification::Broadcast {
			content: content.to_string(),
			sender: sender.clone(),
		}
	}

	pub fn mention(channel: &Rc<PublicChannel>, message: &Rc<ChannelMessage>) -> Notification {
		Notification::Mention {
			channel: channel.clone(),
			message: message.clone(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct PublicChannel {
	pub id: String,
	pub name: Mutable<String>,
	pub description: Mutable<String>,
	pub characters: Rc<MutableVec<Rc<Character>>>,
	pub messages: Rc<MutableVec<Rc<ChannelMessage>>>,
	pub pinned: Mutable<bool>,
	pub official_channel: bool,

	pub channel_settings: PublicChannelSettings,

	view_state: Option<Rc<channel::ViewState>>,
}

impl PublicChannel {
	pub fn _process_change(&self, _change: ChannelStateChange) {
		todo!("processing changes to channel state is not yet implemented")
	}

	pub fn attach_view_state(mut channel: PublicChannel) -> Rc<PublicChannel> {
		use crate::views::client::channel::ViewState;

		// UNSTABLE: feature(arc_new_cyclic)
		Rc::new_cyclic(|self_ref| {
			let view_state = ViewState::new(&channel, self_ref);
			channel.view_state = Some(Rc::new(view_state));

			channel
		})
	}

	pub fn view_state(&self) -> &Rc<channel::ViewState> {
		self.view_state.as_ref().expect("a view state object")
	}
}

impl PartialEq<PublicChannel> for PublicChannel {
	fn eq(&self, other: &PublicChannel) -> bool {
		self.id == other.id
	}
}

impl Eq for PublicChannel { }

#[derive(Debug, Clone)]
pub struct PublicChannelSettings {
	pub allows_ads: Mutable<bool>,
	pub allows_chat: Mutable<bool>,
}

#[derive(Debug, Clone)]
pub struct SystemMessage {
	pub data: Notification,
	pub timestamp: DateTime<Utc>,
}

impl SystemMessage {
	pub fn new(data: Notification, timestamp: DateTime<Utc>) -> SystemMessage {
		SystemMessage {
			data,
			timestamp,
		}
	}
}

#[allow(clippy::redundant_clone)]
#[allow(clippy::vec_init_then_push)]
pub mod mock {
	use super::{Notification, PublicChannel, PublicChannelSettings};
	use chrono::{DateTime, ParseResult, Utc};
	use futures_signals::signal::Mutable;
    use futures_signals::signal_vec::MutableVec;
    use snowcat_common::state::{ChannelMessage, MessageType, Action};
    use snowcat_common::state::character::Character;
    use std::rc::Rc;

	thread_local! {
		pub static CHARACTERS: [Rc<Character>; 7] = [
			Rc::new(Character {
				id: 20154,
				name: String::from("Haze"),
			}),

			Rc::new(Character {
				id: 2543,
				name: String::from("Marabel Thorne"),
			}),

			Rc::new(Character {
				id: 191498,
				name: String::from("Markelio"),
			}),

			Rc::new(Character {
				id: 273593,
				name: String::from("Phoney Baloney"),
			}),

			Rc::new(Character {
				id: 7614,
				name: String::from("Ryos"),
			}),

			Rc::new(Character {
				id: 327067,
				name: String::from("Sarah Blitz Garrison"),
			}),

			Rc::new(Character {
				id: 68851,
				name: String::from("Yanozo Serna"),
			}),
		];
	}

	pub fn get_channels() -> Vec<PublicChannel> {
		let helpdesk = PublicChannel {
			id: String::from("Helpdesk"),
			name: Mutable::new(String::from("Helpdesk")),
			description: Mutable::new(String::from("[i][b][color=red][u]READ HERE FIRST[/u][/color][/b][/i] — [color=green][url=https://wiki.f-list.net/Rules]F-List's CoC[/url][/color] [color=green][url=https://wiki.f-list.net/Frequently_Asked_Questions]F-List's FAQ[/url][/color] [color=green][url=https://www.f-list.net/tickets.php]Helpdesk Tickets[/url][/color] [color=green][url=https://www.f-list.net/bugreports.php]Bug Reports[/url][/color] — [color=yellow]All discussions for F-list 2.0 should go to [session=F-List 2.0 Discussion]F-List 2.0 Discussion[/session][/color]\nPlease initially attempt to answer your own questions by visiting F-List's FAQ. If you still need help, [u][b]clearly state your question in the channel[/b][/u] instead of asking to speak with someone.\nThis channel is for general questions related to F-List that present staff or users may answer. The actual Helpdesk consists of filing tickets, which Helpdesk staff process—if you have a complaint or a report to file, [u][b]do not[/b][/u] bring it up here. Keep in mind that this channel is not for your personal issues such as trying to find RP.\nIf—after posing your question to the channel and waiting a while—no one responds, or if your problem is of an urgent nature, [color=green]then mention the [b]full[/b] name of a moderator [b](those with diamonds and stars)[/b] to get their attention.[/color] Keep in mind that moderators may be idling, and if you go a while more with no response, try someone else. [color=red]If no one appears to be able to answer your question, submit a [url=https://www.f-list.net/tickets.php]Helpdesk Ticket[/url] which a staff member will handle in due time.[/color]")),
			characters: Rc::new(MutableVec::new()),
			messages: Rc::new(MutableVec::new()),
			pinned: Mutable::new(true),
			official_channel: true,

			channel_settings: PublicChannelSettings {
				allows_ads: Mutable::new(false),
				allows_chat: Mutable::new(true),
			},

			view_state: None,
		};

		let development = PublicChannel {
			id: String::from("Development"),
			name: Mutable::new(String::from("Development")),
			description: Mutable::new(String::from("This channel is for discussing the development of third-party F-Chat clients. Documentation on the F-Chat Protocol is publicly available. Please report any bugs in the bug tracker! If you are implementing a chat bot, please see the bot rules, and mention you're testing a bot, before you actually start testing it in here. (Just to keep the confusion to a minimum.)")),
			characters: Rc::new(MutableVec::new()),
			messages: Rc::new(MutableVec::new_with_values(messages::development())),
			pinned: Mutable::new(false),
			official_channel: true,

			channel_settings: PublicChannelSettings {
				allows_ads: Mutable::new(false),
				allows_chat: Mutable::new(true),
			},

			view_state: None,
		};

		let hyper_giga_growth = PublicChannel {
			id: String::from("ADH-Hyper/Giga Growth"),
			name: Mutable::new(String::from("Hyper/Giga Growth")),
			description: Mutable::new(String::from("For those that find \"hyper\" sizes just not big enough here is the place where you can find other like minded people who are bigger than or want to grow to be bigger than generic hypers, including people who wish to help others grow to such sizes! Things like bigger than the earth, the galaxy even the universe itself! There is absolutely no limit when it comes to sizes in this room.\n\nWhether it be cock, balls,breasts, muscles, ass, etc. Even the number of gentials and breasts you have there is no limit! Whatever it is that you're into. Can you cum enough to flood the earth? Great! The universe? Even better! Same thing goes with sizes!\n\n[b]Remember people this place is meant to be friendly and fun so treat everyone here with respect otherwise you may get kicked.[/b] Ads are allowed here, but do not spam the chat with them, 1 every 10 minutes is the most i'll tolerate. Don't be afraid to say hi and invite others to join! [b]Public RP is allowed![/b] Friend us here: [icon]Hyper Giga Growth[/icon]\n\nAlso try out our growth game with each other and see how big you all can get! Details are on the channel's profile: [icon]Hyper Giga Growth[/icon]")),
			characters: Rc::new(MutableVec::new()),
			messages: Rc::new(MutableVec::new_with_values(messages::hyper_giga_growth())),
			pinned: Mutable::new(false),
			official_channel: false,

			channel_settings: PublicChannelSettings {
				allows_ads: Mutable::new(true),
				allows_chat: Mutable::new(true),
			},

			view_state: None,
		};

		let flist_20_discussion = PublicChannel {
			id: String::from("F-List 2.0 Discussion"),
			name: Mutable::new(String::from("F-List 2.0 Discussion")),
			description: Mutable::new(String::from("This channel is intended for discussion and help with the new F-List 2.0 site during the testing phases.\n\nThe 2.0 Test Site is currently . . . [color=green][b]Open![/b][/color]\n\n6/04/2020:  Feedback Submission and Comments have been frozen for staff review.  Voting is still enabled, but no new feedback can be submitted and users cannot add comments to existing feedback. We are doing this so that we can address and review feedback in a more organized manner.  Feedback submission will re-open when we have completed this review, but we have no ETA for the duration of this review period.\n\n12/25/2020:  The test site has been updated and reopened for the Open Beta Test phase.\n\n[url=https://icantbelieveitsnotfinishedyet.f-list.net/]You may access the F-List 2.0 test site here.[/url]\n\nFeedback submission has been changed slightly.  Please make sure to access the [b]Feedback[/b] section under the [b]Help[/b] dropdown from the main menu.  You may submit feature requests, bug reports, and other feedback there. \n\nPlease make sure to utilize the [b]Search[/b] feature of the Feedback section to ensure no one has posted a similar submission.  [b]When posting feedback/bugreports, please make sure to only post one issue per submission.[/b]\n\nPlease remember, all site [url=https://wiki.f-list.net/Code_of_Conduct]Code of Conduct[/url] policies are to be followed in this channel.  If an incident occurs you believe should be reported for site moderation, please use the [url=https://wiki.f-list.net/How_to_Report_a_User]Alert Staff[/url] button to bring this to our attention!")),
			characters: Rc::new(MutableVec::new()),
			messages: Rc::new(MutableVec::new()),
			pinned: Mutable::new(false),
			official_channel: true,

			channel_settings: PublicChannelSettings {
				allows_ads: Mutable::new(false),
				allows_chat: Mutable::new(true),
			},

			view_state: None,
		};

		vec![helpdesk, development, hyper_giga_growth, flist_20_discussion]
	}

	pub fn get_characters() -> [Rc<Character>; 7] {
		CHARACTERS.with(|characters| characters.clone())
	}

	pub fn get_notifications() -> Vec<Notification> {
		let [_, marabel_thorne, ..] = get_characters();

		vec![
			Notification::broadcast("This sentence is false.", &marabel_thorne)
		]
	}

	fn parse_datetime(datetime: &str) -> ParseResult<DateTime<Utc>> {
		DateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S %z")
			.map(|datetime| datetime.with_timezone(&Utc))
	}

	mod messages {
		use super::{get_characters, parse_datetime};
		use snowcat_common::state::{ChannelMessage, MessageType, Action};
		use std::rc::Rc;

		pub fn development() -> Vec<Rc<ChannelMessage>> {
			let [_, marabel_thorne, markelio, _, _, sarah_blitz_garrison, _] = get_characters();
	
			vec![
				Rc::new(ChannelMessage {
					author: marabel_thorne.clone(),
					timestamp: parse_datetime("2021-07-20 23:01:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("For me, PRI isn't yet working. I get a syntax error, but I only tried it for the first time today")),
				}),
	
				Rc::new(ChannelMessage {
					author: marabel_thorne.clone(),
					timestamp: parse_datetime("2021-07-20 23:01:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("I'd assume that if it changed, then the clients wouldn't work")),
				}),
	
				Rc::new(ChannelMessage {
					author: sarah_blitz_garrison.clone(),
					timestamp: parse_datetime("2021-07-20 23:05:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("I'm getting NOTHING but LIS")),
				}),
	
				Rc::new(ChannelMessage {
					author: sarah_blitz_garrison.clone(),
					timestamp: parse_datetime("2021-07-20 23:06:30 +00:00").unwrap(),
					content: MessageType::Message(String::from("Alright, now it pushed past the massive LIS and didn't crash")),
				}),
	
				Rc::new(ChannelMessage {
					author: sarah_blitz_garrison.clone(),
					timestamp: parse_datetime("2021-07-20 23:08:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("Did LIS always exist because it's literally clogging up my bot so badly it won't get to the PIN before getting kicked")),
				}),
	
				Rc::new(ChannelMessage {
					author: marabel_thorne.clone(),
					timestamp: parse_datetime("2021-07-20 23:08:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("Hmm, too many people online for it to handle maybe?")),
				}),
	
				Rc::new(ChannelMessage {
					author: sarah_blitz_garrison.clone(),
					timestamp: parse_datetime("2021-07-20 23:08:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("I managed to get it up and running only once now and that was thanks to it getting through the LIS spam")),
				}),
	
				Rc::new(ChannelMessage {
					author: marabel_thorne.clone(),
					timestamp: parse_datetime("2021-07-20 23:08:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("It doesn't seem new.")),
				}),
	
				Rc::new(ChannelMessage {
					author: sarah_blitz_garrison.clone(),
					timestamp: parse_datetime("2021-07-20 23:15:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("Seems like it literally was due to F-Chat having too many people online")),
				}),
	
				Rc::new(ChannelMessage {
					author: sarah_blitz_garrison.clone(),
					timestamp: parse_datetime("2021-07-20 23:20:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("Traffic gets clogged up by LIS spam, bot handles the useless LIS requests too slowly to reach the PIN bot, bot gets kicked out")),
				}),
	
				Rc::new(ChannelMessage {
					author: markelio.clone(),
					timestamp: parse_datetime("2021-07-20 23:30:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("Instead of responding to PIN messages, you can just send PIN messages on a timer")),
				}),
	
				Rc::new(ChannelMessage {
					author: sarah_blitz_garrison.clone(),
					timestamp: parse_datetime("2021-07-20 23:31:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("Oh? Odd, that was causing me issues when I tried to make it that way at first")),
				}),
	
				Rc::new(ChannelMessage {
					author: markelio.clone(),
					timestamp: parse_datetime("2021-07-20 23:33:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("I've had luck sending one every 30 seconds")),
				}),
	
				Rc::new(ChannelMessage {
					author: markelio.clone(),
					timestamp: parse_datetime("2021-07-20 23:34:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("Not sure what the ideal interval is")),
				}),
	
				Rc::new(ChannelMessage {
					author: markelio.clone(),
					timestamp: parse_datetime("2021-07-20 23:35:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("FChat 3.0 seems to properly respond to PIN messages and do nothing else, though")),
				}),
	
				Rc::new(ChannelMessage {
					author: markelio.clone(),
					timestamp: parse_datetime("2021-07-20 23:36:00 +00:00").unwrap(),
					content: MessageType::Message(String::from("And it depends on the PINs coming in to not time out")),
				}),
			]
		}

		pub fn hyper_giga_growth() -> Vec<Rc<ChannelMessage>> {
			let [haze, _, _, _, ryos, _, _] = get_characters();

			vec![
				Rc::new(ChannelMessage {
					author: haze.clone(),
					timestamp: parse_datetime("2021-07-20 23:08:00 +00:00").unwrap(),
					content: MessageType::Action(Action::Post(String::from(" pokes her head into the room, quirking a brow at the sight before her and the feeling of water freezing, rapidly retreating from where she is. Someone was certainly here just a bit ago, but it was impossible to tell who it might have been from here, or what they were, for that matter. She frowns as she starts to walk in, a gentle mist rising up from the water to brush over the ground, searching the place for any traps that may be around. Finding none, she lets the water fall back down and flow away. A quick peek around the room helps to confirm that whomever it is left already, and that Trance must be hiding them down here. With a small smile, she starts to follow the path the ice left in her mind, pulling in her power to better hide herself as she begins to stalk her new found prey.\n\nEven if there wasn't ice, she might have been able to track Ryos as he fled, simply seeking the blood in his veins to keep track of him, her senses growing far more attuned to the flow of fluids as she goes along, her focus narrowing down to track him, just keeping down her slowly growing instincts of an assassin. No. This, she's sure, is simply play. Best to keep things painless."))),
				}),

	
				Rc::new(ChannelMessage {
					author: ryos.clone(),
					timestamp: parse_datetime("2021-07-20 23:14:00 +00:00").unwrap(),
					content: MessageType::Action(Action::Post(String::from(" - An old mirror sits unnoticed in the corner the dividing archway, reflecting the world in front of it… for a time. Haze's reflection actually disappears from the mirror before she has even left the confines of the divider, and it's just as she steps away from the tunnel that the mirror disappears entirely. No burst of power precedes the event; no ripples in space-time to signify a hole in reality opening. It just straight up vanishes as if it hadn't been there to begin with. A curious event to be sure, had anyone actually stuck around to witness it. As it is, the vanishing of the mirror goes completely unnoticed by all parties.")))
				}),
	
				Rc::new(ChannelMessage {
					author: ryos.clone(),
					timestamp: parse_datetime("2021-07-20 23:14:00 +00:00").unwrap(),
					content: MessageType::Action(Action::Post(String::from(" slowly makes his way through the tunnel network, working to put as much distance between him and the power well of his pursuer when it begins to dim from his senses. Not a sudden change as Haze withdrawing the flood that surrounds her would imply; but a gradual one. The purple dragon initially makes the mistake of assuming that he is finally losing his unknown pursuer and slows down, ceasing the cold aura around his body as he feels it isn't necessary any more. He comes to a stop, and immediately notices his mistake — the sensation he is feeling isn't coming from his own senses as it was before. It's like it is being layered on top of what he is receiving from his surroundings, but the purpose of such a trick remains unknown to him. He steps forward, feeling it fade just a little more, and turns the corner into a side tunnel. A dead end, with an antique mirror hanging on the far wall. His eyes are drawn to the frame of the mirror and he fails to pay attention to his own walking, causing his already slow stride to falter. The dragon losing valuable time — and distance — to the mirror. The simulacrum of Haze's power dies out more and more the closer he steps to the mirror, and is completely gone by the time Ryos stops in front of it. At no point does this mirror show the dragon's reflection, and even the reflection of the tunnel seems just slightly off…")))
				}),
	
				Rc::new(ChannelMessage {
					author: ryos.clone(),
					timestamp: parse_datetime("2021-07-20 23:14:00 +00:00").unwrap(),
					content: MessageType::Action(Action::Post(String::from(" - A quick glance over his shoulder tells him that, unlike what the mirror shows, Haze is not in fact at the end of the tunnel. Not that he would ever recognise her even if she was, as he has never seen her in the real world and has only the scent of her strength to go off for identifying her. A scent that has vanished; a fact that scares him. A lot. More than enough for his mind to justify a very stupid idea. Ryos lifts a paw and presses it against the surface of the mirror. The sudden appearance of his reflection on the other side shocks him more than the disappearance of the other purple dragon, though that shock is short-lived as his perspective seems to… shift. Almost as if it were leaving his body and transferring to the reflection, which stares back through the mirror to the side he was once on. Ryos lowers his paw and the body that was once him disappears from the world entirely, leaving only the body that is now him to turn around and look over the landscape behind him. The low light of the tunnels seems almost inverted in hue, but that's not the most glaring change in his surroundings or perception that he suffers. \"?kceh eht tahW\" Ryos' jaw gapes at the sound that meets his ears when he speaks. It's… it's his own voice, but… he's never heard himself speak that way before. [i]'That did NOT sound right.'[/i] He takes a step forward and almost immediate collapses to the ground as the wrong side of his body moves, but manages to catch himself before he hits the ground. \",soyR si eman yM .gnitset ,gnitseT\" He mutters to himself, almost a confirmation than anything else. \".won sdrawkcab kaeps I ,yakO\" A nod, and this time he braces himself before walking deeper into the inversely-lit tunnel of this… mirror world, careful to move his body in the inverse to how he would usually walk, to maintain gait. It's a strange sensation, but not something he can't get used to rather quickly. After all, he's dealt with worse. Is dealing with worse."))),
				}),
			]
		}
	}
}

pub mod settings {
	use futures_signals::signal::Mutable;
	use futures_signals::signal_vec::MutableVec;
	use snowcat_common::settings::{
		self as common_settings, ClickOpenTarget, ClockFormat, DisplaySize, LogStorageMethod, MovementShortcut,
		TextFormatShortcut,
	};
	use std::collections::HashMap;

	// Stores the application settings.
	#[derive(Debug)]
	pub struct AppSettings {
		pub client: ClientSettings,
		pub logger: LoggerSettings,
		pub notifcations: NotificationSettings,
		pub shortcuts: KeyboardShortcuts,
	}

	impl From<common_settings::Settings> for AppSettings {
		fn from(settings: common_settings::Settings) -> Self {
			AppSettings {
				client: ClientSettings::from(settings.client),
				logger: LoggerSettings::from(settings.logger),
				notifcations: NotificationSettings::from(settings.notifications),
				shortcuts: KeyboardShortcuts::from(settings.shortcuts),
			}
		}
	}

	#[derive(Debug)]
	pub struct ClientSettings {
		pub animate_eicons: Mutable<bool>,
		pub character_name_click_opens: Mutable<ClickOpenTarget>,
		pub clock_format: Mutable<ClockFormat>,
		pub display_size: Mutable<DisplaySize>,
		pub exclude_tags: MutableVec<String>,
		pub inactivity_timer: Mutable<Option<f32>>,
		pub show_avatars_in: ProfileAvatarLocations,
	}

	impl From<common_settings::ClientSettings> for ClientSettings {
		fn from(client: common_settings::ClientSettings) -> Self {
			ClientSettings {
				animate_eicons: Mutable::new(client.animate_eicons),
				character_name_click_opens: Mutable::new(client.character_name_click_opens),
				clock_format: Mutable::new(client.clock_format),
				display_size: Mutable::new(client.display_size),
				exclude_tags: MutableVec::new_with_values(client.exclude_tags),
				inactivity_timer: Mutable::new(client.inactivity_timer),
				show_avatars_in: ProfileAvatarLocations::from(client.show_avatars_in),
			}
		}
	}

	/// Where profile avatars should be displayed on the client
	#[derive(Debug)]
	pub struct ProfileAvatarLocations {
		pub channels: Mutable<bool>,
		pub character_lists: Mutable<bool>,
		pub console: Mutable<bool>,
		pub private_conversations: Mutable<bool>,
		pub profile_links: Mutable<bool>,
	}

	impl From<common_settings::ProfileAvatarLocations> for ProfileAvatarLocations {
		fn from(profile_avatar_locations: common_settings::ProfileAvatarLocations) -> Self {
			ProfileAvatarLocations {
				channels: Mutable::new(profile_avatar_locations.channels),
				character_lists: Mutable::new(profile_avatar_locations.character_lists),
				console: Mutable::new(profile_avatar_locations.console),
				private_conversations: Mutable::new(profile_avatar_locations.private_conversations),
				profile_links: Mutable::new(profile_avatar_locations.profile_links),
			}
		}
	}

	#[derive(Debug)]
	pub struct LoggerSettings {
		pub log_ads: Mutable<bool>,
		pub log_messages: Mutable<bool>,
		pub storage_method: Mutable<LogStorageMethod>,
	}

	impl From<common_settings::LoggerSettings> for LoggerSettings {
		fn from(logger: common_settings::LoggerSettings) -> Self {
			LoggerSettings {
				log_ads: Mutable::new(logger.log_ads),
				log_messages: Mutable::new(logger.log_messages),
				storage_method: Mutable::new(logger.storage_method),
			}
		}
	}

	#[derive(Debug)]
	pub struct NotificationSettings {
		pub in_app_notifications: Mutable<bool>,
		pub in_app_notification_timer: Mutable<Option<f32>>,
		pub native_notifications: Mutable<bool>,
		pub notify_for: NotificationSets,
		pub word_list: MutableVec<String>,
	}

	impl From<common_settings::NotificationSettings> for NotificationSettings {
		fn from(notifications: common_settings::NotificationSettings) -> Self {
			NotificationSettings {
				in_app_notifications: Mutable::new(notifications.in_app_notifications),
				in_app_notification_timer: Mutable::new(notifications.in_app_notification_timer),
				native_notifications: Mutable::new(notifications.native_notifications),
				notify_for: NotificationSets::from(notifications.notify_for),
				word_list: MutableVec::new_with_values(notifications.word_list),
			}
		}
	}

	/// What notifications should be displayed to the user
	#[derive(Debug)]
	pub struct NotificationSets {
		pub announcements: Mutable<bool>,
		pub mentions: Mutable<bool>,
		pub private_messages: Mutable<bool>,
		pub word_list_entries: Mutable<bool>,
	}

	impl From<common_settings::NotificationSets> for NotificationSets {
		fn from(notification_sets: common_settings::NotificationSets) -> Self {
			NotificationSets {
				announcements: Mutable::new(notification_sets.announcements),
				mentions: Mutable::new(notification_sets.mentions),
				private_messages: Mutable::new(notification_sets.private_messages),
				word_list_entries: Mutable::new(notification_sets.word_list_entries),
			}
		}
	}

	#[derive(Debug)]
	pub struct KeyboardShortcuts {
		pub use_custom_bindings: Mutable<bool>,
		pub movement: Mutable<HashMap<MovementShortcut, String>>,
		pub text_format: Mutable<HashMap<TextFormatShortcut, String>>,
	}

	impl From<common_settings::KeyboardShortcuts> for KeyboardShortcuts {
		fn from(shortcuts: common_settings::KeyboardShortcuts) -> Self {
			let movement = shortcuts.movement.into_iter().map(|(key, value)| (key, value.to_owned())).collect();
			let text_format = shortcuts.text_format.into_iter().map(|(key, value)| (key, value.to_owned())).collect();

			KeyboardShortcuts {
				use_custom_bindings: Mutable::new(shortcuts.use_custom_bindings),
				movement: Mutable::new(movement),
				text_format: Mutable::new(text_format),
			}
		}
	}
}
