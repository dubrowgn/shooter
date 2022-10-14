use bevy::prelude::*;

fn main() {
	App::new()
		.insert_resource(ClearColor(Color::rgb(1.0, 1.0, 1.0)))
		.insert_resource(WindowDescriptor {
			title: "test".to_string(),
			..default()
		})
		.add_plugins(DefaultPlugins)
		.run();
}
