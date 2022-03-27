use crate::state::Conversation;
use std::rc::{Weak, Rc};

#[derive(Debug, Clone)]
pub struct ViewState {
	conversation: Weak<Conversation>
}

impl ViewState {
	pub fn new(conversation: &Conversation, weak_ref: &Weak<Conversation>) -> ViewState {
		ViewState {
			conversation: weak_ref.clone(),
		}
	}

	pub fn conversation(&self) -> Rc<Conversation> {
		self.conversation.upgrade().expect("a surviving reference")
	}
}
