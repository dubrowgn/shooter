use bevy::prelude::*;
use crate::{
	input::interpret::PlayerInput,
	tick_schedule::{
		TickSchedule,
		single_thread_schedule,
	},
	player::Player,
};
use naia_bevy_client::{
	events::{
		ClientTickEvent,
		ConnectEvent,
		DisconnectEvent,
		ErrorEvent,
		MessageEvents,
		RejectEvent,
		ServerTickEvent,
	},
	transport::udp,
	Client,
	ClientConfig,
	Plugin as NaiaClientPlugin,
};
use std::{
	collections::HashMap,
	f32::consts::TAU,
};
use super::{
	config::{
		self,
		CmdStreamChannel,
		InputSrcChannel,
	}, msg, peer::*
};

pub struct NetClientPlugin;

impl Plugin for NetClientPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(TickState::default())
			.add_plugins(NaiaClientPlugin::new(
				ClientConfig::default(),
				config::global_avg(),
			))
			.insert_resource(ClientContext::default())
			.add_schedule(single_thread_schedule(TickSchedule::Network))
			.add_systems(TickSchedule::Network, (
				sys_event_connect,
				sys_event_disconnect,
				sys_event_error,
				sys_event_reject,
			))
			.add_systems(TickSchedule::PreTicks, (
				sys_consume_tick_events,
			))
			.add_schedule(single_thread_schedule(TickSchedule::InputSend))
			.add_systems(TickSchedule::InputSend, (
				sys_send_input,
			))
			.add_systems(Update, sys_run_tick_schedules)
			.add_systems(Startup, sys_connect);
	}
}

#[derive(Default, Resource)]
pub struct ClientContext {
    pub client_entities: HashMap<u32, Entity>,
}

fn sys_consume_tick_events(
	mut state: ResMut<TickState>,
	mut ticks: EventReader<ServerTickEvent>,
) {
	state.ticks_pending += ticks.len();
	for t in ticks.read() {
		state.cur_tick = t.0;
		break;
	}
	ticks.clear();
}

pub fn sys_connect(mut client: Client) {
	let addr = udp_sock_addr((127, 0, 0, 1), 5323);
	let sock = udp::Socket::new(&addr, None);

	info!("Connecting to server @ {}...", addr);
	client.auth(msg::Auth::new("token-content"));
	client.connect(sock);
}

pub fn sys_event_connect(
	client: Client,
	mut events: EventReader<ConnectEvent>,
) {
	for _event in events.read() {
		if let Ok(server_address) = client.server_address() {
			info!("Connected to server {}", server_address);
		}
	}
}

pub fn sys_event_disconnect(mut events: EventReader<DisconnectEvent>, client: Client) {
	for _event in events.read() {
		if let Ok(server_address) = client.server_address() {
			info!("Disconnected from server {}", server_address);
		}
	}
}

pub fn sys_event_error(mut events: EventReader<ErrorEvent>) {
	for ErrorEvent(err) in events.read() {
		error!("{}", err);
	}
}

pub fn sys_event_msg(
	mut ctx: ResMut<ClientContext>,
	mut event_sets: EventReader<MessageEvents>,
	players: Query<Entity, With<Player>>,
) {
    for events in event_sets.read() {
		info!("Received message events!");
		for msg in events.read::<CmdStreamChannel, msg::Assign>() {
			info!("sys_event_msg:{:?}", msg);

			let player_ent = players.single();
			ctx.client_entities.insert(msg.client_id, player_ent);
		}
		for msg in events.read::<CmdStreamChannel, msg::InputRepl>() {
			//let player_ent = ctx.client_entities[&msg.client_id];
			//players.get(player_ent).
			info!("sys_event_msg:{:?}", msg);
		}
	}
}

pub fn sys_event_reject(mut events: EventReader<RejectEvent>, client: Client) {
	for _event in events.read() {
		if let Ok(server_address) = client.server_address() {
			info!("Rejected by server {}", server_address);
		}
	}
}

pub fn sys_send_input(
	mut client: Client,
	input: Res<PlayerInput>,
	mut ticks: EventReader<ClientTickEvent>,
) {
	for t in ticks.read() {
		let cursor: Vec2 = Vec2::from_angle(input.face_turns * TAU);
		let msg = msg::Input {
			cursor_dx: cursor.x,
			cursor_dy: cursor.y,
			velocity_x: input.dir.x,
			velocity_y: input.dir.y,
			primary: input.primary,
		};
		//info!("sys_xmit_input {:?}: {:?}", state.cur_tick, msg);
		client.send_tick_buffer_message::<InputSrcChannel, msg::Input>(&t.0, &msg);
	}
}
