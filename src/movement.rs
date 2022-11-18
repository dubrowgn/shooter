use bevy::prelude::*;
use parry2d::na;
use std::ops::{Add, Sub};

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct Position {
	pub p: Vec2,
}

impl Position {
	pub const fn new(x: f32, y: f32) -> Self {
		Position { p: Vec2::new(x, y) }
	}

	pub const ZERO: Self = Self::new(0.0, 0.0);

	pub fn to_iso(&self) -> na::Isometry2<f32> {
		na::Isometry2::new(na::Vector2::new(self.p.x, self.p.y), 0.0)
	}
}

impl From<Vec2> for Position {
	fn from(p: Vec2) -> Self {
		Position{ p }
	}
}

impl Add for Position {
	type Output = Self;

	fn add(self, other: Self) -> Self {
		Self { p: self.p + other.p }
	}
}

impl<'a, 'b> Add<&'b Position> for &'a Position {
	type Output = Position;

	fn add(self, other: &'b Position) -> Position {
		Position { p: self.p + other.p }
	}
}

impl Sub for Position {
	type Output = Self;

	fn sub(self, other: Self) -> Self {
		Self { p: self.p - other.p }
	}
}

impl<'a, 'b> Sub<&'b Position> for &'a Position {
	type Output = Position;

	fn sub(self, other: &'b Position) -> Position {
		Position { p: self.p - other.p }
	}
}

#[derive(Component, Clone, Default, Reflect)]
#[reflect(Component)]
pub struct Velocity {
	pub v: Vec2,
}

impl Velocity {
	pub const fn new(x: f32, y: f32) -> Self {
		Velocity { v: Vec2::new(x, y) }
	}

	pub const ZERO: Self = Self::new(0.0, 0.0);

	pub fn to_vector2(&self) -> na::Vector2<f32> {
		na::Vector2::new(self.v.x, self.v.y)
	}
}

impl From<Vec2> for Velocity {
	fn from(v: Vec2) -> Self {
		Velocity{ v }
	}
}

pub fn sys_write_back(mut q: Query<(&Position, &mut Transform)>) {
	for (pos, mut t) in q.iter_mut() {
		t.translation.x = pos.p.x;
		t.translation.y = pos.p.y;
	}
}
