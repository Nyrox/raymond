use std::{
	default::Default, f64, marker, sync::{
		atomic::{AtomicUsize, Ordering}, mpsc::{self, Receiver, Sender, TryRecvError}, Arc, RwLock
	}, thread,
	time::Duration,
};

use crossbeam::{queue::MsQueue, thread::Scope};
use crossbeam_utils;
use rand;

use cgmath::*;
use num_cpus;

use super::{
	material::Material, primitives::{Hit, Plane, Ray}, scene::*, transform::Transform
};

use super::{F, F_MAX, PI};

#[derive(Builder, Clone, Debug)]
pub struct CameraSettings {
	pub backbuffer_width: usize,
	pub backbuffer_height: usize,
	pub fov_vert: f64,
	pub transform: Transform,
	pub focal_length: f64,
	pub aperture_radius: f64,
}

#[derive(Builder, Clone, Debug)]
pub struct Settings {
	#[builder(default = "num_cpus::get()")]
	pub worker_count: usize,

	pub camera_settings: CameraSettings,
	pub sample_count: usize,
	// The amount of samples per iteration
	// If set to 0, the renderer will only notify the Receiver when a tile is fully rendered
	#[builder(default = "0")]
	pub samples_per_iteration: usize,
	pub tile_size: (usize, usize),
	pub bounce_limit: usize,
}

struct TraceContext {
	pub scene: Scene,
	pub settings: Settings,
}

#[derive(Clone, Debug)]
pub enum Message {
	TileFinished(super::Tile),
	TileProgressed(super::Tile),
}

pub struct TaskHandle {
	pub receiver: mpsc::Receiver<Message>,
	pub settings: Settings,
	alive_thread_count: Arc<AtomicUsize>,
}

impl TaskHandle {
	pub fn await(&self) -> Vec<Vector3<f64>> {
		let mut out = vec![
			Vector3::new(0.0, 0.0, 0.0);
			self.settings.camera_settings.backbuffer_width
				* self.settings.camera_settings.backbuffer_height
		];

		'poll: loop {
			if self.alive_thread_count.load(Ordering::Relaxed) == 0 {
				'collect: loop {
					match self.receiver.try_recv() {
						Ok(Message::TileFinished(tile)) => {
							for y in 0..tile.height {
								for x in 0..tile.width {
									let s = tile.data[x + y * tile.width] / tile.sample_count as f64;

									out[x + tile.left
									        + (y + tile.top) * self.settings.camera_settings.backbuffer_width] = s;
								}
							}
						},
						_ => { break 'collect; }
					}
				}

				break 'poll;
			}
			thread::sleep(Duration::from_millis(500));
		}

		out
	}
}

pub fn render_tiled(scene: Scene, settings: Settings) -> TaskHandle {
	let queue = Arc::new(MsQueue::new());
	let (sender, receiver) = mpsc::channel();

	// Split the backbuffer into tiles and push them into the queue
	{
		let mut x = 0;
		let mut y = 0;

		'gen_tiles: loop {
			let tile_min = (x, y);
			let tile_max = (
				(x + settings.tile_size.0).min(settings.camera_settings.backbuffer_width ),
				(y + settings.tile_size.1).min(settings.camera_settings.backbuffer_height ),
			);
			let width = tile_max.0 - tile_min.0;
			let height = tile_max.1 - tile_min.1;

			queue.push(super::Tile {
				sample_count: 0,
				left: tile_min.0,
				top: tile_min.1,
				width,
				height,
				data: vec![Vector3::new(0.0, 0.0, 0.0); width * height],
			});

			y += settings.tile_size.1;
			if y >= settings.camera_settings.backbuffer_height {
				y = 0;
				x += settings.tile_size.0;
			}
			if x >= settings.camera_settings.backbuffer_width {
				break 'gen_tiles;
			}
		}
	}

	let thread_count = Arc::new(AtomicUsize::new(settings.worker_count));
	for _ in 0..settings.worker_count {
		let queue = queue.clone();
		let sender = sender.clone();

		// We clone both our scene and settings
		// This simplifies implementation
		let context = TraceContext {
			scene: scene.clone(),
			settings: settings.clone(),
		};

		let thread_count = thread_count.clone();
		thread::spawn(move || loop {
			let mut tile = match queue.try_pop() {
				Some(t) => t,
				None => {
					thread_count.fetch_sub(1, Ordering::Relaxed);
					return;
				}
			};

			for y in tile.top..(tile.top + tile.height) {
				for x in tile.left..(tile.left + tile.width) {
					let primary = generate_primary_ray(x, y, &context.settings.camera_settings);
					let sample = trace(primary, &context, 1);

					// Map the global pixel indices to the local tile buffer and store the sample
					tile.data[(x - tile.left) + (y - tile.top) * tile.width] += sample;
				}
			}

			tile.sample_count += 1;

			// Check if we are done
			if tile.sample_count == context.settings.sample_count {
				sender.send(Message::TileFinished(tile.clone())).unwrap();
			} else {
				queue.push(tile.clone());

				// Check if we want to send our tile down the pipe
				if context.settings.samples_per_iteration != 0
					&& tile.sample_count % context.settings.samples_per_iteration == 0
				{
					sender.send(Message::TileProgressed(tile.clone())).unwrap();
				}
			}
		});
	}

	return TaskHandle {
		receiver,
		settings,
		alive_thread_count: thread_count,
	};
}

