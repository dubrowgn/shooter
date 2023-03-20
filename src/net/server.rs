
use bevy::prelude::*;
use naia_bevy_server::{
	events::{AuthEvents, ConnectEvent, DisconnectEvent, ErrorEvent, MessageEvents, TickEvent},
	Plugin as NaiaServerPlugin,
	RoomKey,
	Server,
	ServerConfig,
	transport::udp,
};
use naia_bevy_shared::Message;
use super::{
	config,
	msg::Auth,
	peer::{sys_run_tick_schedules, udp_sock_addr},
};
use crate::tick_schedule::{TickState, TickSchedule, single_thread_schedule};

pub struct NetServerPlugin;

impl Plugin for NetServerPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(TickState { steps: 0 })
			.add_plugins(NaiaServerPlugin::new(
				ServerConfig::default(),
				config::global_avg(), // TODO -- use actual TickConfig.interval
			))
			.add_schedule(TickSchedule::Network, single_thread_schedule())
			.add_systems(TickSchedule::Network, (
				sys_event_auth,
				sys_event_connect,
				sys_event_disconnect,
				sys_event_error,
			))
			.add_systems(TickSchedule::PreTicks, sys_consume_tick_events)
			.add_systems(Update, sys_run_tick_schedules)
			.add_systems(Startup, sys_start);
	}
}

fn sys_consume_tick_events(
	mut state: ResMut<TickState>,
	mut ticks: EventReader<TickEvent>,
) {
	state.steps += ticks.len();
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
	for events in events.iter() {
		for (uid, auth) in events.read::<Auth>() {
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
	for ConnectEvent(uid) in events.iter() {
		let mut user = server.user_mut(uid);

		info!("Client connected from {}", user.address());

		user.enter_room(&global.room_key);

		// TODO -- send world state here
	}
}

pub fn sys_event_disconnect(
	mut events: EventReader<DisconnectEvent>,
) {
	for DisconnectEvent(_uid, user) in events.iter() {
		info!("Client disconnected from {}", user.address);
	}
}

pub fn sys_event_error(
	mut events: EventReader<ErrorEvent>,
) {
	for ErrorEvent(err) in events.iter() {
		error!("{}", err);
	}
}

/*
pub fn sys_event_msg(
	mut events: EventReader<MessageEvents>,
	server: Server,
) {
	for events in events.iter() {
		for (uid, msg) in events.read::<Message>() {
			let user = server.user(&uid);
			info!("Received a {} message from {}", msg.name(), user.address());
		}
	}
}
*/
