use chrono::{DateTime, Utc};
use snowcat_macros::discriminate;
use std::rc::Rc;

pub mod character;

pub struct State {
	
}

pub enum ChannelStateChange {
	
}

pub enum ConversationStateChange {
	
}

#[derive(Debug, Clone)]
#[discriminate]
pub enum Action {
	Bottle {
		choice: Rc<character::Character>,
	},

	Post(String),
	
	Roll {
		dice: Vec<String>,
		results: Vec<f64>,
		total: u128,
	}
}

#[derive(Debug, Clone)]
pub struct ChannelMessage {
	pub author: Rc<character::Character>,
	pub timestamp: DateTime<Utc>,
	pub content: MessageType,
}

#[derive(Debug, Clone)]
#[discriminate]
pub enum MessageType {
	Action(Action),
	Ad(String),
	Message(String),
}
