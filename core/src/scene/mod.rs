use std::{
	error::Error,
	fs::File,
	io::{self, Read, Write},
	path::Path,
};

use serde::{Deserialize, Serialize};

use crate::{primitives, prelude::*};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum SceneObject {
	Plane(primitives::Plane, Material),
	Sphere(primitives::Sphere, Material),
	// Model_SpatialGrid (Index),
}

impl SceneObject {
	pub fn intersects(&self, ray: Ray) -> Option<Hit> {
		match self {
			&SceneObject::Plane(ref p, _) => p.intersects(ray),
			&SceneObject::Sphere(ref s, _) => s.intersects(ray),
		}
	}

	pub fn get_surface_properties(&self, hit: Hit) -> SurfaceProperties {
		match self {
			&SceneObject::Plane(ref p, material) => SurfaceProperties { normal: p.get_normal_at (hit), material },
			&SceneObject::Sphere(ref s, material) => SurfaceProperties { normal: s.get_normal_at (hit), material },
		}
	}
}


#[derive(PartialEq, Clone, Debug, Serialize, Deserialize)]
pub struct Scene {
	pub objects: Vec<SceneObject>,
}

impl Scene {
	pub fn new () -> Scene {
		Scene {
			objects: Vec::new()
		}
	}

	pub fn load(path: &Path) -> Result<Scene, Box<Error>> {
		let f = File::open(path)?;
		Ok(ron::de::from_reader(f)?)
	}

	pub fn store(&self, path: &Path) -> Result<(), Box<Error>> {
		let mut f = File::create(path)?;
		let s = ron::ser::to_string_pretty(self, Default::default())?;
		f.write(s.as_bytes())?;

		Ok(())
	}

	pub fn intersect(&self, ray: Ray) -> Option<(&SceneObject, Hit)> {
		let mut closest_distance = ::std::f64::MAX;
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
