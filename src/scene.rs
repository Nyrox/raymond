use std::sync::Arc;

use cgmath::{prelude::*, *};

use super::{F, F_MAX, PI};

use super::{
	acc_grid,
	material::Material,
	mesh::Mesh,
	primitives::{Hit, Plane, Ray, SurfaceProperties, Triangle, AABB},
};

#[derive(Clone)]
pub enum Object {
	Plane(Plane),
	Model(Model),
	Grid(Arc<acc_grid::AccGrid>, Material),
}

impl Object {
	pub fn get_material(&self) -> &Material {
		match self {
			&Object::Plane(ref p) => &p.material,
			&Object::Model(ref m) => &m.material,
			&Object::Grid(ref g, ref m) => m,
		}
	}

	pub fn intersects(&self, ray: Ray) -> Option<Hit> {
		match self {
			&Object::Plane(ref p) => p.intersects(ray),
			&Object::Model(ref m) => m.intersects(ray),
			&Object::Grid(ref g, _) => g.intersects(ray),
		}
	}

	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		match self {
			&Object::Plane(ref p) => p.get_surface_properties(hit),
			&Object::Model(ref m) => m.get_surface_properties(hit),
			&Object::Grid(ref g, _) => g.get_surface_properties(hit),
		}
	}
}

#[derive(Clone)]
pub struct Model {
	pub mesh: Arc<Mesh>,
	pub material: Material,
}

impl Model {
	pub fn intersects(&self, ray: Ray) -> Option<Hit> {
		self.mesh.intersects(ray)
	}
	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		self.mesh.get_surface_properties(hit)
	}
}

#[derive(Clone)]
pub struct Scene {
	pub objects: Vec<Object>,
}

impl Scene {
	pub fn new() -> Scene {
		Scene { objects: Vec::new() }
	}
}
