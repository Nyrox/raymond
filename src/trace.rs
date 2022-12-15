use std::{
	f32,
	sync::{
		atomic::{AtomicUsize, Ordering},
		mpsc::{self},
		Arc,
	},
	thread,
	time::Duration,
};

use crossbeam::queue::ArrayQueue;
use rand;

use num_cpus;

use super::transform::Transform;

use raymond_core::{geometry::*, prelude::*, scene::Scene, tile::Tile};


#[derive(Builder, Clone, Debug)]
pub struct CameraSettings {
	pub backbuffer_width: usize,
	pub backbuffer_height: usize,
	pub fov_vert: f32,
	pub transform: Transform,
	pub focal_length: f32,
	pub aperture_radius: f32,
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
	Finished,
	TileFinished(Tile),
	TileProgressed(Tile),
}

pub type TileCallback = Box<dyn Fn(Tile) -> ()>;


pub struct TaskHandle {
	pub receiver: mpsc::Receiver<Message>,
	pub settings: Settings,
	callback: Option<TileCallback>,
	alive_thread_count: Arc<AtomicUsize>,
}



impl TaskHandle {
	pub fn set_callback(&mut self, callback: Option<TileCallback>) {
		self.callback = callback;
	}

	pub fn r#await(&self) -> Vec<Vector3> {
		let mut out = vec![
			Vector3::new(0.0, 0.0, 0.0);
			self.settings.camera_settings.backbuffer_width * self.settings.camera_settings.backbuffer_height
		];

		'poll: loop {
			if self.alive_thread_count.load(Ordering::Relaxed) == 0 {
				'collect: loop {
					match self.receiver.try_recv() {
						Ok(Message::TileFinished(tile)) => {
							for y in 0..tile.height {
								for x in 0..tile.width {
									let s = tile.data[x + y * tile.width] / tile.sample_count as f32;

									out[x + tile.left + (y + tile.top) * self.settings.camera_settings.backbuffer_width] = s;
								}
							}
						}
						_ => {
							break 'collect;
						}
					}
				}

				break 'poll;
			}
			thread::sleep(Duration::from_millis(500));
		}

		out
	}

	pub fn poll(&self) -> Option<Message> {
		self.receiver.try_recv().ok()
	}

	pub fn async_await(&self) -> () {
		'collect: loop {
			match self.receiver.try_recv() {
				Ok(Message::TileProgressed(tile)) => {
					println!("CB");
					if let Some(cb) = &self.callback {
						cb(tile);
					}
				}
				_ => {
					break 'collect;
				}
			}
			thread::sleep(Duration::from_millis(200));
		}
	}
}

pub fn render_tiled(scene: Scene, settings: Settings) -> TaskHandle {
	let mut initial_tiles = Vec::new();
	let (sender, receiver) = mpsc::channel();

	// Split the backbuffer into tiles and push them into the queue
	{
		let mut x = 0;
		let mut y = 0;

		'gen_tiles: loop {
			let tile_min = (x, y);
			let tile_max = (
				(x + settings.tile_size.0).min(settings.camera_settings.backbuffer_width),
				(y + settings.tile_size.1).min(settings.camera_settings.backbuffer_height),
			);
			let width = tile_max.0 - tile_min.0;
			let height = tile_max.1 - tile_min.1;

			initial_tiles.push(Tile {
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

	let queue = Arc::new(ArrayQueue::new(initial_tiles.len()));
	for t in initial_tiles {
		queue.push(t).unwrap();
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
			let mut tile = match queue.pop() {
				Some(t) => t,
				None => {
					if 1 == thread_count.fetch_sub(1, Ordering::Relaxed) {
						sender.send(Message::Finished);
					}
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
				if context.settings.samples_per_iteration != 0 && tile.sample_count % context.settings.samples_per_iteration == 0 {
					sender.send(Message::TileProgressed(tile.clone())).unwrap();
				}
			}
		});
	}

	return TaskHandle {
		receiver,
		settings,
		alive_thread_count: thread_count,
		callback: None,
	};
}


fn generate_primary_ray(x: usize, y: usize, camera: &CameraSettings) -> Ray {
	let width = camera.backbuffer_width as f32;
	let height = camera.backbuffer_height as f32;
	let aspect = width / height;
	let x = x as f32 + (rand::random::<f32>() - 0.5);
	let y = y as f32 + (rand::random::<f32>() - 0.5);

	let px = (2.0 * ((x + 0.5) / width) - 1.0) * f32::tan(camera.fov_vert / 2.0 * PI / 180.0) * aspect;
	let py = (1.0 - 2.0 * ((y + 0.5) / height)) * f32::tan(camera.fov_vert / 2.0 * PI / 180.0);

	Ray::new(camera.transform.position, Vector3::new(px, py, 1.0).normalize())
}

fn generate_primary_ray_with_dof(x: usize, y: usize, camera: &CameraSettings) -> Ray {
	let primary = generate_primary_ray(x, y, camera);

	// chose random point on the aperture, through rejection sampling
	let start = loop {
		let r1 = rand::random::<f32>() * 2.0 - 1.0;
		let r2 = rand::random::<f32>() * 2.0 - 1.0;
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
	};
	let end = camera.transform.position + focal_plane.intersects(primary).unwrap().distance * primary.direction;

	return Ray {
		origin: start,
		direction: (end - start).normalize(),
	};
}

fn trace(ray: Ray, context: &TraceContext, depth: usize) -> Vector3 {
	return raymond_core::trace::trace(ray, &context.scene, depth, context.settings.bounce_limit);
}