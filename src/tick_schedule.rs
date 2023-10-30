use bevy::ecs::schedule::{ScheduleLabel, ExecutorKind};
use bevy::prelude::*;
use std::time::{Duration, Instant};
use std::marker::Copy;


// zero or more ticks per frame
//	stop ticks if time budget spent
// (once) clean up events if at least one tick happened
// (once) interpolate remaining acc time

#[derive(Clone, Copy, Resource)]
pub struct TickInfo {
	pub acc: Duration,
	pub budget: Duration,
	pub step: Duration,
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TickSchedule {
	PreTicks,
	Ticks,
	PostTicks,
}

fn sys_run_tick_schedules(world: &mut World) {
	world.run_schedule(TickSchedule::PreTicks);

	let write_acc = |world: &mut World, acc: Duration|
	world.get_resource_mut::<TickInfo>()
		.expect("Missing TickInfo resource")
		.acc = acc;

	let info = world
		.get_resource::<TickInfo>()
		.expect("Missing TickInfo resource");
	let mut acc = info.acc;
	let budget = info.budget;
	let step = info.step;

	acc += world.get_resource_mut::<Time>()
		.expect("Missing Time resource")
		.delta();

	let start = Instant::now();
	while acc > step && Instant::now() - start <= budget {
		acc -= step;
		write_acc(world, acc);

		world.run_schedule(TickSchedule::Ticks);
	}

	write_acc(world, acc);

	world.run_schedule(TickSchedule::PostTicks);
}

pub struct TickPlugin;

impl Plugin for TickPlugin {
    fn build(&self, app: &mut App) {
		app
			.add_schedule(TickSchedule::PreTicks, multi_thread_schedule())
			.add_schedule(TickSchedule::Ticks, multi_thread_schedule())
			.add_schedule(TickSchedule::PostTicks, multi_thread_schedule())
			.add_systems(Update, sys_run_tick_schedules);
    }
}

fn multi_thread_schedule() -> Schedule {
	let mut sched = Schedule::new();
	sched.set_executor_kind(ExecutorKind::MultiThreaded);
	sched
}
