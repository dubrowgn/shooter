use bevy::prelude::*;
use std::time::{Duration, Instant};
use std::marker::Copy;

#[derive(Clone, Copy, Resource)]
pub struct TickInfo {
	pub acc: Duration,
	pub budget: Duration,
	pub step: Duration,
}

// zero or more ticks per frame
//	stop ticks if time budget spent
// (once) clean up events if at least one tick happened
// (once) interpolate remaining acc time

pub struct TickStage {
	inner: SystemStage,
}

impl TickStage {
	pub fn parallel() -> Self {
		TickStage { inner: SystemStage::parallel() }
	}

	pub fn single_threaded() -> Self {
		TickStage { inner: SystemStage::single_threaded() }
	}

	pub fn add_system<Params>(&mut self, system: impl IntoSystemDescriptor<Params>) -> &mut Self {
		self.inner.add_system(system);
		self
	}
}

impl Stage for TickStage {
	fn run(&mut self, world: &mut World) {
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

			self.inner.run(world);
		}

		write_acc(world, acc);
	}
}
