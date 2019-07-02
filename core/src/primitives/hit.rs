use crate::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Hit {
	pub distance: f64,
	pub ray: Ray,
	pub subobject_index: usize,
}

impl Hit {
	pub fn new(ray: Ray, distance: f64) -> Hit {
		Hit {
			ray,
			distance,
			subobject_index: 0,
		}
	}

	pub fn with_child(ray: Ray, distance: f64, subobject_index: usize) -> Hit {
		Hit {
			ray,
			distance,
			subobject_index,
		}
	}
}
