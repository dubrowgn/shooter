use bevy::prelude::*;
use naia_bevy_client::{
	Client,
	ClientConfig,
	events::{ConnectEvent, DisconnectEvent, RejectEvent, ServerTickEvent},
	Plugin as NaiaClientPlugin,
	transport::udp,
};
use super::{
	config,
	msg::Auth,
	peer::{sys_run_tick_schedules, udp_sock_addr},
};
use crate::tick_schedule::{TickSchedule, single_thread_schedule, TickState};

pub struct NetClientPlugin;

impl Plugin for NetClientPlugin {
	fn build(&self, app: &mut App) {
		app
			.insert_resource(TickState { steps: 0 })
			.add_plugins(NaiaClientPlugin::new(
				ClientConfig::default(),
				config::global_avg(), // TODO -- use actual TickConfig.interval
			))
			.add_schedule(TickSchedule::Network, single_thread_schedule())
			.add_systems(TickSchedule::Network, (
				sys_event_connect,
				sys_event_disconnect,
				sys_event_reject,
			))
			.add_systems(TickSchedule::PreTicks, (
				sys_consume_tick_events,
			))
			.add_systems(Update, sys_run_tick_schedules)
			.add_systems(Startup, sys_connect);
	}
}

fn sys_consume_tick_events(
	mut state: ResMut<TickState>,
	mut ticks: EventReader<ServerTickEvent>,
) {
	state.steps += ticks.len();
	ticks.clear();
}

pub fn sys_connect(mut client: Client) {
	let addr = udp_sock_addr((127, 0, 0, 1), 5323);
	let sock = udp::Socket::new(&addr, None);

	client.auth(Auth::new("token-content"));
	client.connect(sock);
}

pub fn sys_event_connect(mut events: EventReader<ConnectEvent>, client: Client) {
	for _event in events.iter() {
		if let Ok(server_address) = client.server_address() {
			info!("Connected to server {}", server_address);
		}
	}
}

pub fn sys_event_disconnect(mut events: EventReader<DisconnectEvent>, client: Client) {
	for _event in events.iter() {
		if let Ok(server_address) = client.server_address() {
			info!("Disconnected from server {}", server_address);
		}
	}
}

pub fn sys_event_reject(mut events: EventReader<RejectEvent>, client: Client) {
	for _event in events.iter() {
		if let Ok(server_address) = client.server_address() {
			info!("Rejected by server {}", server_address);
		}
	}
}
