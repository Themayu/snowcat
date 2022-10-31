#![cfg_attr(
	all(not(debug_assertions), target_os = "windows"),
	windows_subsystem = "windows"
)]

mod commands;

use anyhow::Context;
use reqwest::Client as HttpClient;
use snowcat::api::headers;
use snowcat::client::Client;
use snowcat::state::prelude::*;

fn main() -> Result<(), anyhow::Error> {
	let State {
		channel_cache,
		character_cache,
		client,
	} = initial_state()?;

	Ok(tauri::Builder::default()
		.manage(channel_cache)
		.manage(character_cache)
		.manage(client)
		.invoke_handler(commands::command_handler())
		.run(tauri::generate_context!())?)
}

struct State {
	channel_cache: ChannelCache,
	character_cache: CharacterCache,
	client: Client,
}

fn initial_state() -> Result<State, anyhow::Error> {
	let http = HttpClient::builder()
		.http1_title_case_headers()
		.user_agent(headers::USER_AGENT_VALUE)
		.build()
		.context("Failed to build HTTP client")?;

	Ok(State {
		channel_cache: ChannelCache::default(),
		character_cache: CharacterCache::default(),
		client: Client::new(http),
	})
}
