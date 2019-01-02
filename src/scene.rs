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
pub struct Light {
	pub mesh: Arc<Mesh>,
	pub intensity: Vector3<f64>,
}

#[derive(Clone)]
pub struct Scene {
	pub objects: Vec<Object>,
	pub lights: Vec<Light>,
}

impl Scene {
	pub fn new() -> Scene {
		Scene {
			objects: Vec::new(),
			lights: Vec::new(),
		}
	}

	pub fn intersect(&self, ray: Ray) -> Option<(&Object, Hit)> {
		let mut closest_distance = F_MAX;
		let mut closest_object = None;

		for (i, object) in self.objects.iter().enumerate() {
			match object.intersects(ray) {
				Some(hit) => {
					if hit.distance < closest_distance {
						closest_distance = hit.distance;
						closest_object = Some((i, hit));
					}
				}
				None => (),
			}
		}

		match closest_object {
			Some((o, h)) => return Some((&self.objects[o], h)),
			_ => return None,
		}
	}
}
