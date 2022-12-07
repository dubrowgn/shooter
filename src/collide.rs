use bevy::{
	ecs::query::{WorldQuery, ReadOnlyWorldQuery},
	prelude::*,
};
use bevy_prototype_lyon::{
	shapes::{Circle, RectangleOrigin, Rectangle},
	prelude::{GeometryBuilder, DrawMode, StrokeMode},
	entity::ShapeBundle,
};
use crate::{debug::Debug, movement::Velocity};
use crate::layer::Layer;
use crate::movement::Position;
use parry2d::{
	math::{Real, Isometry},
	partitioning::{Qbvh, IndexedData},
	query::{
		DefaultQueryDispatcher,
		QueryDispatcher, details::TOICompositeShapeShapeBestFirstVisitor, TOIStatus,
	},
	shape::{SharedShape, Shape, TypedShape, TypedSimdCompositeShape},
	utils::DefaultStorage,
};

#[derive(Component)]
pub struct Collidable {
	pub shape: SharedShape,
}

impl Collidable {
	pub fn circle(r: f32) -> Self {
		SharedShape::ball(r).into()
	}

	pub fn aa_rect(w: f32, h: f32) -> Self {
		SharedShape::cuboid(w / 2.0, h / 2.0).into()
	}
}

impl From<SharedShape> for Collidable {
	fn from(shape: SharedShape) -> Collidable {
		Collidable {
			shape: shape.clone(),
		}
	}
}

#[derive(Clone, Copy, Debug)]
pub struct EntityHandle(pub Entity);

impl From<Entity> for EntityHandle {
	fn from(ent: Entity) -> Self {
		Self(ent)
	}
}

impl IndexedData for EntityHandle {
	fn default() -> Self {
		Self(Entity::from_bits(u64::MAX))
	}

	fn index(&self) -> usize {
		self.0.to_bits() as usize
	}
}

pub struct QueryCompositeShape<'a, 'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> {
	pub query: &'a Query<'w, 's, Q, F>,
	pub bvh: &'a Qbvh<EntityHandle>,
}

impl<'a, 'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery> TypedSimdCompositeShape for QueryCompositeShape<'a, 'w, 's, Q, F> {
	type PartShape = dyn Shape;
	type PartId = EntityHandle;
	type QbvhStorage = DefaultStorage;

	fn map_typed_part_at(
		&self,
		shape_id: Self::PartId,
		mut f: impl FnMut(Option<&Isometry<Real>>, &Self::PartShape),
	) {
		// FIXME -- figure out how to add type safety here?
		if let Ok(col) = self.query.get_component::<Collidable>(shape_id.0) {
			if let Ok(pos) = self.query.get_component::<Position>(shape_id.0) {
				f(Some(&pos.to_iso()), &*col.shape)
			}
		}
	}

	fn map_untyped_part_at(
		&self,
		shape_id: Self::PartId,
		f: impl FnMut(Option<&Isometry<Real>>, &Self::PartShape),
	) {
		self.map_typed_part_at(shape_id, f);
	}

	fn typed_qbvh(&self) -> &Qbvh<EntityHandle> {
		&self.bvh
	}
}

fn shape_to_bundle(shape: &dyn Shape, t: &Transform, visible: bool) -> ShapeBundle {
	let builder = GeometryBuilder::new();
	let mut bundle = match shape.as_typed_shape() {
		TypedShape::Ball(b) => builder.add(&Circle {
			center: Vec2::ZERO,
			radius:b.radius,
		}),
		TypedShape::Cuboid(c) => builder.add(&Rectangle {
			extents: 2.0 * Vec2::new(c.half_extents.x, c.half_extents.y),
			origin: RectangleOrigin::Center,
		}),
		_ => panic!("Unimplemented shape type {:?}", shape.shape_type()),
	}.build(
		DrawMode::Stroke(StrokeMode::color(Color::rgb(0.8, 0.8, 0.0))),
		Transform::from_xyz(0.0, 0.0, Layer::FG)
			.with_rotation(-t.rotation),
	);

	bundle.visibility.is_visible = visible;

	bundle
}

