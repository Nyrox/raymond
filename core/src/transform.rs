use crate::Vector3;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct Transform {
	pub position: Vector3,
}

impl Transform {
	pub fn identity() -> Self {
		Transform {
			position: Vector3::new(0.0, 0.0, 0.0),
		}
	}
}

impl Default for Transform {
	fn default() -> Self {
		Self::identity()
	}
}
