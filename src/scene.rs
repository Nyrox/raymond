use cgmath::Vector3;
use cgmath::prelude::*;
use raytracer::{Ray, Hit};

type F = f64;

#[derive(Debug, Clone)]
pub enum Material {
    Diffuse(Vector3<F>, F),
    Metal(Vector3<F>, F),
    Emission(Vector3<F>),
}

#[derive(Clone)]
pub enum Object {
    Plane(Plane),
    Sphere(Sphere),
}

impl Object {
    pub fn get_material(&self) -> &Material {
        match self {
            &Object::Plane(ref p) => &p.material,
            &Object::Sphere(ref s) => &s.material,
        }
    }

    pub fn check_ray(&self, ray: &Ray) -> Option<Hit> {
        match self {
            &Object::Plane(ref p) => p.check_ray(ray),
            &Object::Sphere(ref s) => s.check_ray(ray),
        }
    }
}

#[derive(Clone)]
pub struct Plane {
    pub origin: Vector3<F>,
    pub normal: Vector3<F>,
    pub material: Material,
}

impl Plane {
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

#[derive(Clone)]
pub struct Sphere {
    pub origin: Vector3<F>,
    pub radius: F,
    pub material: Material,
}

impl Sphere {
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

#[derive(Clone)]
pub struct Light {
    pub position: Vector3<F>,
    pub intensity: Vector3<F>,
}

#[derive(Clone)]
pub struct Scene {
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
}

impl Scene {
    pub fn new() -> Scene {
        Scene { objects: Vec::new(), lights: Vec::new() }
    }
}