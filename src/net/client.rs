use bevy::prelude::*;
use naia_bevy_client::Client;
use super::{
	msg::Auth,
	protocol::{Channels, Protocol},
};

pub fn sys_connect(mut client: Client<Protocol, Channels>) {
	client.auth(Auth::new("token-content"));
	client.connect("http://127.0.0.1:5322");
}

pub fn sys_event_connect(client: Client<Protocol, Channels>) {
	if let Ok(server_address) = client.server_address() {
		info!("Connected to server {}", server_address);
	}
}

pub fn sys_event_disconnect(client: Client<Protocol, Channels>) {
	if let Ok(server_address) = client.server_address() {
		info!("Disconnected from server {}", server_address);
	}
}

pub fn sys_event_reject(client: Client<Protocol, Channels>) {
	if let Ok(server_address) = client.server_address() {
		info!("Rejected by server {}", server_address);
	}
}
