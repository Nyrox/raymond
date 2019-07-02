use raytracer::{
	acc_grid,
	material::Material,
	mesh::Mesh,
	primitives::Plane,
	scene::{Object, Scene},
	trace::{Message, *},
	transform::Transform,
};

use cgmath::Vector3;

use log::info;
use std::{path::PathBuf, sync::Arc};

use std::{
	fs::File,
	sync::{
		atomic::{AtomicBool, AtomicUsize, Ordering},
		mpsc,
	},
	time::Duration,
};

pub mod protocol;
use crate::protocol::*;

pub trait TaskHandle {
	fn finished(&self) -> bool;
	fn await_finish(&self);
	fn stop(&mut self);
	fn poll_message(&mut self) -> Option<Message>;
}

pub struct RenderTaskHandle {
	receiver: mpsc::Receiver<Message>,
	abort_flag: Arc<AtomicBool>,
	alive_thread_count: Arc<AtomicUsize>,
}

impl TaskHandle for RenderTaskHandle {
	fn finished(&self) -> bool {
		self.alive_thread_count.load(Ordering::Relaxed) <= 0
	}

	fn await_finish(&self) {
		while !self.finished() {
			std::thread::sleep(Duration::from_millis(200));
		}
	}

	fn stop(&mut self) {
		self.abort_flag.store(true, Ordering::SeqCst);
	}

	fn poll_message(&mut self) -> Option<Message> {
		self.receiver.try_recv().ok()
	}
}

use std::{
	io::prelude::*,
	net::{self, TcpListener, TcpStream},
};

fn main() {
	use simplelog::*;
	CombinedLogger::init(vec![WriteLogger::new(
		LevelFilter::Info,
		Config::default(),
		File::create("trace.log").unwrap(),
	)])
	.unwrap();

	let scene = {
		let mut scene = Scene::new();
		let mut sphere_mesh = Mesh::load_ply(PathBuf::from("assets/meshes/ico_sphere.ply"));

		// scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(-1.5,
		// -0.5, 3.5), radius: 0.5, material: Material::Diffuse(
		// Vector3::new(1.0, 0.00, 0.00), 0.04 )}));
		// scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(1.25,
		// -0.25, 3.5), radius: 0.75, material: Material::Metal(
		// Vector3::new(0.05, 0.25, 1.00), 0.02 )}));

		let mut cube_mesh = Mesh::load_ply(PathBuf::from("assets/meshes/dragon_vrip.ply"));
		cube_mesh.bake_transform(Vector3::new(0.0, -0.3, 2.9));
		// let mut cube_mesh = Arc::new(cube_mesh);
		let mut cube_grid = acc_grid::AccGrid::build_from_mesh(cube_mesh);
		// let hit = cube_grid.intersects(&Ray { origin: Vector3::new(0.0, 0.0,
		// 0.0), direction: Vector3::new(0.0, 0.0, 1.0) }); println!("{:?}",
		// hit); panic!();
		// cube_grid.intersects(&Ray { origin: Vector3::new(0.0, 0.0, 0.0),
		// direction: Vector3::new(0.0, 0.0, 1.0) });

		let cube_grid = Arc::new(cube_grid);
		let cube_model = Object::Grid(
			cube_grid,
			Material::Metal(Vector3::new(1.0, 1.0, 0.1), 0.15),
		);

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
			material: Material::Emission(
				Vector3::new(1.5, 1.5, 1.5),
				Vector3::new(1.0, 1.0, 1.0),
				0.27,
				0.0,
			),
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

		scene
	};

	let HEIGHT = 340;
	let WIDTH = HEIGHT / 9 * 16;

	let camera = CameraSettingsBuilder::default()
		.backbuffer_width(WIDTH)
		.backbuffer_height(HEIGHT)
		.fov_vert(55.0)
		.transform(Transform::identity())
		.focal_length(2.5)
		.aperture_radius(0.5)
		.build()
		.unwrap();

	let settings = SettingsBuilder::default()
		.camera_settings(camera)
		.sample_count(50)
		.tile_size((16, 16))
		.bounce_limit(5)
		.build()
		.unwrap();

	println!(
		"{}",
		serde_json::to_string(&protocol::Message::TileProgressed(raytracer::Tile {
			left: 15,
			top: 42,
			width: 12,
			height: 18,
			sample_count: 15,
			data: vec![]
		}))
		.unwrap()
	);

	let listener = TcpListener::bind("127.0.0.1:17025").unwrap();
	for stream in listener.incoming() {
		let mut stream = stream.unwrap();
		info!("New connection accepeted.");

		std::thread::spawn(move || {
			stream.write(b"Hello World").unwrap();

			loop {
				let mut buf = vec![0; 256];
				let result = stream.read(&mut buf).unwrap();

				println!("{:?}", buf);
				if result > 0 {
					info!("Received data: {:?}", buf);
				}
			}
		});
	}
}
