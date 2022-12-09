pub struct Metric {
	count: u32,
	max: f32,
	min: f32,
	total: f32,
}

impl Default for Metric {
	fn default() -> Metric {
		Metric {
			count: 0,
			max: f32::MIN,
			min: f32::MAX,
			total: 0.0,
		}
	}
}

impl Metric {
	pub fn avg(&self) -> f32 { self.total / self.count as f32 }
	pub fn count(&self) -> u32 { self.count }
	pub fn max(&self) -> f32 { self.max }
	pub fn min(&self) -> f32 { self.min }
	pub fn total(&self) -> f32 { self.total }

	pub fn reset(&mut self) {
		*self = Metric::default();
	}

	pub fn sample(&mut self, val: f32) {
		self.min = f32::min(self.min, val);
		self.max = f32::max(self.max, val);
		self.total += val;
		self.count += 1;
	}
}
