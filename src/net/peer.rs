use bevy::{
	ecs::system::Resource,
	prelude::*,
};
use naia_bevy_shared::Tick;
use std::{
	net::{IpAddr, SocketAddr, Ipv4Addr},
	time::Instant,
};
use crate::tick_schedule::{TickSchedule, TickConfig};

#[derive(Clone, Copy, Default, Resource)]
pub struct TickState {
	pub cur_tick: Tick,
	pub ticks_pending: usize,
}

pub fn sys_run_tick_schedules(world: &mut World) {
	world.run_schedule(TickSchedule::Network);
	world.run_schedule(TickSchedule::PreTicks);

	let budget = world.get_resource::<TickConfig>()
		.expect("Missing TickConfig resource")
		.budget;

	world.resource_scope(|world: &mut World, mut state: Mut<TickState>| {
		let start = Instant::now();
		while state.ticks_pending > 0 && Instant::now() - start <= budget {
			state.ticks_pending -= 1;

			world.insert_resource(*state); // FIXME ???
			world.run_schedule(TickSchedule::InputCollect);
			world.run_schedule(TickSchedule::InputSend);
			world.run_schedule(TickSchedule::Tick);
			world.remove_resource::<TickState>();

			state.cur_tick = state.cur_tick.wrapping_add(1);
		}
	});

	world.run_schedule(TickSchedule::PostTicks);
}

pub fn udp_sock_addr(addr: (u8, u8, u8, u8), port: u16) -> SocketAddr {
	let addr = Ipv4Addr::new(addr.0, addr.1, addr.2, addr.3);

	SocketAddr::new(IpAddr::V4(addr), port)
}
