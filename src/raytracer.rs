use scene::*;
use num_traits;
use cgmath::prelude::*;
use cgmath::*;
use std::cell::RefCell;
use rand;

use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

type F = f32;
const PI: F = 3.141592;

pub struct Ray {
    pub origin: Vector3<F>,
    pub direction: Vector3<F>,
}

#[derive(Debug)]
pub struct Hit {
    pub position: Vector3<F>,
    pub normal: Vector3<F>,
}

impl Default for Hit {
    fn default() -> Hit {
        Hit { position: Vector3::new(0.0, 0.0, 0.0), normal: Vector3::new(0.0, 0.0, 0.0) }
    }
}

impl Ray {
    pub fn new (origin: Vector3<F>, direction: Vector3<F>) -> Ray {
        Ray { origin, direction }
    }
}

pub fn element_wise_map<Fun: Fn(F) -> F>(vec: &Vector3<F>, f: Fun) -> Vector3<F> {
    Vector3::new(f(vec.x), f(vec.y), f(vec.z))
}

pub fn element_wise_division(left: &Vector3<F>, right: &Vector3<F>) -> Vector3<F> {
    Vector3::new(left.x / right.x, left.y / right.y, left.z / right.z)
}

pub struct Raytracer<'a> {
    pub image: Vec<Vector3<F>>,
    pub width: usize,
    pub height: usize,
    pub fov: F,
    pub scene: &'a RefCell<Scene>,
}


