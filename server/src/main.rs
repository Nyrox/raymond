extern crate raytracer;
extern crate ws;
#[macro_use]
extern crate serde;
extern crate serde_json;

use raytracer::{prelude::*, *};
use ws::*;

use std::path::PathBuf;
use std::sync::Arc;

struct Server {
	sender: Sender,
}

impl Handler for Server {
	fn on_open(&mut self, _: Handshake) -> Result<()> {
		println!("Connection established.");

			let mut scene = Scene::new();
	let mut sphere_mesh = Mesh::load_ply(PathBuf::from("assets/meshes/ico_sphere.ply"));

	// scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(-1.5, -0.5, 3.5), radius: 0.5, material: Material::Diffuse(
	//     Vector3::new(1.0, 0.00, 0.00), 0.04
	// )}));
	// scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(1.25, -0.25, 3.5), radius: 0.75, material: Material::Metal(
	//     Vector3::new(0.05, 0.25, 1.00), 0.02
	// )}));

	let mut cube_mesh = Mesh::load_ply(PathBuf::from("assets/meshes/dragon_vrip.ply"));
	cube_mesh.bake_transform(Vector3::new(0.0, -0.3, 2.9));
	// let mut cube_mesh = Arc::new(cube_mesh);
	let mut cube_grid = acc_grid::AccGrid::build_from_mesh(cube_mesh);
	// let hit = cube_grid.intersects(&Ray { origin: Vector3::new(0.0, 0.0, 0.0), direction: Vector3::new(0.0, 0.0, 1.0) });
	// println!("{:?}", hit);
	// panic!();
	// cube_grid.intersects(&Ray { origin: Vector3::new(0.0, 0.0, 0.0), direction: Vector3::new(0.0, 0.0, 1.0) });

	let cube_grid = Arc::new(cube_grid);
	let cube_model = Object::Grid(cube_grid, Material::Metal(Vector3::new(1.0, 1.0, 0.1), 0.15));

	scene.objects.push(cube_model);

	// // Floor
	scene.objects.push(Object::Plane(Plane {
		origin: Vector3::new(0.0, -1.0, 0.0),
		normal: Vector3::new(0.0, 1.0, 0.0),
		material: Material::Diffuse(Vector3::new(0.75, 0.75, 0.75), 0.5),
	}));
	// Ceiling
	scene.objects.push(Object::Plane(Plane {
		origin: Vector3::new(0.0, 2.0, 0.0),
		normal: Vector3::new(0.0, -1.0, 0.0),
		material: Material::Emission(Vector3::new(1.5, 1.5, 1.5), Vector3::new(1.0, 1.0, 1.0), 0.27, 0.0),
	}));
	// Frontwall
	scene.objects.push(Object::Plane(Plane {
		origin: Vector3::new(0.0, 0.0, -2.0),
		normal: Vector3::new(0.0, 0.0, 1.0),
		material: Material::Diffuse(Vector3::new(1.0, 1.0, 1.0), 0.4),
	}));
	// Backwall
	scene.objects.push(Object::Plane(Plane {
		origin: Vector3::new(0.0, 0.0, 5.0),
		normal: Vector3::new(0.0, 0.0, -1.0),
		material: Material::Diffuse(Vector3::new(0.0, 0.0, 0.0), 0.9),
	}));
	// left wall
	scene.objects.push(Object::Plane(Plane {
		origin: Vector3::new(-2.0, 0.0, 0.0),
		normal: Vector3::new(1.0, 0.0, 0.0),
		material: Material::Diffuse(Vector3::new(0.0, 0.0, 0.0), 0.9),
	}));
	// right wall
	scene.objects.push(Object::Plane(Plane {
		origin: Vector3::new(2.0, 0.0, 0.0),
		normal: Vector3::new(-1.0, 0.0, 0.0),
		material: Material::Diffuse(Vector3::new(0.0, 0.0, 0.0), 0.9),
	}));

		let WIDTH = 400;
		let HEIGHT = 340;

		let config = RaytracerConfig {
			width: WIDTH,
			height: HEIGHT,
			fov: 50.0,
			num_samples: 500,
			max_bounces: 5,

			..RaytracerConfig::default()
		};

		let sender = self.sender.clone();
		config.launch_tiled(scene.clone(), (64, 64), move |tile| {
			sender.send(serde_json::to_string(&tile).unwrap()).unwrap();
		});

		Ok(())
	}

	fn on_message(&mut self, msg: Message) -> Result<()> {
		self.sender.send(msg)
	}

	fn on_error(&mut self, err: Error) {
		println!("Error: {:?}", err);
	}

	fn on_close(&mut self, code: CloseCode, reason: &str) {
		println!("Connection closed: {:?}, {}", code, reason);
	}
}

fn main() {
	listen("127.0.0.1:3012", |sender| Server { sender }).unwrap();
}
