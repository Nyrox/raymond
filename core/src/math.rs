///
/// Module to abstract away some of the library-specific algebra stuff
/// In case I ever choose to switch to nalgebra or something else
///
///

use cgmath;

pub mod types {
	pub type TFloat = f64;

	pub type Vector2 = cgmath::Vector2<TFloat>;
	pub type Vector3 = cgmath::Vector3<TFloat>;
}

pub mod consts {
	use super::types::*;

	pub const PI: TFloat = ::std::f64::consts::PI;
	pub const F_MAX: TFloat = ::std::f64::MAX;
}


pub mod prelude {

	// We need to ex-export some of cgmath's traits to make certain operations
	// usable without having to leak cgmath
	pub use cgmath::{ElementWise, InnerSpace, MetricSpace};
	pub use super::types::*;
	pub use super::consts::*;
}

