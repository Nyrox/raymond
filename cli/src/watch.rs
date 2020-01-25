use minifb::{Key, Window, WindowOptions};

use std::{
	sync::mpsc::{self, Receiver, Sender},
	thread,
};

pub struct WatcherHandle {
	sender: Sender<ToWatcher>,
	receiver: Receiver<FromWatcher>,
	wants_to_close: bool,
}

impl WatcherHandle {
	pub fn update(&mut self) {
		while let Ok(message) = self.receiver.try_recv() {
			match message {
				FromWatcher::CloseRequest => self.wants_to_close = true,
				_ => unimplemented!(),
			}
		}
	}

	pub fn wants_to_close(&self) -> bool {
		self.wants_to_close
	}

	pub fn send_tile(&self, tile: core::Tile) {
		self.sender.send(ToWatcher::TileProgress(tile)).unwrap()
	}
}

pub enum ToWatcher {
	TileProgress(core::Tile)
}

pub enum FromWatcher {
	CloseRequest,
}

pub fn start_watcher(width: usize, height: usize) -> Result<WatcherHandle, minifb::Error> {
	let (out_sender, out_receiver) = mpsc::channel::<FromWatcher>();
	let (in_sender, in_receiver) = mpsc::channel::<ToWatcher>();

	thread::spawn(move || {
		let mut buffer: Vec<u32> = vec![0; width * height];

		let mut window = Window::new("Raymond - Render In Progress", width, height, WindowOptions::default()).unwrap();

		// 60 fps
		window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
		window.set_background_color(0, 0, 0);

		while window.is_open() {
			if window.is_key_down(Key::Escape) {
				out_sender.send(FromWatcher::CloseRequest).unwrap();
			}

			match in_receiver.try_recv() {
				Ok(ToWatcher::TileProgress(tile)) => {
					for y in 0..tile.height {
						for x in 0..tile.width {
							let data = tile.data[x + y * tile.width];
							let x = tile.left + x;
							let y = tile.top + y;

							let map = |p| {
								let exposure = 1.0;
								let gamma = 2.2;

								let p = p / (tile.sample_count as f64);

								let tone_mapped = 1.0 - f64::exp(p * -1.0 * exposure);
								let tone_mapped = tone_mapped.powf(1.0 / gamma);

								(tone_mapped.min(1.0).max(0.0) * 255.0) as u32
							};

							let ix = map(data.x);
							let iy = map(data.y);
							let iz = map(data.z);
							let data = ix << 16 | iy << 8 | iz << 0 | 0xFF << 24;

							buffer[(x + y * width)] = data;

						}
					}
					window.update_with_buffer(&buffer, width, height).unwrap();
				}
				Err(_) => (),
				_ => unimplemented!(),
			}


			window.update();
		}
		println!("Window thread exited");
	});

	Ok(WatcherHandle {
		sender: in_sender,
		receiver: out_receiver,
		wants_to_close: false,
	})
}
