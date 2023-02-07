
use bevy::prelude::*;
use naia_bevy_server::{
	events::{AuthorizationEvent, ConnectionEvent, DisconnectionEvent, MessageEvent},
	RoomKey,
	Server,
	ServerAddrs,
};

use super::protocol::{Channels, Protocol};

#[derive(Resource)]
pub struct Global {
	room_key: RoomKey,
}

pub fn sys_start(mut commands: Commands, mut server: Server<Protocol, Channels>) {
	let server_addresses = ServerAddrs::new(
		"0.0.0.0:5322"
			.parse()
			.expect("could not parse Signaling address/port"),
		"0.0.0.0:5323"
			.parse()
			.expect("could not parse WebRTC data address/port"),
		"http://127.0.0.1:5323",
	);

	server.listen(&server_addresses);

	// Resources
	commands.insert_resource(Global {
		room_key: server.make_room().key(),
	});
}

pub fn sys_event_auth(
	mut events: EventReader<AuthorizationEvent<Protocol>>,
	mut server: Server<Protocol, Channels>,
) {
	for event in events.iter() {
		let AuthorizationEvent(ref uid, Protocol::Auth(auth)) = event;
		let user = server.user(uid);

		info!("Received auth request from {} with token '{}'", user.address(), *auth.token);
		//server.reject_connection(uid);
		server.accept_connection(uid);
	}
}

pub fn sys_event_connect<'world, 'state>(
	mut events: EventReader<ConnectionEvent>,
	global: Res<Global>,
	mut server: Server<'world, 'state, Protocol, Channels>,
) {
	for event in events.iter() {
		let ConnectionEvent(uid) = event;
		let mut user = server.user_mut(uid);

		info!("Client connected from {}", user.address());

		user.enter_room(&global.room_key);

		// TODO -- send world state here
	}
}

pub fn sys_event_disconnect(
	mut events: EventReader<DisconnectionEvent>,
) {
	for event in events.iter() {
		let DisconnectionEvent(_, user) = event;
		info!("Client disconnected from {}", user.address);
	}
}

pub fn sys_event_msg(
	mut events: EventReader<MessageEvent<Protocol, Channels>>,
	server: Server<Protocol, Channels>,
) {
	for event in events.iter() {
		let MessageEvent(uid, _, _) = event;
		let user = server.user(uid);

		info!("Received a message from {}", user.address());
	}
}
