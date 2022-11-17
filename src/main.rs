mod args;
mod collide;
mod debug;
mod layer;
#[macro_use]
mod macros;
mod movement;

use args::parse_args;
use bevy::{
	input::Input,
	prelude::*,
	utils::{Duration, HashMap},
};
use bevy_inspector_egui::{WorldInspectorPlugin, WorldInspectorParams};
use bevy_prototype_lyon::plugin::ShapePlugin;
use collide::{
	Collidable,
	contact,
	EntityHandle,
	QueryCompositeShape,
	sys_collide_debug_add,
	sys_collide_debug_toggle,
};
use debug::Debug;
use iyes_loopless::prelude::*;
use layer::Layer;
use movement::{Position, sys_write_back, Velocity};
use parry2d::{
	na,
	partitioning::Qbvh,
	query::{
		DefaultQueryDispatcher,
		details::TOICompositeShapeShapeBestFirstVisitor,
		TOI,
		TOIStatus,
	},
};

const HALF_TURN: f32 = std::f32::consts::PI;
const QUARTER_TURN: f32 = HALF_TURN / 2.0;

fn main() {
	let config = unwrap!(parse_args(), {
		return;
	});

	println!("{:?}", config);

	App::new()
		// types
		.register_type::<Player>()
		.register_type::<Position>()
		.register_type::<Shot>()
		.register_type::<Velocity>()
		// resources
		.insert_resource(ClearColor(Color::rgb(0.2, 0.2, 0.2)))
		.insert_resource(Debug { enabled: false })
		.insert_resource({
			let mut params = WorldInspectorParams::default();
			params.enabled = false;
			params
		})
		.insert_resource(WindowDescriptor {
			title: "shooter".into(),
			..default()
		})
		.insert_resource(Textures::new())
		.insert_resource(Walls::new())
		// plugins
		.add_plugins(DefaultPlugins)
		.add_plugin(ShapePlugin)
		.add_plugin(WorldInspectorPlugin::new())
		// startup systems
		.add_startup_system(load_assets)
		.add_startup_system(spawn_camera)
		.add_startup_system(spawn_player.after(load_assets))
		.add_startup_system(spawn_walls.after(load_assets))
		.add_startup_system(sys_spawn_shots.after(load_assets))
		.add_startup_system_to_stage(StartupStage::PostStartup, sys_index_walls)
		// game systems
		.add_fixed_timestep(Duration::from_nanos(1_000_000_000/60), "game")
		.add_fixed_timestep_system(
			"game", 0, sys_move_shots,
		)
		.add_fixed_timestep_system(
			"game", 0, sys_move_player.after(sys_move_shots),
		)
		.add_fixed_timestep_system(
			"game", 0, sys_write_back.after(sys_move_player),
		)
		// systems
		.add_system(handle_input)
		.add_system(sys_collide_debug_add)
		.add_system(sys_collide_debug_toggle)
		.add_system(sys_inspector_toggle)
		.add_system(sys_spawn_shot
			.after(handle_input)
			.before(update_camera)
		)
		.add_system(update_camera)
		// run
		.run();
}

pub fn sys_inspector_toggle(
	debug: Res<Debug>,
	mut inspector: ResMut<WorldInspectorParams>,
) {
	if !debug.is_changed() {
		inspector.enabled = debug.enabled;
	}
}

type Textures = HashMap<String, Handle<TextureAtlas>>;
type Walls = Qbvh<EntityHandle>;

fn spawn_camera(mut cmds: Commands) {
	let scale = 1.5;
	let far  = 1000.0;

	let cam = Camera2dBundle {
		transform: Transform::from_xyz(0.0, 0.0, far)
			.with_scale(Vec3::new(scale, scale, 1.0)),
		..default()
	};

	cmds.spawn_bundle(cam);
}

fn update_camera(
	mut q_camera: Query<&mut Transform, With<Camera>>,
	q_player: Query<&Transform, (With<Player>, Without<Camera>)>
) {
	let p = q_player.single().translation;
	let c = &mut q_camera.single_mut().translation;
	c.x = p.x;
	c.y = p.y;
}

fn sys_index_walls(
	mut walls: ResMut<Walls>,
	q_walls: Query<(Entity, &Collidable, &Position), With<Wall>>
) {
	let shapes = q_walls.iter()
		.map(|(ent, ref col, ref pos)| (EntityHandle::from(ent), col.shape.compute_aabb(&pos.to_iso())));
	walls.clear_and_rebuild(shapes, 0.0);
}

