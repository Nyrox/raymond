use std::sync::Arc;

use crate::math::prelude::*;
use crate::geometry::{AccGrid, Mesh, Plane, Sphere, Triangle, Ray, Hit, SurfaceProperties, Intersect};
use crate::Material;


#[derive(Clone)]
pub enum Geometry {
	Plane(Plane),
	Sphere(Sphere),
	Grid(Arc<AccGrid>),
}

impl Geometry {
	pub fn intersects(&self, ray: Ray) -> Option<Hit> {
		match self {
			&Geometry::Plane(ref p) => p.intersects(ray),
			&Geometry::Sphere(ref s) => s.intersects(ray),
			&Geometry::Grid(ref g) => g.intersects(ray),
		}
	}

	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		match self {
			&Geometry::Plane(ref p) => p.get_surface_properties(hit),
			&Geometry::Sphere(ref s) => s.get_surface_properties(hit),
			&Geometry::Grid(ref g) => g.get_surface_properties(hit),
		}
	}
}

#[derive(Clone)]
pub struct Object {
	pub geometry: Geometry,
	pub material: Material,
}

impl Object {
}

#[derive(Clone)]
pub struct Scene {
	pub objects: Vec<Object>,
}

impl Scene {
	pub fn new() -> Scene {
		Scene {
			objects: Vec::new(),
		}
	}

	pub fn intersect(&self, ray: Ray) -> Option<(&Object, Hit)> {
		let mut closest_distance = F_MAX;
		let mut closest_object = None;

		for (i, object) in self.objects.iter().enumerate() {
			match object.geometry.intersects(ray) {
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
