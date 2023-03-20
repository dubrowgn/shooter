use bevy::prelude::{Mut, World};
use std::{time::Instant, net::{IpAddr, SocketAddr, Ipv4Addr}};
use crate::tick_schedule::{TickSchedule, TickConfig, TickState};

pub fn sys_run_tick_schedules(world: &mut World) {
	world.run_schedule(TickSchedule::Network);
	world.run_schedule(TickSchedule::PreTicks);

	let budget = world.get_resource::<TickConfig>()
		.expect("Missing TickConfig resource")
		.budget;

	world.resource_scope(|world: &mut World, mut state: Mut<TickState>| {
		let start = Instant::now();
		while state.steps > 0 && Instant::now() - start <= budget {
			state.steps -= 1;
			world.run_schedule(TickSchedule::Ticks);
		}
	});

	world.run_schedule(TickSchedule::PostTicks);
}

pub fn udp_sock_addr(addr: (u8, u8, u8, u8), port: u16) -> SocketAddr {
	let addr = Ipv4Addr::new(addr.0, addr.1, addr.2, addr.3);

	SocketAddr::new(IpAddr::V4(addr), port)
}
