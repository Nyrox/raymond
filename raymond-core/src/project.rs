use crate::{Material, Vector3, geometry::{AccGrid, Mesh, Plane, Sphere}, scene, scene::Scene};

use std::{
	fs::File,
	io::Write,
	path::{Path, PathBuf},
	sync::Arc,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Geometry {
	Plane(Plane),
	Sphere(Sphere),
	Mesh(PathBuf, Vector3),
}

#[derive(Serialize, Deserialize)]
pub struct Object {
	pub geometry: Geometry,
	pub material: Material,
}

#[derive(Serialize, Deserialize)]
pub struct Project {
	pub objects: Vec<Object>,
}

impl Project {
	pub fn new() -> Project {
		Project { objects: Vec::new() }
	}

	pub fn save(&self, p: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
		let mut file = File::create(p)?;
		let text = serde_json::to_string_pretty(&self)?;
		file.write_all(text.as_bytes())?;

		Ok(())
	}

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
					Geometry::Mesh(m, t) => {
						let mut mesh = Mesh::load_ply(m);
						mesh.bake_transform(t);
						scene::Geometry::Grid(Arc::new(AccGrid::build_from_mesh(mesh)))
					},
				},
				material: obj.material,
			});
		}

		scene
	}
}
