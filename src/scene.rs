use cgmath::Vector3;
use cgmath::prelude::*;
use raytracer::{Ray, Hit};
type F = f32;



#[derive(Debug, Clone)]
pub struct Material {
    pub color: Vector3<F>,
}

pub trait Object {
    fn get_material(&self) -> Material;
    fn check_ray(&self, &Ray) -> Option<Hit>;
}

pub struct Plane {
    pub origin: Vector3<F>,
    pub normal: Vector3<F>,
    pub material: Material,
}

impl Object for Plane {
    fn get_material(&self) -> Material {
        self.material.clone()
    }

    fn check_ray(&self, ray: &Ray) -> Option<Hit> {
        let normal = self.normal;

        let denom = normal.dot(-ray.direction);
        if denom > 1e-6 {
            let p0l0 = self.origin - ray.origin;
            let t = p0l0.dot(-normal) / denom;
            if t >= 0.0 {
                let hit_pos = ray.origin + ray.direction * t;
                return Some(Hit { position: hit_pos, normal });
            }
        }

        return None;
    }
}

pub struct Sphere {
    pub origin: Vector3<F>,
    pub radius: F,
    pub material: Material,
}

impl Object for Sphere {
    fn get_material(&self) -> Material {
        self.material.clone()
    }
    
    fn check_ray(&self, ray: &Ray) -> Option<Hit> {
        let c = self.origin - ray.origin;
        let mut t = c.dot(ray.direction);
        let q = c - t * ray.direction;
        let p = q.dot(q);
        
        if p > self.radius * self.radius {
            return None;
        }

        t -= (self.radius * self.radius - p).sqrt();
        if t <= 0.0 { return None; }
        
        let hit_pos = ray.origin + ray.direction * t;
        return Some(Hit { position: hit_pos, normal: (hit_pos - self.origin).normalize() });
    }
}

pub struct Light {
    pub position: Vector3<F>,
    pub intensity: Vector3<F>,
}

pub struct Scene {
    pub objects: Vec<Box<dyn Object>>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene { objects: Vec::new(), lights: Vec::new() }
    }
}