fn trace(ray: Ray, context: &TraceContext, depth: usize) -> Vector3<F> {
	let settings = &context.settings;

	if depth > settings.bounce_limit {
		return Vector3::new(0.0, 0.0, 0.0);
	}

	let intersect = context.scene.intersect(ray);
	let (object, hit) = match intersect {
		Some((o, h)) => (o, h),
		None => return Vector3::new(0.0, 0.0, 0.0),
	};
	let surface_properties = object.get_surface_properties(hit);
	let normal = surface_properties.normal;
	let fragment_position = ray.origin + ray.direction * hit.distance;
	let (material_color, material_roughness, material_metalness) = match object.get_material() {
		Material::Diffuse(color, roughness) => (color, *roughness, 0.0),
		Material::Metal(color, roughness) => (color, *roughness, 1.0),
		Material::Emission(e, _, _, _) => {
			return *e;
		}
		_ => panic!(),
	};

	for light in context.scene.lights.iter() {}

	let view_dir = (settings.camera_settings.transform.position - fragment_position).normalize();
	let f0 = Vector3::new(0.04, 0.04, 0.04);
	let f0 = lerp_vec(f0, *material_color, material_metalness);
	// Decide whether to sample diffuse or specular
	let r = rand::random::<F>();
	let local_cartesian = create_coordinate_system_of_n(normal);
	let local_cartesian_transform = Matrix3::from_cols(local_cartesian.0, normal, local_cartesian.1);
	let prob_d = lerp(0.5, 0.0, material_metalness);
	if r < prob_d {
		let (sample, pdf) = uniform_sample_hemisphere();
		let sample_world = (local_cartesian_transform * sample).normalize();
		let radiance = trace(
			Ray {
				origin: fragment_position + normal * 0.00001,
				direction: sample_world,
			},
			context,
			depth + 1,
		);
		let cos_theta = normal.dot(sample_world).max(0.0);
		let halfway = (sample_world + view_dir).normalize();
		let fresnel = fresnel_schlick(halfway.dot(view_dir).max(0.0), f0);
		let specular_part = fresnel;
		let mut diffuse_part = Vector3::new(1.0, 1.0, 1.0) - specular_part;
		diffuse_part *= 1.0 - material_metalness;
		let output = (diffuse_part.mul_element_wise(*material_color)).mul_element_wise(radiance) * cos_theta;
		return output / (prob_d * pdf);
	} else {
		// Sample specular
		let reflect = (-view_dir - 2.0 * (-view_dir.dot(normal) * normal)).normalize();
		fn importance_sample_ggx(reflect: Vector3<F>, roughness: F) -> Vector3<F> {
			let r1: F = rand::random();
			let r2: F = rand::random();
			let a = roughness * roughness;
			let phi = 2.0 * PI * r1;
			let theta = a * (r2 / (1.0 - r2)).sqrt();
			let h = Vector3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
			let (tangent, bitangent) = create_coordinate_system_of_n(reflect);
			let matrix = Matrix3::from_cols(tangent, reflect, bitangent);
			return (matrix * h).normalize();
		}
		let sample_world = importance_sample_ggx(reflect, material_roughness);
		let radiance = trace(
			Ray {
				origin: fragment_position + normal * 0.0001,
				direction: sample_world,
			},
			context,
			depth + 1,
		);
		let cos_theta = normal.dot(sample_world);
		let light_dir = sample_world.normalize();
		let halfway = (light_dir + view_dir).normalize();
		let F = fresnel_schlick(halfway.dot(view_dir), f0);
		let D = ggx_distribution(normal, halfway, material_roughness);
		let G = geometry_smith(normal, view_dir, sample_world, material_roughness);
		let nominator = D * G * F;
		let denominator = 4.0 * normal.dot(view_dir) * cos_theta + 0.001;
		let specular = nominator / denominator;
		let output = (specular).mul_element_wise(radiance) * cos_theta;
		// pdf
		let pdf = { (D * normal.dot(halfway)) / (4.0 * halfway.dot(view_dir)) + 0.0001 };
		return output / (1.0 - prob_d) / pdf;
	}
}

