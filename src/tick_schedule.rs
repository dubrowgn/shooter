use bevy::ecs::schedule::{ScheduleLabel, ExecutorKind};
use bevy::prelude::*;
use std::time::Duration;
use std::marker::Copy;

#[derive(Clone, Copy, Resource)]
pub struct TickConfig {
	pub budget: Duration,
	pub interval: Duration,
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
pub enum TickSchedule {
	InputCollect,
	InputSend,
	Network,
	PreTicks,
	Tick,
	PostTicks,
}

pub struct TickPlugin;

impl Plugin for TickPlugin {
	fn build(&self, app: &mut App) {
		app
			.add_schedule(multi_thread_schedule(TickSchedule::InputCollect))
			.add_schedule(multi_thread_schedule(TickSchedule::PreTicks))
			.add_schedule(multi_thread_schedule(TickSchedule::Tick))
			.add_schedule(multi_thread_schedule(TickSchedule::PostTicks));
	}
}

fn make_schedule(kind: ExecutorKind, label: impl ScheduleLabel) -> Schedule {
	let mut sched = Schedule::new(label);
	sched.set_executor_kind(kind);
	sched
}

pub fn multi_thread_schedule(label: impl ScheduleLabel) -> Schedule {
	make_schedule(ExecutorKind::MultiThreaded, label)
}

pub fn single_thread_schedule(label: impl ScheduleLabel) -> Schedule {
	make_schedule(ExecutorKind::SingleThreaded, label)
}
