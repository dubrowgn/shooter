use bevy::ecs::{
	change_detection::Res,
	system::Resource,
};

#[derive(Debug, Default, Resource)]
pub struct Debug {
	pub enabled: bool,
}

pub fn debug_enabled(debug: Res<Debug>) -> bool {
	debug.enabled
}
