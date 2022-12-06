use minifb::{Key, Window, WindowOptions};

use std::{
	sync::mpsc::{self, Receiver, Sender},
	thread,
};

pub struct WatcherHandle {
	window: minifb::Window,
	width: usize,
	height: usize,
	buffer: Vec<u32>,
	wants_to_close: bool,
}

impl WatcherHandle {
	pub fn update(&mut self) {
		if self.window.is_key_down(Key::Escape) {
			self.wants_to_close = true
		}
		self.window.update()
	}

	pub fn is_open(&self) -> bool {
		self.window.is_open() && (!self.wants_to_close)
	}

	pub fn progress_tile(&mut self, tile: raymond_core::Tile) {
		for y in 0..tile.height {
			for x in 0..tile.width {
				let data = tile.data[x + y * tile.width];
				let x = tile.left + x;
				let y = tile.top + y;

				let map = |p| {
					let exposure = 1.0;
					let gamma = 2.2;

					let p = p / (tile.sample_count as f32);

					let tone_mapped = 1.0 - f32::exp(p * -1.0 * exposure);
					let tone_mapped = tone_mapped.powf(1.0 / gamma);

					(tone_mapped.min(1.0).max(0.0) * 255.0) as u32
				};

				let ix = map(data.x);
				let iy = map(data.y);
				let iz = map(data.z);
				let data = ix << 16 | iy << 8 | iz << 0 | 0xFF << 24;

				self.buffer[(x + y * self.width)] = data;
			}
		}
		self.window.update_with_buffer(&self.buffer, self.width, self.height).unwrap();
	}
}

pub fn start(width: usize, height: usize) -> Result<WatcherHandle, minifb::Error> {
	let mut buffer: Vec<u32> = vec![0; width * height];

	let mut window = Window::new("Raymond - Render In Progress", width, height, WindowOptions::default()).unwrap();

	// 60 fps
	window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
	window.set_background_color(0, 0, 0);

	Ok(WatcherHandle {
		wants_to_close: false,
		window,
		buffer,
		width,
		height,
	})
}
