pub mod watch;

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "raymond-cli")]
enum RaymondCli {
	Render {
		#[structopt(long)]
		watch: bool,
		#[structopt(name = "SCENE")]
		scene: PathBuf,
	},
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let cli = RaymondCli::from_args();
	println!("{:?}", cli);

	match cli {
		RaymondCli::Render { watch, scene } => {
			println!("{:?}, {:?}", watch, scene);

			let handle = raytracer::trace::render_tiled(scene, settings);


			if watch {
				let mut watcher = watch::start_watcher(1280, 720)?;

				'wait: loop {
					watcher.update();

					if watcher.wants_to_close() {
						break 'wait;
					}

					while let Some(tile) = handle.poll() {
						match tile {
							raytracer::trace::Message::TileProgressed(tile) => {
								watcher.send_tile(tile);
							},
							_ => unimplemented!()
						}
					}
				}
			} else {
			}
		}
		_ => unimplemented!(),
	}

	Ok(())
}