fn generate_primary_ray(x: usize, y: usize, camera: &CameraSettings) -> Ray {
	let width = camera.backbuffer_width as F;
	let height = camera.backbuffer_height as F;
	let aspect = width / height;
	let x = x as F + (rand::random::<F>() - 0.5);
	let y = y as F + (rand::random::<F>() - 0.5);

	let px = (2.0 * ((x + 0.5) / width) - 1.0) * F::tan(camera.fov_vert / 2.0 * PI / 180.0) * aspect;
	let py = (1.0 - 2.0 * ((y + 0.5) / height)) * F::tan(camera.fov_vert / 2.0 * PI / 180.0);

	Ray::new(camera.transform.position, Vector3::new(px, py, 1.0).normalize())
}

fn generate_primary_ray_with_dof(x: usize, y: usize, camera: &CameraSettings) -> Ray {
	let primary = generate_primary_ray(x, y, camera);

	// chose random point on the aperture, through rejection sampling
	let start = loop {
		let r1 = rand::random::<F>() * 2.0 - 1.0;
		let r2 = rand::random::<F>() * 2.0 - 1.0;
		let ax = camera.transform.position.x + r1 * camera.aperture_radius;
		let ay = camera.transform.position.y + r2 * camera.aperture_radius;
		let _start = Vector3::new(ax, ay, camera.transform.position.z);
		if _start.distance(camera.transform.position) < camera.aperture_radius {
			break _start;
		}
	};

	let focal_plane = Plane {
		origin: camera.transform.position + Vector3::new(0.0, 0.0, 1.0) * camera.focal_length,
		normal: Vector3::new(0.0, 0.0, -1.0),
		material: Material::Diffuse(Vector3::new(0.75, 0.75, 0.75), 0.5),
	};
	let end = camera.transform.position + focal_plane.intersects(primary).unwrap().distance * primary.direction;

	return Ray {
		origin: start,
		direction: (end - start).normalize(),
	};
}

fn ggx_distribution(n: Vector3<F>, h: Vector3<F>, roughness: F) -> F {
	let a2 = roughness * roughness;
	let NdotH = n.dot(h);

	let nominator = a2;
	let denominator = NdotH.powf(2.0) * (a2 - 1.0) + 1.0;
	let denominator = (PI * denominator * denominator).max(1e-7);
	return nominator / denominator;
}

fn geometry_schlick_ggx(n: Vector3<F>, v: Vector3<F>, r: F) -> F {
	let numerator = n.dot(v).max(0.0);
	let k = (r * r) / 8.0;
	let denominator = numerator * (1.0 - k) + k;

	return numerator / denominator;
}

fn geometry_smith(n: Vector3<F>, v: Vector3<F>, l: Vector3<F>, r: F) -> F {
	return geometry_schlick_ggx(n, v, r) * geometry_schlick_ggx(n, l, r);
}

fn fresnel_schlick(cos_theta: F, F0: Vector3<F>) -> Vector3<F> {
	return F0 + (Vector3::new(1.0, 1.0, 1.0) - F0) * (1.0 - cos_theta).powf(5.0);
}

fn lerp_vec(min: Vector3<F>, max: Vector3<F>, a: F) -> Vector3<F> {
	Vector3::new(lerp(min.x, max.x, a), lerp(min.y, max.y, a), lerp(min.z, max.z, a))
}

fn lerp(min: F, max: F, a: F) -> F {
	min + a * (max - min)
}

fn uniform_sample_hemisphere() -> (Vector3<F>, F) {
	let r1 = rand::random::<F>();
	let r2 = rand::random::<F>();

	let theta = (r1.sqrt()).acos();
	let phi = 2.0 * PI * r2;

	let pdf = r1.sqrt();
	let cartesian = Vector3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
	return (cartesian, pdf);
}

fn create_coordinate_system_of_n(n: Vector3<F>) -> (Vector3<F>, Vector3<F>) {
	let sign = if n.z > 0.0 { 1.0 } else { -1.0 };
	let a = -1.0 / (sign + n.z);
	let b = n.x * n.y * a;
	return (
		Vector3::new(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x),
		Vector3::new(b, sign + n.y * n.y * a, -n.y),
	);
}