#[derive(Component, Default, Reflect)]
#[reflect(Component)]
pub struct CollidableDebug;

pub fn sys_collide_debug_add(
	mut cmds: Commands,
	debug: Res<Debug>,
	q_added: Query<(Entity, &Collidable, &Transform), Added<Collidable>>
) {
	for (ent, col, t) in &q_added {
		let id = cmds.spawn((
				CollidableDebug,
				Name::new("CollidableDebug"),
				shape_to_bundle(col.shape.as_ref(), t, debug.enabled),
			))
			.id();
		cmds.entity(ent)
			.add_child(id);
	}
}

pub fn sys_collide_debug_toggle(
	debug: Res<Debug>,
	mut q_collide: Query<&mut Visibility, With<CollidableDebug>>,
) {
	if !debug.is_changed() {
		return;
	}

	for mut vis in &mut q_collide {
		vis.is_visible = debug.enabled;
	}
}

#[derive(Debug)]
pub struct Contact {
	pub pos: Vec2,
	pub norm: Vec2,
	pub dist: f32,
}

impl Contact {
	pub fn mtv(&self) -> Vec2 {
		self.norm * (self.dist)
	}
}

/// Results are from the perspective of `col1`
pub fn contact(
	col1: &Collidable, pos1: &Position, col2: &Collidable, pos2: &Position
) -> Option<Contact> {
	let res = DefaultQueryDispatcher{}.contact(
		&(pos2 - pos1).to_iso(),
		col1.shape.as_ref(),
		col2.shape.as_ref(),
		f32::MAX,
	);

	let contact = match res {
		Ok(Some(c)) => c,
		Ok(None) => return None,
		Err(e) => {
			warn!("{}", e);
			return None;
		},
	};

	Some(Contact {
		pos: Vec2::new(contact.point1.x, contact.point1.y) + pos1.p,
		norm: Vec2::new(contact.normal2.x, contact.normal2.y),
		dist: -contact.dist,
	})
}

#[derive(Debug)]
pub struct Toi {
	pub norm: Vec2,
	pub toi_sec: f32,
}

#[derive(Debug)]
pub enum ToiResult {
	Contact(Contact),
	Miss,
	Toi(Toi),
}

pub fn toi<'w, 's, Q: WorldQuery, F: ReadOnlyWorldQuery>(
	query_geometry: &Query<'w, 's, Q, F>,
	query_bvh: &Qbvh<EntityHandle>,
	col: &Collidable,
	pos: &Position,
	vel: &Velocity,
	max_toi_sec: f32
) -> ToiResult {
	let dispatcher = DefaultQueryDispatcher{};
	let shapes = QueryCompositeShape {
		query: &query_geometry,
		bvh: &query_bvh,
	};
	let pos_iso = pos.to_iso();
	let vel_v2 = vel.to_vector2();

	let mut visitor = TOICompositeShapeShapeBestFirstVisitor::new(
		&dispatcher,
		&pos_iso,
		&vel_v2,
		&shapes,
		col.shape.as_ref(),
		max_toi_sec,
		true,
	);

	let (ent, toi) = unwrap!(query_bvh.traverse_best_first(&mut visitor).map(|h| h.1), {
		return ToiResult::Miss;
	});

	if toi.status == TOIStatus::Converged && toi.toi > 0.0 {
		return ToiResult::Toi(Toi {
			norm: Vec2::new(toi.normal1.x, toi.normal1.y),
			toi_sec: toi.toi,
		});
	}

	let geo_col = query_geometry.get_component::<Collidable>(ent.0).unwrap();
	let geo_pos = query_geometry.get_component::<Position>(ent.0).unwrap();

	return ToiResult::Contact(contact(col, &pos, geo_col, geo_pos).unwrap());
}
