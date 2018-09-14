use cgmath::*;

use super::{F};

#[derive(Debug, Clone)]
pub enum Material {
    Diffuse(Vector3<F>, F),
    Metal(Vector3<F>, F),
    Emission(Vector3<F>, Vector3<F>, F, F),
}