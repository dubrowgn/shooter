use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
	App::new()
		// types
		.register_type::<Player>()
		// resources
		.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
		.insert_resource(WindowDescriptor {
			title: "shooter".to_string(),
			..default()
		})
		.insert_resource(Textures::new())
		// plugins
		.add_plugins(DefaultPlugins)
		.add_plugin(WorldInspectorPlugin::new())
		// systems
		.add_startup_system_to_stage(StartupStage::PreStartup, load_assets)
		.add_startup_system(spawn_camera)
		.add_startup_system(spawn_player)
		// run
		.run();
}

type Textures = HashMap<String, Handle<TextureAtlas>>;

fn spawn_camera(mut cmds: Commands) {
	let cam = Camera2dBundle::default();

	cmds.spawn_bundle(cam);
}

#[derive(Component, Default, Reflect)]
struct Player;

fn spawn_player(mut cmds: Commands, textures: Res<Textures>) {
	cmds.spawn_bundle(SpriteSheetBundle {
		texture_atlas: textures.get("player_blue").unwrap().clone(),
		..default()
	})
	.insert(Player)
	.insert(Name::new("Player"));
}

fn load_assets(
	mut textures: ResMut<Textures>,
	assets: Res<AssetServer>,
	mut atlases: ResMut<Assets<TextureAtlas>>
) {
	let img: Handle<Image> = assets.load("image/player_blue.png");
	let handle = atlases.add(TextureAtlas::from_grid(
		img,
		Vec2::splat(256.0),
		3,
		1
	));

	textures.insert("player_blue".to_string(), handle.clone());
}
