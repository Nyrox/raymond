use raytracer::{acc_grid, mesh::Mesh, scene::Light};

use core::{prelude::*, primitives::*, Scene, SceneObject, Settings};
use std::net::{TcpListener, TcpStream};

use cgmath::Vector3;
use std::cell::RefCell;

use raytracer::trace;

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

use serde::{Deserialize, Serialize};
use std::{io::prelude::*, rc::Rc};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "cli")]
enum Opt {
	#[structopt(name = "dump-default-config")]
	DumpConfig {
		#[structopt(short = "o", long, default_value = "default-config.ron")]
		out: PathBuf,
	},
	#[structopt(name = "render")]
	Render {
		#[structopt(default_value = "./", short = "s", long)]
		scene: PathBuf,
		#[structopt(short = "c", long)]
		config: Option<PathBuf>,
		#[structopt(long)]
		host: Option<String>,
	},
}

use std::mem;

fn main() -> Result<(), Box<dyn ::std::error::Error>> {
	use simplelog::*;
	CombinedLogger::init(vec![
		WriteLogger::new(
			LevelFilter::Trace,
			Config::default(),
			File::create("trace.log").unwrap(),
		),
		TermLogger::new(LevelFilter::Info, Config::default(), TerminalMode::Mixed).unwrap(),
	]);

	match Opt::from_args() {
		Opt::DumpConfig { out } => {
			let mut f = File::create(&out)?;
			let settings = core::settings::Settings::default();
			let settings = ron::ser::to_string_pretty(&settings, Default::default())?;

			f.write(settings.as_bytes())?;
		}
		Opt::Render {
			config,
			scene,
			host,
		} => {
			// Load the tracer settings
			let settings = match config {
				Some(ref p) => Settings::load(p)?,
				None => Settings::default(),
			};

			// Try and load the scene file
			let scene = match &scene {
				_ if scene.is_file() => Scene::load(&scene)?,
				_ if scene.is_dir() => {
					unimplemented!("Loading scenes from a directory is not yet supported")
				}
				// There probably exists some realm where this is the case
				_ => panic!("Expected [context] to be a file or a directory"),
			};

			let now = Instant::now();

			let mut task_handle = trace::render_tiled(scene, settings.clone());

			#[derive(Clone, Serialize, Deserialize)]
			pub struct MessageHeader {
				pub sample_count: u32,
				pub width: u32,
				pub height: u32,
				pub left: u32,
				pub top: u32,
			}

			#[derive(Clone, Serialize, Deserialize)]
			pub struct MessageBody<'a> {
				#[serde(with = "serde_bytes")]
				pub data: &'a [u8],
			}

			use log::info;
			info!("Message Header Length: {}", mem::size_of::<MessageHeader>());

			let final_image = match host {
				Some(ref hostname) => {
					let listener = TcpListener::bind(hostname)?;
					listener.set_nonblocking(true);
					let mut streams = Vec::new();

					let mut out = Rc::new(RefCell::new(vec![
						Vector3::new(0.0, 0.0, 0.0);
						settings.camera.backbuffer_height
							* settings.camera.backbuffer_width
					]));
					let _out = out.clone();
					let settings = settings.clone();

					task_handle.set_callback(Some(Box::new(move |mut tile| {
						let binaryData = unsafe {
							std::slice::from_raw_parts(
								tile.data.as_mut_ptr() as *mut u8,
								mem::size_of::<Vector3<f64>>() * tile.data.len(),
							)
						};

						for y in 0..tile.height {
							for x in 0..tile.width {
								let bF = (binaryData.as_ptr() as *const f64);
								let s = unsafe {
									Vector3::new(
										*(bF.offset((3 * (x + y * tile.width) + 0) as isize)),
										*(bF.offset((3 * (x + y * tile.width) + 1) as isize)),
										*(bF.offset((3 * (x + y * tile.width) + 2) as isize)),
									)
								};

								let s = tile.data[x + y * tile.width] / tile.sample_count as f64;

								out.borrow_mut()[x
									+ tile.left + (y + tile.top)
									* settings.camera.backbuffer_width] = s;
							}
						}

						match listener.accept() {
							Ok((socket, addr)) => {
								println!("Received new connection on addr: {}", addr);
								socket.set_nonblocking(false);
								streams.push(socket);
							}
							Err(_) => {}
						}

						for (i, stream) in streams.iter_mut().enumerate() {
							let header = MessageHeader {
								sample_count: tile.sample_count as u32,
								width: tile.width as u32,
								height: tile.height as u32,
								left: tile.left as u32,
								top: tile.top as u32,
							};

							let body = MessageBody {
								data: unsafe {
									std::slice::from_raw_parts(
										tile.data.as_mut_ptr() as *mut u8,
										mem::size_of::<Vector3<f64>>() * tile.data.len(),
									)
								},
							};

							// handle outgoing
							// let json = rmp_serde::to_vec(&message).expect("suka blyat");
							let header_data = unsafe {
								std::slice::from_raw_parts(
									(&header as *const MessageHeader) as *const u8,
									mem::size_of::<MessageHeader>(),
								)
							};

							let body_data = body.data;

							stream.write_all(&header_data).expect("failed to send header");
							stream.write_all(&body_data).expect("failed to send body");
						}
					})));

					task_handle.async_await();
					let v = _out.borrow().clone();
					v
				}
				None => task_handle.r#await(),
			};

			fn element_wise_map<Fun: Fn(f64) -> f64>(vec: &Vector3<f64>, f: Fun) -> Vector3<f64> {
				Vector3::new(f(vec.x), f(vec.y), f(vec.z))
			}

			let mut export = vec![
				Vector3::new(0, 0, 0);
				settings.camera.backbuffer_width
					* settings.camera.backbuffer_height
			];
			for (i, p) in final_image.iter().enumerate() {
				let exposure = 1.0;
				let gamma = 2.2;
				let tone_mapped = Vector3::new(1.0, 1.0, 1.0)
					- element_wise_map(&(p * -1.0 * exposure), |e| f64::exp(e));
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
				"Finished render.\nTotal render time: {}s\n",
				(Instant::now() - now).as_millis() as f32 / 1000.0,
			);

			let image: image::ImageBuffer<image::Rgb<u8>, Vec<u8>> = image::ImageBuffer::from_fn(
				settings.camera.backbuffer_width as u32,
				settings.camera.backbuffer_height as u32,
				|x, y| {
					image::Rgb(
						export[(x + y * settings.camera.backbuffer_width as u32) as usize].into(),
					)
				},
			);
			image
				.save("output.png")
				.expect("Failed to save buffer to disk");
		}
	};

	return Ok(());
}
