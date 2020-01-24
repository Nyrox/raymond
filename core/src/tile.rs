use serde::{Serialize, Deserialize};
use crate::math::prelude::*;

use std::fmt;

#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
	pub sample_count: usize,
	pub width: usize,
	pub height: usize,
	pub left: usize,
	pub top: usize,
	pub data: Vec<Vector3>,
}

impl fmt::Debug for Tile {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
		write!(
			f,
			"Tile {{ sample_count: {}, width: {}, height: {}, left: {}, top: {} }}",
			self.sample_count, self.width, self.height, self.left, self.top
		)
	}
}
