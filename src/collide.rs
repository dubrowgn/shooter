use bevy::ecs::{ component::Component, entity::Entity, query::WorldQuery, system::Query };
use crate::movement::Position;
use parry2d::{
	math::{Real, Isometry},
	partitioning::{Qbvh, IndexedData},
	shape::{SharedShape, Shape, TypedSimdCompositeShape},
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

pub struct QueryCompositeShape<'a, 'w, 's, Q: WorldQuery, F: WorldQuery> {
	pub query: &'a Query<'w, 's, Q, F>,
	pub bvh: &'a Qbvh<EntityHandle>,
}

impl<'a, 'w, 's, Q: WorldQuery, F: WorldQuery> TypedSimdCompositeShape for QueryCompositeShape<'a, 'w, 's, Q, F> {
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
