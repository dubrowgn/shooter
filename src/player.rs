use bevy::{
	ecs::component::Component,
	reflect::Reflect,
};
use crate::time::Accumulator;

#[derive(Component, Default, Reflect)]
pub struct Player {
	pub shot_acc: Option<Accumulator>,
}