fn refect(v: Vec2, norm: Vec2) -> Vec2 {
	v - 2.0 * v.dot(norm) * norm
}

fn slide(v: Vec2, norm: Vec2) -> Vec2 {
	v - v.dot(norm) * norm
}

fn sys_move_shots(
	mut cmds: Commands,
	timesteps: Res<FixedTimesteps>,
	walls: Res<Walls>,
	mut q_shots: Query<(Entity, &Collidable, &mut Position, &mut Velocity, &mut Shot)>,
	q_walls: Query<(Entity, &Collidable, &Position), (With<Wall>, Without<Shot>)>,
) {
	let step_secs = timesteps.get_current().unwrap().step.as_secs_f32();

	let dispatcher = DefaultQueryDispatcher{};
	for (ent, col, mut pos, mut vel, mut shot) in &mut q_shots {
		let mut dt = step_secs;
		while dt > 0.0 {
			let shapes = QueryCompositeShape {
				query: &q_walls,
				bvh: &walls,
			};
			let iso = pos.to_iso();
			let v2 = vel.to_vector2();
			let mut visitor = TOICompositeShapeShapeBestFirstVisitor::new(
				&dispatcher,
				&iso,
				&v2,
				&shapes,
				col.shape.as_ref(),
				dt,
				false,
			);
			if let Some(res) = walls.traverse_best_first(&mut visitor).map(|h| h.1) {
				let toi: TOI = res.1;
				if shot.bounces == 0 {
					cmds.entity(ent)
						.despawn_recursive();
					break;
				} else {
					shot.bounces -= 1;
					dt -= toi.toi;

					pos.p += vel.v * toi.toi;
					vel.v = refect(vel.v, Vec2::new(toi.normal1.x, toi.normal1.y));
				}
			} else {
				pos.p += vel.v * dt;
				break;
			}
		}
	}
}

