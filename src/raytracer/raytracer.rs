use std::sync::RwLock;
use std::marker;
use std::f64;
use std::default::Default;

use rand;
use crossbeam_utils;
use cgmath::*;
use num_cpus;

use super::scene::*;
use super::primitives::{Ray, Hit};
use super::material::Material;

use super::{F, PI, F_MAX};



#[derive(Clone, Debug)]
pub struct RaytracerConfig {
    pub num_workers: usize,
    pub max_bounces: usize,
    pub num_samples: usize,
    pub width: usize,
    pub height: usize,
    pub fov: F,
    pub camera_pos: Vector3<F>,
}

impl Default for RaytracerConfig {
    fn default() -> Self {
        RaytracerConfig {
            num_workers: num_cpus::get(),
            max_bounces: 4,
            num_samples: 16,
            width: 0,
            height: 0,
            camera_pos: Vector3::new(0.0, 0.0, 0.0),
            fov: 65.0,
        }
    }
}


impl RaytracerConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn launch(self, scene: Scene) -> Vec<Vector3<u8>> {
        let mut thread_handles = Vec::new();
        let THREAD_COUNT = self.num_workers;

        let image = RwLock::new(vec![Vector3::new(0.0, 0.0, 0.0); self.width * self.height]);

        
        for i in 0..THREAD_COUNT {
            let raytracer = Raytracer { 
                config: self.clone(),
                scene: scene.clone(),
                image: &image,
            };
            // let result = raytracer.trace(raytracer.generate_primary_ray(335, 99), raytracer.config.camera_pos, 1);
            // println!("Result: {:?}", result);
            // panic!();
            unsafe {
                thread_handles.push(crossbeam_utils::thread::spawn_unchecked(move || {
                    let start = raytracer.config.height / THREAD_COUNT * i;
                    for y in start..(start + raytracer.config.height / THREAD_COUNT) {
                        for x in 0..raytracer.config.width {
                            let mut result = Vector3::new(0.0, 0.0, 0.0);
                            for _ in 0..raytracer.config.num_samples {
                                let ray = raytracer.generate_primary_ray(x, y);
                                result += raytracer.trace(ray, raytracer.config.camera_pos, 1);
                            }
                            result /= raytracer.config.num_samples as F;
                            raytracer.image.write().unwrap()[x + y * raytracer.config.width] = result;
                        }

                        println!("Finished line {} of {}", y, raytracer.config.height);
                    }
                }));
            }
        }

        for h in thread_handles {
            h.join().unwrap();
        }

        let mut export = vec![Vector3::new(0, 0, 0); self.width * self.height];
        for (i, p) in image.into_inner().unwrap().iter().enumerate() {
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
}



pub fn element_wise_map<Fun: Fn(F) -> F>(vec: &Vector3<F>, f: Fun) -> Vector3<F> {
    Vector3::new(f(vec.x), f(vec.y), f(vec.z))
}

#[derive(Clone)]
pub struct Raytracer<'a> {
    pub image: &'a RwLock<Vec<Vector3<F>>>,
    pub config: RaytracerConfig,
    pub scene: Scene,
}

unsafe impl<'a> marker::Send for Raytracer<'a> {}
unsafe impl<'a> marker::Sync for Raytracer<'a> {}

impl<'a> Raytracer<'a> {
    fn intersect(&self, ray: Ray) -> Option<(&Object, Hit)> {
        let mut closest_distance = F_MAX;
        let mut closest_object = None;

        for (i, object) in self.scene.objects.iter().enumerate() {
            match object.intersects(ray) {
                Some(hit) => {
                    if hit.distance < closest_distance {
                        closest_distance = hit.distance;
                        closest_object = Some((i, hit));
                    }
                }
                None => ()
            }
        }

        match closest_object {
            Some((o, h)) => return Some((&self.scene.objects[o], h)),
            _ => return None
        }
    }