impl<'a> Raytracer<'a> {
    pub fn new(width: usize, height: usize, fov: F, scene: &'a RefCell<Scene>) -> Raytracer<'a> {
        Raytracer {
            image: vec!(Vector3::new(0.0, 0.0, 0.0); (width * height) as usize),
            width, height,
            fov,
            scene
        }
    }

    pub fn render(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let ray = self.generate_primary_ray(x, y);

                self.image[x + y * self.width] = self.trace(&ray, Vector3::new(0.0, 0.0, 0.0), 3).1;
            }

            println!("Finished line: {} of {}", y, self.height);
        }
    }

    fn trace(&mut self, ray: &Ray, camera_pos: Vector3<F>, bounces: usize) -> (F, Vector3<F>) {
        super::TRACE_COUNT.fetch_add(1, Ordering::Relaxed);

        let mut closest: (F, usize, Hit) = (123125.0, 12958125, Hit::default());

        for (i, object) in self.scene.borrow().objects.iter().enumerate() {
            match object.check_ray(ray) {
                Some(hit) => { 
                    let distance = ray.origin.distance(hit.position);
                    if distance < closest.0 {
                        closest = (distance, i, hit);
                    }
                }
                None => continue,
            }
        }

        if closest.1 == 12958125 { return (0.001, Vector3::new(0.0, 0.0, 0.0)) };
        
        let scene = self.scene.borrow();
        let object = &scene.objects[closest.1];
        let hit = closest.2;

        let mut total = Vector3::new(0.0, 0.0, 0.0);
        
        match object.get_material() {
            Material::Emission(intensity) => { return (closest.0, intensity); },
            _ => {}
        }
        let (material_color, material_roughness, material_metalness) = match object.get_material() {
            Material::Diffuse(color, roughness) => (color, roughness, 0.0),
            Material::Metal(color, roughness) => (color, roughness, 1.0),
            _ => panic!()
        };

        // Direct lighting
        'light_iter: for light in scene.lights.iter() {
            // Shadow
            let now = Instant::now();

            for (index, shadow_object) in scene.objects.iter().enumerate() {
                if closest.1 == index { continue; }
                super::SHADOW_RAY_COUNT.fetch_add(1, Ordering::Relaxed);

                let shadow_hit = shadow_object.check_ray(&Ray { origin: light.position, direction: (hit.position - light.position).normalize() });
                if let Some(shadow_hit) = shadow_hit {
                    if shadow_hit.position.distance(light.position) < light.position.distance(hit.position) {
                        continue 'light_iter;
                    }
                }
            }

            super::SHADOW_TOTAL_TIME.fetch_add((Instant::now() - now).as_nanos() as usize, Ordering::Relaxed);

            let distance = light.position.distance(hit.position);
            let attenuation = 1.0 / (distance * distance);
            let cos_theta = hit.normal.dot((light.position - hit.position).normalize()).max(0.0);
            
            total += light.intensity * cos_theta * attenuation;
        }

        if bounces == 0 { return (closest.0, total.mul_element_wise(material_color)); }

        // Indirect lighting
        const N: usize = 8;

        let mut total_indirect = Vector3::new(0.0, 0.0, 0.0);
        let local_cartesian = self.create_coordinate_system_of_n(hit.normal);
        let local_cartesian_transform = Matrix3::from_cols(local_cartesian.0, hit.normal, local_cartesian.1);
        
        let view_dir = (camera_pos - hit.position).normalize();
        let f0 = Vector3::new(0.04, 0.04, 0.04);
        let f0 = Self::lerp_vec(f0, material_color, material_metalness);

        for i in 0..N {
            let sample = self.uniform_sample_hemisphere();
            let sample_world = (local_cartesian_transform * sample).normalize();

            let incoming = self.trace(&Ray { origin: hit.position + sample_world * 0.001, direction: sample_world }, hit.position, bounces - 1);

            let incoming_radiance = incoming.1;
            let distance = incoming.0;
            let cos_theta = hit.normal.dot(sample_world).max(0.0);
            let attenuation = 1.0 / (distance * distance);
            let radiance = incoming_radiance * attenuation;

            let light_dir = sample_world.normalize();
            let halfway = (light_dir + view_dir).normalize();

            let fresnel = Self::fresnel_schlick(halfway.dot(view_dir).max(0.0), f0);

            let specular_part = fresnel;
            let mut diffuse_part = Vector3::new(1.0, 1.0, 1.0) - specular_part;

            diffuse_part *= 1.0 - material_metalness;

            let D = Self::ggx_distribution(hit.normal, halfway, material_roughness);
            let G = Self::geometry_smith(hit.normal, view_dir, sample_world, material_roughness);

            let nominator = D * G * fresnel;
            let denominator = 4.0 * hit.normal.dot(view_dir).max(0.0) * cos_theta + 0.001;
            let specular = nominator / denominator;

            let output = (diffuse_part.mul_element_wise(material_color) / PI + specular).mul_element_wise(radiance) * cos_theta;

            total_indirect += radiance / 10.0;
        }
        total_indirect /= N as F * (1.0 / (2.0 * PI));

        return (closest.0, (total + total_indirect));
    }

    fn ggx_distribution(n: Vector3<F>, h: Vector3<F>, roughness: F) -> F {
        let nominator = roughness.powf(2.0);
        let denominator = n.dot(h).max(0.0).powf(2.0) * (roughness.powf(2.0) - 1.0) + 1.0;
        let denominator = PI * denominator * denominator;
        return nominator / denominator;
    }

    fn geometry_schlick_ggx(n: Vector3<F>, v: Vector3<F>, k: F) -> F {
        let n_dot_v = n.dot(v).max(0.0);
        let k = (k*k) / 8.0;

        let nominator = n_dot_v;
        let denominator = n_dot_v * (1.0 - k) + k;

        return nominator / denominator;
    }

    fn geometry_smith(n: Vector3<F>, v: Vector3<F>, l: Vector3<F>, k: F) -> F {
        return Self::geometry_schlick_ggx(n, v, k) * Self::geometry_schlick_ggx(n, l, k);
    }

    fn fresnel_schlick(cos_theta: F, F0: Vector3<F>) -> Vector3<F> {
        return F0 + (Vector3::new(1.0, 1.0, 1.0) - F0) * (1.0 - cos_theta).powf(5.0);
    }

    fn lerp_vec(min: Vector3<F>, max: Vector3<F>, a: F) -> Vector3<F> {
        Vector3::new(Self::lerp(min.x, max.x, a), Self::lerp(min.y, max.y, a), Self::lerp(min.z, max.z, a))
    }

    fn lerp(min: F, max: F, a: F) -> F {
        min + a * (max - min)
    }

    fn uniform_sample_hemisphere(&self) -> Vector3<F> {
        let r1 = rand::random::<F>();
        let r2 = rand::random::<F>();

        let sin_theta = (1.0 - r1 * r1).sqrt();
        let phi = 2.0 * PI * r2;
        let x = sin_theta * phi.cos();
        let z = sin_theta * phi.sin();
        return Vector3::new(x, r1, z);
    }
    
    fn create_coordinate_system_of_n(&self, n: Vector3<F>) -> (Vector3<F>, Vector3<F>) {
        let mut nt;
        if n.x.abs() > n.y.abs() {
            nt = Vector3::new(n.z, 0.0, -n.x).normalize();
        }
        else {
            nt = Vector3::new(0.0, -n.z, n.y).normalize();
        }
        let nb = n.cross(nt);

        return (nt, nb);
    }
    

    // Exports the internal HDR format to RGB for saving or display
    pub fn export_image(&self) -> Vec<Vector3<u8>> {
        let mut export: Vec<Vector3<u8>> = vec!(Vector3::new(0, 0, 0); (self.width * self.height) as usize);

        for (i, p) in self.image.iter().enumerate() {
            // let tone_mapped = element_wise_division(p , &(p + Vector3::new(1.0, 1.0, 1.0)));
            let exposure = 1.0;
            let gamma = 2.2;
            let tone_mapped = Vector3::new(1.0, 1.0, 1.0) - element_wise_map(&(p * -1.0 * exposure), |e| F::exp(e));
            let tone_mapped = element_wise_map(&tone_mapped, |x| x.powf(1.0 / gamma));

            for i in 0..3 {
                let e = tone_mapped[i];
                if e > 1.0 || e < 0.0 {
                    println!("Problem: {:?}", e);
                }
            }
            
            match (tone_mapped * 255.0).cast() {
                Some(v) => { export[i] = v; },
                None => { println!("PROBLEM: {:?}", tone_mapped * 255.0) }
            };
        }

        return export;
    }

    fn generate_primary_ray(&self, x: usize, y: usize) -> Ray {
        let width = self.width as f32; let height = self.height as f32;
        let x = x as f32; let y = y as f32;
        let aspect = width / height;

        let px = (2.0 * ((x + 0.5) / width) - 1.0) * F::tan(self.fov / 2.0 * PI / 180.0) * aspect;
        let py = (1.0 - 2.0 * ((y + 0.5) / height)) * F::tan(self.fov / 2.0 * PI / 180.0);

        Ray::new(Vector3::new(0.0, 0.0, 0.0), Vector3::new(px, py, 1.0).normalize())
    }
}