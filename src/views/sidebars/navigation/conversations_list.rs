use crate::App;
use crate::state::Conversation;
use crate::views::client::ClientScreenState;
use dominator::{Dom, clone, html};
use futures_signals::signal::{Signal, always};
use futures_signals::signal_vec::SignalVecExt;
use std::rc::Rc;
use std::sync::Arc;

pub fn conversations_list(_app: Arc<App>, state: Rc<ClientScreenState>) -> Dom {
	html!("div", {
		.class(&*super::NAVIGATION_CONTAINER)
		.child(super::view_console_button(&state))

		.children_signal_vec(state.conversations_signal_vec().map(clone!(state => move |conversation| {
			conversation_list_entry(&state, &conversation, always(conversation.character.name.to_owned()))
		})))
	})
}

fn conversation_list_entry<Title>(_state: &Rc<ClientScreenState>, _conversation: &Rc<Conversation>, _title: Title) -> Dom
where Title: Signal<Item = String> + 'static {
	// html!("div", {
	// 	.text_signal(title)
	// })

	todo!("render private conversation list entries")
}
