use bevy::reflect::{FromReflect, Reflect};
use std::ops::Range;

#[derive(Clone, Debug, FromReflect, Reflect)]
pub struct Accumulator {
	acc_ns: u64,
	interval_ns: u64,
}

impl Accumulator {
	pub fn from_nanos(interval_ns: u64) -> Self {
		Self { acc_ns: 0, interval_ns }
	}

	pub fn from_millis(interval_ms: u32) -> Self {
		Self::from_nanos(1_000 * interval_ms as u64)
	}

	pub fn from_secs(interval_s: u32) -> Self {
		Self::from_nanos(1_000_000_000 * interval_s as u64)
	}

	pub fn ready_from_nanos(interval_ns: u64) -> Self {
		Self { acc_ns: interval_ns, interval_ns }
	}

	pub fn ready_from_millis(interval_ms: u32) -> Self {
		Self::ready_from_nanos(1_000_000 * interval_ms as u64)
	}

	pub fn ready_from_secs(interval_s: u32) -> Self {
		Self::ready_from_nanos(1_000_000_000 * interval_s as u64)
	}

	pub fn advance(&mut self, ns: u64) -> Range<u64> {
		self.acc_ns += ns;

		let n = self.acc_ns / self.interval_ns;
		self.acc_ns = self.acc_ns % self.interval_ns;

		0..n
	}
}
