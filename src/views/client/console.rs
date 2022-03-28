use crate::App;
use crate::state::Notification;
use crate::styles::{NO_USER_SELECT, component};
use crate::views::client::{
	VIEW_DESCRIPTION_MINIMISED, VIEW_HEADER, VIEW_TITLE, VIEW_TITLE_BOX, ClientScreenState, profile_picture_url,
};
use dominator::{Dom, html};
use futures_signals::signal::{always, SignalExt};
use futures_signals::signal_vec::SignalVecExt;
use std::rc::Rc;
use std::sync::Arc;

pub fn render(app: Arc<App>, state: Rc<ClientScreenState>) -> Dom {
	html!("div", {
		.class(&*component::view::CONTAINER)

		// Console header
		.child(html!("header", {
			.class(&*NO_USER_SELECT)
			.class(&*VIEW_HEADER)

			.child(html!("h1", {
				.class(&*VIEW_TITLE)
				.class(&*VIEW_TITLE_BOX)
				.text("Console")
			}))
			
			.child(html!("p", {
				.class(&*VIEW_DESCRIPTION_MINIMISED)
				.text("System notifications and staff broadcasts will appear here.")
			}))
		}))
		// End console header

		// Notification display
		.child(html!("main", {
			.class(&*component::channel::CHANNEL)
			.class(&*component::channel::CONSOLE)

			.child(html!("div", {
				.class(&*component::channel::MESSAGE_AREA)
				.class(&*component::channel::notification::NOTIFICATION_AREA)

				.children_signal_vec(state.system_signal_vec().map(move |message| {
					html!("div", {
						.class(&*component::channel::notification::NOTIFICATION)

						.child(html!("time", {
							.attr("datetime", &message.timestamp.format("%+").to_string())
							.class(&*component::channel::message::TIME)
							.class(&*NO_USER_SELECT)
							.text(&message.timestamp.format("%R").to_string())
						}))

						.child(html!("div", {
							.apply(match &message.data {
								Notification::Broadcast { content, sender } => notifications::broadcast(&app, sender, content),
								Notification::Mention { channel, message } => notifications::mention(&app, channel, message),
							})
						}))
					})
				}))
			}))
		}))
		// End notification display
	})
}

mod notifications {
    use crate::App;
    use crate::state::PublicChannel;
    use crate::views::client::clickable_profile_link;
	use dominator::{DomBuilder, apply_methods};
    use snowcat_common::state::ChannelMessage;
    use snowcat_common::state::character::Character;
    use web_sys::HtmlElement;
    use std::rc::Rc;
    use std::sync::Arc;

	pub fn broadcast<'post>(app: &'post Arc<App>, sender: &'post Rc<Character>, content: &'post str)
	-> Box<dyn FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> + 'post> {
		let show_avatars_mutable = &*app.data.settings.client.show_avatars_in.console;

		let closure = move |dom: DomBuilder<HtmlElement>| apply_methods!(dom, {
			.child(clickable_profile_link(&sender, show_avatars_mutable.clone()))
			.text(" has broadcast: ")
			.text(content)
		});

		Box::new(closure)
	}

	pub fn mention<'post>(_app: &'post Arc<App>, _channel: &'post Rc<PublicChannel>, _message: &'post Rc<ChannelMessage>)
	-> Box<dyn FnOnce(DomBuilder<HtmlElement>) -> DomBuilder<HtmlElement> + 'post> {
		let closure = move |_| {
			todo!("show mentions")
		};

		Box::new(closure)
	}
}
