
use raytracer::{
	acc_grid,
	mesh::Mesh,
	scene::{Light},
	trace::*,
	transform::Transform,
};

use core::{prelude::*, primitives::*, SceneObject, Scene};

use cgmath::Vector3;
use std::cell::RefCell;

use std::{
	path::PathBuf,
	sync::{
		atomic::{AtomicUsize, Ordering, ATOMIC_USIZE_INIT},
		Arc,
		RwLock,
	},
	time::{Duration, Instant},
};

use std::fs::File;

static TRACE_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
static SHADOW_RAY_COUNT: AtomicUsize = ATOMIC_USIZE_INIT;
static SHADOW_TOTAL_TIME: AtomicUsize = ATOMIC_USIZE_INIT;

fn main() {
	let now = Instant::now();

	use simplelog::*;
	CombinedLogger::init(vec![WriteLogger::new(
		LevelFilter::Info,
		Config::default(),
		File::create("trace.log").unwrap(),
	)]);

	let mut scene = Scene::new();
	let mut sphere_mesh = Mesh::load_ply(PathBuf::from("assets/meshes/ico_sphere.ply"));

	let mut scene = Scene::load ("./core/tests/resources/basic_scene/scene.ron").expect("failed to load scene");

	// scene.objects.push(SceneObject::Sphere(Sphere {
	// 	origin: Vector3::new(-1.0, -0.5, 3.5),
	// 	radius: 0.5,
	// }, Material::Diffuse(Vector3::new(1.0, 0.00, 0.00), 0.02)));
	// scene.objects.push(Object::Sphere(Sphere { origin: Vector3::new(0.74,
	// -0.25, 3.5), radius: 0.75, material: Material::Metal(
	// Vector3::new(0.05, 0.25, 1.00), 0.01 )}));

	let mut cube_mesh = Mesh::load_ply(PathBuf::from("assets/meshes/dragon_vrip.ply"));
	cube_mesh.bake_transform(Vector3::new(0.0, -0.3, 2.9));
	// let mut cube_mesh = Arc::new(cube_mesh);
	let mut cube_grid = acc_grid::AccGrid::build_from_mesh(cube_mesh);
	// let hit = cube_grid.intersects(&Ray { origin: Vector3::new(0.0, 0.0,
	// 0.0), direction: Vector3::new(0.0, 0.0, 1.0) }); println!("{:?}", hit);
	// panic!();
	// cube_grid.intersects(&Ray { origin: Vector3::new(0.0, 0.0, 0.0),
	// direction: Vector3::new(0.0, 0.0, 1.0) });

	// let cube_grid = Arc::new(cube_grid);
	// let cube_model = Object::Grid(
	// 	cube_grid,
	// 	Material::Metal(Vector3::new(1.0, 1.0, 0.1), 0.15),
	// );

	// scene.objects.push(cube_model);

	// // Floor
	// scene.objects.push(SceneObject::Plane(Plane {
	// 	origin: Vector3::new(0.0, -1.0, 0.0),
	// 	normal: Vector3::new(0.0, 1.0, 0.0),
	// }, Material::Diffuse(Vector3::new(0.75, 0.75, 0.75), 0.5)));
	// // Ceiling
	// scene.objects.push(SceneObject::Plane(Plane {
	// 	origin: Vector3::new(0.0, 2.0, 0.0),
	// 	normal: Vector3::new(0.0, -1.0, 0.0),
	// }, Material::Emission(
	// 		Vector3::new(1.5, 1.5, 1.5),
	// 		Vector3::new(1.0, 1.0, 1.0),
	// 		0.27,
	// 		0.0,
	// 	)));
	// // Frontwall
	// scene.objects.push(SceneObject::Plane(Plane {
	// 	origin: Vector3::new(0.0, 0.0, -2.0),
	// 	normal: Vector3::new(0.0, 0.0, 1.0),
	// }, Material::Diffuse(Vector3::new(1.0, 1.0, 1.0), 0.4)));
	// // Backwall
	// scene.objects.push(SceneObject::Plane(Plane {
	// 	origin: Vector3::new(0.0, 0.0, 5.0),
	// 	normal: Vector3::new(0.0, 0.0, -1.0),
		
	// }, Material::Diffuse(Vector3::new(0.0, 0.0, 0.0), 0.9)));
	// // left wall
	// scene.objects.push(SceneObject::Plane(Plane {
	// 	origin: Vector3::new(-2.0, 0.0, 0.0),
	// 	normal: Vector3::new(1.0, 0.0, 0.0),
		
	// }, Material::Diffuse(Vector3::new(0.0, 0.0, 0.0), 0.3)));
	// // right wall
	// scene.objects.push(SceneObject::Plane(Plane {
	// 	origin: Vector3::new(2.0, 0.0, 0.0),
	// 	normal: Vector3::new(-1.0, 0.0, 0.0),
	// }, Material::Diffuse(Vector3::new(0.0, 0.0, 0.0), 0.3)));

	// scene.lights.push(Light { position: Vector3::new(0.0, 1.95, 2.5),
	// intensity: Vector3::new(0.8, 0.8, 1.0) }); scene.lights.push(Light {
	// position: Vector3::new(1.75, -0.75, 1.0), intensity: Vector3::new(0.8,
	// 1.0, 0.7) });
	let HEIGHT = 340;
	let WIDTH = HEIGHT / 9 * 16;

	let camera = CameraSettingsBuilder::default()
		.backbuffer_width(WIDTH)
		.backbuffer_height(HEIGHT)
		.fov_vert(55.0)
		.transform(Transform::identity())
		.focal_length(2.5)
		.aperture_radius(0.0)
		.build()
		.unwrap();

	let settings = SettingsBuilder::default()
		.camera_settings(camera)
		.sample_count(50)
		.tile_size((16, 16))
		.bounce_limit(5)
		.build()
		.unwrap();

	let task_handle = render_tiled(scene, settings);
	let render = task_handle.r#await();

	let final_image = render;

	fn element_wise_map<Fun: Fn(f64) -> f64>(vec: &Vector3<f64>, f: Fun) -> Vector3<f64> {
		Vector3::new(f(vec.x), f(vec.y), f(vec.z))
	}

	let mut export = vec![Vector3::new(0, 0, 0); WIDTH * HEIGHT];
	for (i, p) in final_image.iter().enumerate() {
		let exposure = 1.0;
		let gamma = 2.2;
		let tone_mapped =
			Vector3::new(1.0, 1.0, 1.0) - element_wise_map(&(p * -1.0 * exposure), |e| f64::exp(e));
		let tone_mapped = element_wise_map(&tone_mapped, |x| x.powf(1.0 / gamma));

		for i in 0..3 {
			let e = tone_mapped[i];
			if e > 1.0 || e < 0.0 {
				println!("Problem: {:?}", e);
			}
		}

		match (tone_mapped * 255.0).cast() {
			Some(v) => {
				export[i] = v;
			}
			None => println!("PROBLEM: {:?}", tone_mapped * 255.0),
		};
	}

	println!(
		"Finished render.\nTotal render time: {}s\nTotal amount of trace calls: {}\nTotal amount of shadow rays cast: {}\n",
		(Instant::now() - now).as_millis() as f32 / 1000.0,
		TRACE_COUNT.load(Ordering::Relaxed),
		SHADOW_RAY_COUNT.load(Ordering::Relaxed),
	);
	println!(
		"Total time spent on shadow rays: {}s",
		SHADOW_TOTAL_TIME.load(Ordering::Relaxed) as f32 / 1000.0 / 1000.0 / 1000.0
	);

	let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> =
		image::ImageBuffer::from_fn(WIDTH as u32, HEIGHT as u32, |x, y| {
			image::Rgb(export[(x + y * WIDTH as u32) as usize].into())
		});
	image
		.save("output.png")
		.expect("Failed to save buffer to disk");
}
