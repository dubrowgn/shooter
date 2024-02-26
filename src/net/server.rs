
use bevy::prelude::*;
use naia_bevy_server::{
	events::{AuthEvents, ConnectEvent, DisconnectEvent, ErrorEvent, TickEvent},
	Plugin as NaiaServerPlugin,
	RoomKey,
	Server,
	ServerConfig,
	transport::udp,
};
use super::{
	config::{self, PlayerCommandChannel},
	msg,
	peer::*,
};
use crate::tick_schedule::{TickSchedule, single_thread_schedule};

pub struct NetServerPlugin;

impl Plugin for NetServerPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(TickState::default())
			.add_plugins(NaiaServerPlugin::new(
				ServerConfig::default(),
				config::global_avg(), // TODO -- use actual TickConfig.interval
			))
			.add_schedule(single_thread_schedule(TickSchedule::Network))
			.add_systems(TickSchedule::Network, (
				sys_event_auth,
				sys_event_connect,
				sys_event_disconnect,
				sys_event_error,
			))
			.add_systems(TickSchedule::Tick, sys_event_msg)
			.add_schedule(single_thread_schedule(TickSchedule::InputSend))
			.add_systems(TickSchedule::PreTicks, sys_consume_tick_events)
			.add_systems(Update, sys_run_tick_schedules)
			.add_systems(Startup, sys_start);
	}
}

fn sys_consume_tick_events(
	mut state: ResMut<TickState>,
	mut ticks: EventReader<TickEvent>,
) {
	state.ticks_pending += ticks.len();
	for t in ticks.read() {
		state.cur_tick = t.0;
		break;
	}
	ticks.clear();
}

#[derive(Resource)]
pub struct Global {
	room_key: RoomKey,
}

pub fn sys_start(mut commands: Commands, mut server: Server) {
	let addr = udp_sock_addr((127, 0, 0, 1), 5323);
	let sock = udp::Socket::new(&addr, None);

	server.listen(sock);

	// Resources
	commands.insert_resource(Global {
		room_key: server.make_room().key(),
	});
}

pub fn sys_event_auth(
	mut events: EventReader<AuthEvents>,
	mut server: Server,
) {
	for events in events.read() {
		for (uid, auth) in events.read::<msg::Auth>() {
			let user = server.user(&uid);

			info!("Received auth request from {} with token '{}'", user.address(), auth.token);
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

		info!("Client connected from {}", user.address());

		user.enter_room(&global.room_key);

		// TODO -- send world state here
	}
}

pub fn sys_event_disconnect(
	mut events: EventReader<DisconnectEvent>,
) {
	for DisconnectEvent(_uid, user) in events.read() {
		info!("Client disconnected from {}", user.address);
	}
}

pub fn sys_event_error(
	mut events: EventReader<ErrorEvent>,
) {
	for ErrorEvent(err) in events.read() {
		error!("{}", err);
	}
}

pub fn sys_event_msg(
	mut server: Server,
	tick_state: Res<TickState>,
) {
	let mut messages = server.receive_tick_buffer_messages(&tick_state.cur_tick);
	for (uid, msg) in messages.read::<PlayerCommandChannel, msg::Input>() {
		let user = server.user(&uid);
		info!("Received {:?} from {}", msg, user.address());
	}
}
