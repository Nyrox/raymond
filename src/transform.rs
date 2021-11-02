use raymond_core::math::prelude::*;

#[derive(Clone, Copy, Debug)]
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
