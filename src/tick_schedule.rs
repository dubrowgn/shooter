use bevy::ecs::schedule::{ScheduleLabel, ExecutorKind};
use bevy::prelude::*;
use std::time::Duration;
use std::marker::Copy;

#[derive(Clone, Copy, Resource)]
pub struct TickConfig {
	pub budget: Duration,
	pub interval: Duration,
}

#[derive(Clone, Copy, Resource)]
pub struct TickState {
	pub steps: usize,
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TickSchedule {
	Network,
	PreTicks,
	Ticks,
	PostTicks,
}

pub struct TickPlugin;

impl Plugin for TickPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_schedule(TickSchedule::PreTicks, multi_thread_schedule())
			.add_schedule(TickSchedule::Ticks, multi_thread_schedule())
			.add_schedule(TickSchedule::PostTicks, multi_thread_schedule());
	}
}

fn make_schedule(kind: ExecutorKind) -> Schedule {
	let mut sched = Schedule::new();
	sched.set_executor_kind(kind);
	sched
}

pub fn multi_thread_schedule() -> Schedule {
	make_schedule(ExecutorKind::MultiThreaded)
}

pub fn single_thread_schedule() -> Schedule {
	make_schedule(ExecutorKind::SingleThreaded)
}
