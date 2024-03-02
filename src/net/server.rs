use std::{
	thread,
	time::Instant,
};
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
};
use super::{
	config::{self, PlayerCommandChannel, TICK_INTERVAL},
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
pub struct Global {
	room_key: RoomKey,
}

pub fn sys_start(mut commands: Commands, mut server: Server) {
	let addr = udp_sock_addr((127, 0, 0, 1), 5323);
	let sock = udp::Socket::new(&addr, None);

	println!("Starting server on {}...", addr);
	server.listen(sock);

	// Resources
	commands.insert_resource(Global {
		room_key: server.make_room().key(),
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
	global: Res<Global>,
	mut server: Server,
) {
	for ConnectEvent(uid) in events.read() {
		let mut user = server.user_mut(uid);

		println!("Client connected from {}", user.address());

		user.enter_room(&global.room_key);

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
) {
	for t in ticks.read() {
		let mut messages = server.receive_tick_buffer_messages(&t.0);
		for (uid, msg) in messages.read::<PlayerCommandChannel, msg::Input>() {
			let user = server.user(&uid);
			println!("Received {:?} from {}", msg, user.address());
		}
	}
}
