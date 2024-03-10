use bevy::{
	app::{RunMode, ScheduleRunnerPlugin},
	prelude::*
};
use naia_bevy_server::{
	events::{AuthEvents, ConnectEvent, DisconnectEvent, ErrorEvent, TickEvent},
	Plugin as NaiaServerPlugin,
	RoomKey,
	Server,
	ServerConfig,
	transport::udp,
	UserKey,
};
use std::{
	collections::HashMap,
	thread,
	time::Instant
};
use crate::net::config::CmdStreamChannel;

use super::{
	config::{self, InputSrcChannel, TICK_INTERVAL},
	msg,
	peer::*,
};

pub struct NetServerPlugin;

impl Plugin for NetServerPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_plugins((
				ScheduleRunnerPlugin{ run_mode: RunMode::Loop { wait: None} },
				NaiaServerPlugin::new(
					ServerConfig::default(),
					config::global_avg(),
				),
			))
			.insert_resource(SleepContext{ frame_start: Instant::now() })
			.add_systems(Update, (
				sys_event_auth,
				sys_event_connect,
				sys_event_disconnect,
				sys_event_error,
				sys_event_msg,
				sys_sleep,
			).chain())
			.add_systems(Startup, sys_start);
	}
}

#[derive(Resource)]
pub struct ServerContext {
	pub room: RoomKey,
    pub client_ids: HashMap<UserKey, u32>,
	pub next_client_id: u32,
}

pub fn sys_start(mut commands: Commands, mut server: Server) {
	let addr = udp_sock_addr((127, 0, 0, 1), 5323);
	let sock = udp::Socket::new(&addr, None);

	println!("Starting server on {}...", addr);
	server.listen(sock);

	// Resources
	commands.insert_resource(ServerContext {
		room: server.make_room().key(),
		client_ids: HashMap::new(),
		next_client_id: 1,
	});
}

#[derive(Resource)]
pub struct SleepContext {
	frame_start: Instant,
}

fn sys_sleep(mut ctx: ResMut<SleepContext>) {
	let frame_time = Instant::now() - ctx.frame_start;

	if frame_time < TICK_INTERVAL {
		thread::sleep(TICK_INTERVAL - frame_time);
	}

	ctx.frame_start += TICK_INTERVAL;
}

pub fn sys_event_auth(
	mut events: EventReader<AuthEvents>,
	mut server: Server,
) {
	for events in events.read() {
		for (uid, auth) in events.read::<msg::Auth>() {
			let user = server.user(&uid);

			println!("Received auth request from {} with token '{}'", user.address(), auth.token);
			//server.reject_connection(uid);
			server.accept_connection(&uid);
		}
	}
}

pub fn sys_event_connect<'world, 'state>(
	mut events: EventReader<ConnectEvent>,
	mut ctx: ResMut<ServerContext>,
	mut server: Server,
) {
	for ConnectEvent(uid) in events.read() {
		// assign client Id
		let client_id = ctx.next_client_id;
		ctx.next_client_id = client_id.wrapping_add(1);
		ctx.client_ids.insert(*uid, client_id);

		// send assignment
		let msg = msg::Assign { client_id };
		server.send_message::<CmdStreamChannel, msg::Assign>(uid, &msg);

		let mut user = server.user_mut(uid);
		println!("Client connected from {}", user.address());
		user.enter_room(&ctx.room);

		// TODO -- send world state here
	}
}

pub fn sys_event_disconnect(
	mut events: EventReader<DisconnectEvent>,
) {
	for DisconnectEvent(_uid, user) in events.read() {
		println!("Client disconnected from {}", user.address);
	}
}

pub fn sys_event_error(
	mut events: EventReader<ErrorEvent>,
) {
	for ErrorEvent(err) in events.read() {
		println!("{}", err);
	}
}

pub fn sys_event_msg(
	mut ticks: EventReader<TickEvent>,
	mut server: Server,
	ctx: Res<ServerContext>,
) {
	for t in ticks.read() {
		let mut messages = server.receive_tick_buffer_messages(&t.0);
		for (uid, msg) in messages.read::<InputSrcChannel, msg::Input>() {
			let msg = msg::InputRepl::new(ctx.client_ids[&uid], &msg);
			//info!("sys_event_msg {:?}: {:?}", state.cur_tick, msg);
			server.broadcast_message::<CmdStreamChannel, msg::InputRepl>(&msg);
		}
	}
}
