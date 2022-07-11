use crate::App;
use crate::state::PublicChannel;
use crate::styles::{classname, icon};
use crate::views::client::{ClientScreenState, ClientView};
use dominator::events::Click;
use dominator::{Dom, EventOptions, clone, html};
use futures_signals::map_ref;
use futures_signals::signal::{Signal, SignalExt};
use futures_signals::signal_vec::SignalVecExt;
use std::rc::Rc;
use std::sync::Arc;

pub fn channels_list(_app: Arc<App>, state: Rc<ClientScreenState>) -> Dom {
	html!("div", {
		.attr("aria-live", "polite")
		.attr("aria-role", "list")
		.class(&*super::NAVIGATION_CONTAINER)
		.child(super::menus::view_console_entry(&state))

		.children_signal_vec(state.channels_signal_vec().map(clone!(state => move |channel| {
			channel_list_item(&state, &channel, map_ref! {
				let name = channel.name.signal_cloned(),
				let count = channel.characters.signal_vec_cloned().len() =>
					format!("{} ({} characters)", name, count)
			})
		})))
	})
}

fn channel_list_item<Title>(state: &Rc<ClientScreenState>, channel: &Rc<PublicChannel>, title: Title) -> Dom
where Title: Signal<Item = String> + 'static {
	html!("div", {
		.attr("aria-role", "button")
		.attr_signal("title", title)
		.class(&*super::VIEW_SWITCH_BUTTON)

		.class_signal(
			&*super::VIEW_SWITCH_BUTTON_ACTIVE,
			is_active_channel_signal(state.client_view.signal_cloned(), channel.clone())
		)

		//- icon
		.child(html!("div", {
			.attr("aria-hidden", "true")
			.class(&*super::VIEW_SWITCH_BUTTON_ICON)
			.text("#")
		}))
		//- icon

		//- title and character count
		.child(html!("div", {
			.attr("aria-hidden", "true")
			.class(&*super::VIEW_SWITCH_BUTTON_TEXT_CONTAINER)

			.child(html!("span", {
				.class(&*super::VIEW_SWITCH_BUTTON_TEXT)
				.text_signal(channel.name.signal_cloned())
			}))

			.child(html!("span", {
				.class(&*super::VIEW_SWITCH_BUTTON_SUBTEXT)
				.text("Characters: ")

				.text_signal(
					channel.characters.signal_vec_cloned()
						.len()
						.map(|len| len.to_string())
				)
			}))
		}))
		//- title and character count

		//- options
		.child(html!("div", {
			.class(&*super::VIEW_SWITCH_BUTTON_OPTIONS_BOX)
			
			.child(html!("div", {
				.attr_signal("title", channel.name.signal_ref(|name| {
					format!("Close {name}")
				}))

				.class(&*super::VIEW_SWITCH_BUTTON_OPTION)
				.class(&*classname::ICON)

				.event_with_options(
					&EventOptions {bubbles: true, preventable: true},
					on_click_close_channel(state, channel)
				)

				.text(&*icon::CLOSE_GLYPH)
			}))

			.child(html!("span", {
				.attr_signal("title", map_ref! {
					let is_pinned = channel.pinned.signal(),
					let name = channel.name.signal_cloned() => {
						let action = if *is_pinned { "Unpin" } else { "Pin" };

						format!("{action} {name}")
					}
				})

				.class_signal(&*super::VIEW_SWITCH_BUTTON_ACTIVE, channel.pinned.signal())
				.class(&*super::VIEW_SWITCH_BUTTON_OPTION)
				.class(&*classname::ICON)

				.event_with_options(
					&EventOptions { bubbles: true, preventable: true },
					on_click_toggle_pin(state, channel)
				)

				.text_signal(channel.pinned.signal_ref(|pinned| match pinned {
					true => &*icon::PINNED_GLYPH,
					false => &*icon::PIN_GLYPH,
				}))
			}))
		}))
		//- options

		.event_with_options(
			&EventOptions::bubbles(),
			on_click_set_active_channel(state, channel)
		)
	})
}

fn is_active_channel_signal<View>(view_signal: View, channel: Rc<PublicChannel>) -> impl Signal<Item = bool>
where View: Signal<Item = ClientView> {
	view_signal.map(move |view| match view {
		ClientView::PublicChannel(view_state) => Rc::ptr_eq(&view_state.channel(), &channel),
		_ => false,
	})
}

// Handlers
fn on_click_close_channel(state: &Rc<ClientScreenState>, channel: &Rc<PublicChannel>) -> impl FnMut(Click) {
	clone!(state, channel => move |event| {
		event.stop_propagation();

		let mut channels = state.channels_mut();
		let current_view = state.client_view.lock_mut();

		let channel_index = match channels.iter().position(|current_channel| Rc::ptr_eq(&channel, current_channel)) {
			Some(index) => index,
			None => return, // somehow we managed to double-issue a channel close request.
		};

		channels.remove(channel_index);

		// If we have closed the currently active channel, try to move the
		// user to the next available one or, failing that, the system
		// message view.
		if let ClientView::PublicChannel(ref view_state) = *current_view {
			if Rc::ptr_eq(&view_state.channel(), &channel) {
				let next_channel = channels.get(channel_index).or_else(|| channels.last());
				let view = match next_channel {
					Some(channel) => ClientView::PublicChannel(channel.view_state().clone()),
					None => ClientView::Console,
				};

				drop(current_view);

				state.client_view.set(view);
			}
		}
	})
}

fn on_click_set_active_channel(state: &Rc<ClientScreenState>, new_channel: &Rc<PublicChannel>) -> impl FnMut(Click) {
	clone!(state, new_channel => move |_| {
		state.client_view.set_if(
			ClientView::PublicChannel(new_channel.view_state().clone()),
			|current_state, new_state| match (current_state, new_state) {
				(ClientView::PublicChannel(ref current_state), ClientView::PublicChannel(ref new_state))
					if Rc::ptr_eq(current_state, new_state) => false,

				_ => true
			}
		);
	})
}

fn on_click_toggle_pin(_state: &Rc<ClientScreenState>, _channel: &Rc<PublicChannel>) -> impl FnMut(Click) {
	|_click| todo!("on click toggle pin")
}
