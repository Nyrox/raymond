use crate::scene::{Scene};
use crate::geometry::{Plane, Sphere, Mesh};
use crate::scene;
use crate::Material;
use crate::geometry::AccGrid;

use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::fs::File;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub enum Geometry {
	Plane(Plane),
	Sphere(Sphere),
	Mesh(PathBuf),
}


#[derive(Serialize, Deserialize)]
pub struct Object {
	geometry: Geometry,
	material: Material,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
	pub objects: Vec<Object>,
}

impl Project {
	pub fn load(p: impl AsRef<Path>) -> Result<Project, Box<dyn std::error::Error>> {
		let file = File::open(p)?;
		Ok(serde_json::from_reader(file)?)
	}

	pub fn build_scene(self) -> Scene {
		let mut scene = Scene::new();

		for obj in self.objects {
			scene.objects.push(scene::Object {
				geometry: match obj.geometry {
					Geometry::Plane(p) => scene::Geometry::Plane(p),
					Geometry::Sphere(s) => scene::Geometry::Sphere(s),
					Geometry::Mesh(m) => {
						scene::Geometry::Grid(
							Arc::new(AccGrid::build_from_mesh(Mesh::load_ply(m)))
						)
					}
				},
				material: obj.material
			});
		}

		scene
	}
}