fn sys_move_player(
	timesteps: Res<FixedTimesteps>,
	walls: Res<Walls>,
	mut q_player: Query<(&Collidable, &mut Position, &Velocity), With<Player>>,
	q_walls: Query<(Entity, &Collidable, &Position), (With<Wall>, Without<Player>)>,
) {
	let step_secs = timesteps.get_current().unwrap().step.as_secs_f32();

	let dispatcher = DefaultQueryDispatcher{};
	for (col, mut pos, vel) in &mut q_player {
		if vel.v == Vec2::ZERO {
			continue;
		}

		let mut dt = step_secs;
		let mut v = vel.v;
		let mut limit = 8;
		while dt > 0.0 && limit > 0 {
			limit -= 1;

			let shapes = QueryCompositeShape {
				query: &q_walls,
				bvh: &walls,
			};
			let iso = pos.to_iso();
			let v2 = na::Vector2::new(v.x, v.y);
			let mut visitor = TOICompositeShapeShapeBestFirstVisitor::new(
				&dispatcher,
				&iso,
				&v2,
				&shapes,
				col.shape.as_ref(),
				dt,
				false,
			);
			if let Some(res) = walls.traverse_best_first(&mut visitor).map(|h| h.1) {
				let toi: TOI = res.1;
				if toi.status == TOIStatus::Penetrating {
					let ent = res.0;
					let (_, wall_col, wall_pos) = q_walls.get(ent.0).unwrap();

					let contact = contact(col, &pos, wall_col, wall_pos).unwrap();
					let margin:f32 = 8192.0 * f32::EPSILON;
					pos.p += contact.norm * (contact.dist + margin);
					v = slide(v, contact.norm);
				} else {
					dt -= toi.toi;

					pos.p += v * toi.toi;
					v = slide(v, Vec2::new(toi.normal1.x, toi.normal1.y));
				}
			} else {
				pos.p += v * dt;
				break;
			}
		}
	}
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Player {
	shot_timer: Timer,
}

impl Player {
	fn new() -> Self {
		let mut p = Player {
			shot_timer: Timer::new(Duration::from_millis(100), true),
		};

		p.shot_timer.pause();

		return p;
	}
}

fn spawn_player(mut cmds: Commands, textures: Res<Textures>) {
	cmds.spawn()
		.insert(Player::new())
		.insert(Name::new("Player"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("player_purple").unwrap().clone(),
			transform: Transform::from_xyz(0.0, 0.0, Layer::PLAYER),
			..default()
		})
		.insert(Collidable::circle(96.0))
		.insert(Position::ZERO)
		.insert(Velocity::ZERO);
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
struct Shot {
	bounces: u8,
}

fn spawn_shot(
	cmds: &mut Commands,
	textures: &Res<Textures>,
	pos: Vec2,
	dir: Vec2,
) {
	let speed: f32 = 2700.0;

	cmds.spawn()
		.insert(Shot{ bounces: 3 })
		.insert(Name::new("Shot"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("shot_purple").unwrap().clone(),
			transform: Transform::from_xyz(pos.x, pos.y, Layer::SHOT),
			..default()
		})
		.insert(Collidable::circle(26.0))
		.insert(Position::from(pos))
		.insert(Velocity::from(dir * speed));
}

fn sys_spawn_shot(
	mut cmds: Commands,
	textures: Res<Textures>,
	time: Res<Time>,
	mut q_player: Query<(&Transform, &mut Player)>
) {
	let (player_t, mut player) = q_player.single_mut();
	let dir = player_t.right().truncate();
	let pos = player_t.translation.truncate() + dir * (96.0 + 26.0);

	player.shot_timer.tick(time.delta());
	let shots = player.shot_timer.times_finished_this_tick();
	for _ in 0..shots {
		spawn_shot(&mut cmds, &textures, pos, dir);
	}
}

fn sys_spawn_shots(
	mut cmds: Commands,
	textures: Res<Textures>,
) {
	let half_range = 0;
	for i in -half_range..half_range {
		let f = 0.001 * i as f32;
		let pos = Vec2::new(f, f);
		let dir = Vec2::from_angle(f);
		spawn_shot(&mut cmds, &textures, pos, dir);
	}
}

#[derive(Component, Default, Reflect)]
struct Wall;

fn spawn_walls(mut cmds: Commands, textures: Res<Textures>) {
	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Left"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_out_left").unwrap().clone(),
			transform: Transform::from_xyz(-1184.0, 0.0, Layer::BG),
			..default()
		})
		.insert(Collidable::aa_rect(192.0, 3840.0))
		.insert(Position::new(-1184.0, 0.0));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Right"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_out_right").unwrap().clone(),
			transform: Transform::from_xyz(1184.0, 0.0, Layer::BG),
			..default()
		})
		.insert(Collidable::aa_rect(192.0, 3840.0))
		.insert(Position::new(1184.0, 0.0));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Top"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_out_top").unwrap().clone(),
			transform: Transform
				::from_rotation(Quat::from_rotation_z(QUARTER_TURN))
				.with_translation(Vec3::new(0.0, 1824.0, Layer::BG)),
			..default()
		})
		.insert(Collidable::aa_rect(2560.0, 192.0))
		.insert(Position::new(0.0, 1824.0));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Bottom"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_out_bottom").unwrap().clone(),
			transform: Transform
				::from_rotation(Quat::from_rotation_z(QUARTER_TURN))
				.with_translation(Vec3::new(0.0, -1824.0, Layer::BG)),
			..default()
		})
		.insert(Collidable::aa_rect(2560.0, 192.0))
		.insert(Position::new(0.0, -1824.0));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Horizontal"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_in_horizontal").unwrap().clone(),
			transform: Transform
				::from_rotation(Quat::from_rotation_z(QUARTER_TURN))
				.with_translation(Vec3::new(-196.0, -1149.5, Layer::BG)),
			..default()
		})
		.insert(Collidable::aa_rect(1066.0, 299.0))
		.insert(Position::new(-196.0, -1149.5));

	cmds.spawn()
		.insert(Wall)
		.insert(Name::new("Wall - Verticle"))
		.insert_bundle(SpriteSheetBundle {
			texture_atlas: textures.get("wall_in_verticle").unwrap().clone(),
			transform: Transform::from_xyz(702.0, 288.5, Layer::BG),
			..default()
		})
		.insert(Collidable::aa_rect(296.0, 2465.0))
		.insert(Position::new(702.0, 288.5));
}

