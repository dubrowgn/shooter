use bevy::ecs::{
	change_detection::Res,
	system::Resource,
};

#[derive(Debug, Resource)]
pub struct Debug {
	pub enabled: bool,
}

pub fn debug_enabled(debug: Res<Debug>) -> bool {
	debug.enabled
}