    fn trace(&self, ray: Ray, camera_pos: Vector3<F>, depth: usize) -> Vector3<F> {
        if depth > self.config.max_bounces {
            return Vector3::new(0.0, 0.0, 0.0);
        }
        
        let intersect = self.intersect(ray);
        let (object, hit) = match intersect {
            Some((o, h)) => { (o, h) }
            None => return Vector3::new(0.0, 0.0, 0.0)
        };

        let surface_properties = object.get_surface_properties(hit);
        let normal = surface_properties.normal;
        let fragment_position = ray.origin + ray.direction * hit.distance;

        let (material_color, material_roughness, material_metalness) = match object.get_material() {
            Material::Diffuse(color, roughness) => (color, *roughness, 0.0),
            Material::Metal(color, roughness) => (color, *roughness, 1.0),
            Material::Emission(e, _, _, _) => { return *e; }
            _ => panic!()
        };

        let view_dir = (camera_pos - fragment_position).normalize();
        let f0 = Vector3::new(0.04, 0.04, 0.04);
        let f0 = Self::lerp_vec(f0, *material_color, material_metalness);

        // Decide whether to sample diffuse or specular
        let r = rand::random::<F>();
        let local_cartesian = Self::create_coordinate_system_of_n(normal);
        let local_cartesian_transform = Matrix3::from_cols(local_cartesian.0, normal, local_cartesian.1);

        let prob_d = Self::lerp(0.5, 0.0, material_metalness);

        if r < prob_d {
            let (sample, pdf) = self.uniform_sample_hemisphere();
            let sample_world = (local_cartesian_transform * sample).normalize();

            let radiance = self.trace(Ray { origin: fragment_position + normal * 0.00001, direction: sample_world }, fragment_position, depth + 1);

            let cos_theta = normal.dot(sample_world).max(0.0);
            let halfway = (sample_world + view_dir).normalize();

            let fresnel = Self::fresnel_schlick(halfway.dot(view_dir).max(0.0), f0);
            
            let specular_part = fresnel;
            let mut diffuse_part = Vector3::new(1.0, 1.0, 1.0) - specular_part;

            diffuse_part *= 1.0 - material_metalness;

            let output = (diffuse_part.mul_element_wise(*material_color) / PI).mul_element_wise(radiance) * cos_theta;

            return output / prob_d / pdf;
        }
        else {
            // Sample specular
            let reflect = (-view_dir - 2.0 * (-view_dir.dot(normal) * normal)).normalize();
            fn importance_sample_ggx(reflect: Vector3<F>, roughness: F) -> Vector3<F> {
                let r1: F = rand::random();
                let r2: F = rand::random();

                let a = roughness*roughness;
                let phi = 2.0 * PI * r1;
                let cos_theta = ((1.0 - r2) / (1.0 + (a*a - 1.0) * r2)).sqrt();
                let sin_theta = (1.0 - cos_theta*cos_theta).sqrt();

                let h = Vector3::new(phi.cos() * sin_theta, phi * sin_theta, cos_theta);

                let (tangent, bitangent) = Raytracer::create_coordinate_system_of_n(h);

                return (tangent * h.x + bitangent * h.y + reflect * h.z).normalize();
            }
            let sample_world = importance_sample_ggx(reflect, material_roughness);
            
            let radiance = self.trace(Ray { origin: fragment_position + normal * 0.0001, direction: sample_world }, fragment_position, depth + 1);
            let cos_theta = normal.dot(sample_world);
            let light_dir = sample_world.normalize();
            let halfway = (light_dir + view_dir).normalize();
            let F = Self::fresnel_schlick(halfway.dot(view_dir), f0);
            let D = Self::ggx_distribution(normal, halfway, material_roughness);
            let G = Self::geometry_smith(normal, view_dir, sample_world, material_roughness);

            let nominator = D * G * F;
            let denominator = 4.0 * normal.dot(view_dir) * cos_theta + 0.001;
            let specular = nominator / denominator;

            let output = (specular).mul_element_wise(radiance) * cos_theta;

            // pdf
            let pdf = {
                (D * normal.dot(halfway)) / (4.0 * halfway.dot(view_dir)) + 0.0001
            };
            return output / (1.0 - prob_d) / pdf;
        }
    }

    fn ggx_distribution(n: Vector3<F>, h: Vector3<F>, roughness: F) -> F {
        let a2 = roughness*roughness;
        let NdotH = n.dot(h);

        let nominator = a2;
        let denominator = NdotH.powf(2.0) * (a2 - 1.0) + 1.0;
        let denominator = (PI * denominator * denominator).max(1e-7);
        return nominator / denominator;
    }

    fn geometry_schlick_ggx(n: Vector3<F>, v: Vector3<F>, r: F) -> F {
        let numerator = n.dot(v).max(0.0);
        let k = (r*r) / 8.0;
        let denominator =  numerator * (1.0 - k) + k;

        return numerator / denominator;
    }

    fn geometry_smith(n: Vector3<F>, v: Vector3<F>, l: Vector3<F>, r: F) -> F {
        return Self::geometry_schlick_ggx(n, v, r) * Self::geometry_schlick_ggx(n, l, r);
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

    fn uniform_sample_hemisphere(&self) -> (Vector3<F>, F) {
        let r1 = rand::random::<F>();
        let r2 = rand::random::<F>();

        let theta = (r1.sqrt()).acos();
        let phi = 2.0 * PI * r2;

        let pdf = r1.sqrt() / PI;        
        let cartesian = Vector3::new(theta.sin() * phi.cos(), theta.cos(), theta.sin() * phi.sin());
        return (cartesian, pdf);
    }
    
    fn create_coordinate_system_of_n(n: Vector3<F>) -> (Vector3<F>, Vector3<F>) {
        let sign = if n.z > 0.0 { 1.0 } else { -1.0 };
        let a = -1.0 / (sign + n.z);
        let b = n.x * n.y * a;
        return (
            Vector3::new(1.0 + sign * n.x * n.x * a, sign * b, -sign * n.x),
            Vector3::new(b, sign + n.y * n.y * a, -n.y),
        );
    }

    fn generate_primary_ray(&self, x: usize, y: usize) -> Ray {
        let width = self.config.width as F; let height = self.config.height as F;
        let aspect = width / height;
        let x = x as F + (rand::random::<F>() - 0.5); let y = y as F + (rand::random::<F>() - 0.5);

        let px = (2.0 * ((x + 0.5) / width) - 1.0) * F::tan(self.config.fov / 2.0 * PI / 180.0) * aspect;
        let py = (1.0 - 2.0 * ((y + 0.5) / height)) * F::tan(self.config.fov / 2.0 * PI / 180.0);

        Ray::new(self.config.camera_pos, Vector3::new(px, py, 1.0).normalize())
    }
}