fn load_assets(
	mut textures: ResMut<Textures>,
	assets: Res<AssetServer>,
	mut atlases: ResMut<Assets<TextureAtlas>>
) {
	{
		let img: Handle<Image> = assets.load("image/players.png");
		let size = 256.0;
		let slice = |row: u16| TextureAtlas::from_grid_with_padding(
			img.clone(),
			Vec2::splat(size),
			3,
			1,
			Vec2::ZERO,
			Vec2::new(0.0, row as f32 * size),
		);
		textures.insert("player_blue".into(), atlases.add(slice(0)));
		textures.insert("player_red".into(), atlases.add(slice(1)));
		textures.insert("player_green".into(), atlases.add(slice(2)));
		textures.insert("player_purple".into(), atlases.add(slice(3)));
	}

	{
		let img: Handle<Image> = assets.load("image/shots.png");
		let size = 96.0;
		let slice = |row: u16| TextureAtlas::from_grid_with_padding(
			img.clone(),
			Vec2::splat(size),
			3,
			1,
			Vec2::ZERO,
			Vec2::new(0.0, row as f32 * size),
		);
		textures.insert("shot_blue".into(), atlases.add(slice(0)));
		textures.insert("shot_red".into(), atlases.add(slice(1)));
		textures.insert("shot_green".into(), atlases.add(slice(2)));
		textures.insert("shot_purple".into(), atlases.add(slice(3)));
	}

	{
		let img: Handle<Image> = assets.load("image/walls.png");
		let rect = |l, t, w, h| TextureAtlas::from_grid_with_padding(
			img.clone(),
			Vec2::new(w, h),
			1,
			1,
			Vec2::ZERO,
			Vec2::new(l, t),
		);
		textures.insert("wall_out_left".into(), atlases.add(rect(0.0, 0.0, 192.0, 3840.0)));
		textures.insert("wall_out_right".into(), atlases.add(rect(194.0, 0.0, 192.0, 3840.0)));
		textures.insert("wall_out_top".into(), atlases.add(rect(686.0, 0.0, 192.0, 2176.0)));
		textures.insert("wall_out_bottom".into(), atlases.add(rect(880.0, 0.0, 192.0, 2176.0)));
		textures.insert("wall_in_verticle".into(), atlases.add(rect(386.0, 0.0, 296.0, 2465.0)));
		textures.insert("wall_in_horizontal".into(), atlases.add(rect(386.0, 2465.0, 299.0, 1066.0)));
	}
}

fn handle_input(
	keys: Res<Input<KeyCode>>,
	btns: Res<Input<MouseButton>>,
	mut debug: ResMut<Debug>,
	mut windows: ResMut<Windows>,
	q_camera: Query<(&Camera, &GlobalTransform), With<Camera>>,
	mut q_player: Query<(&mut Velocity, &mut Transform, &mut Player)>,
) {
	if keys.just_pressed(KeyCode::F11) {
		use bevy::window::WindowMode::{BorderlessFullscreen, Windowed};

		let win: &mut Window = windows.primary_mut();
		win.set_mode(if win.mode() == Windowed { BorderlessFullscreen } else { Windowed });
	}

	if keys.just_released(KeyCode::F12) {
		debug.enabled = !debug.enabled;
	}

	let (cam, cam_t) = q_camera.single();
	let (mut player_v, mut player_t, mut player) = q_player.single_mut();

	// shooting?
	if btns.just_pressed(MouseButton::Left) {
		let dur = player.shot_timer.duration();
		player.shot_timer.reset();
		player.shot_timer.unpause();
		player.shot_timer.set_elapsed(dur);
	} else if btns.just_released(MouseButton::Left) {
		player.shot_timer.pause();
	}

	// velocity

	let mut d = Vec2::ZERO;
	if keys.pressed(KeyCode::W) {
		d.y += 1.0;
	}
	if keys.pressed(KeyCode::S) {
		d.y -= 1.0;
	}
	if keys.pressed(KeyCode::A) {
		d.x -= 1.0;
	}
	if keys.pressed(KeyCode::D) {
		d.x += 1.0;
	}
	player_v.v = 900.0f32 * d.normalize_or_zero();

	// rotation

	let win = windows
		.get_primary()
		.unwrap();
	let win_pos = win.cursor_position();
	if win_pos == None {
		return;
	}

	let win_size = Vec2::new(win.width() as f32, win.height() as f32);

	// convert screen position [0..resolution] to ndc [-1..1] (gpu coordinates)
	let ndc = win_pos.unwrap() / win_size * 2.0 - Vec2::ONE;

	// matrix for undoing the projection and camera transform
	let ndc_to_world = cam_t.compute_matrix() * cam.projection_matrix().inverse();

	// use it to convert ndc to world-space coordinates
	let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));

	let delta_t = world_pos - player_t.translation;
	let rads = Vec2::X.angle_between(delta_t.truncate());

	player_t.rotation = Quat::from_rotation_z(rads);
}
