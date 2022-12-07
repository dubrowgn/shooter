use bevy::ecs::system::Resource;

#[derive(Debug, Resource)]
pub struct Debug {
	pub enabled: bool,
}
