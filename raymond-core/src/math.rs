///
/// Module to abstract away some of the library-specific algebra stuff
/// In case I ever choose to switch to nalgebra or something else
///
///

pub mod types {
	pub type TFloat = f32;

	pub type Vector2 = cgmath::Vector2<TFloat>;
	pub type Vector3 = cgmath::Vector3<TFloat>;
}

pub mod consts {
	use super::types::*;

	pub const PI: TFloat = ::std::f32::consts::PI;
	pub const F_MAX: TFloat = ::std::f32::MAX;
}

pub mod prelude {

	// We need to ex-export some of cgmath's traits to make certain operations
	// usable without having to leak cgmath
	pub use super::{consts::*, types::*};
	pub use cgmath::{ElementWise, InnerSpace, MetricSpace};
}
