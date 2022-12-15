use cgmath::InnerSpace;

use crate::{Vector3, prelude::PI};



pub trait CookTorrance {
    fn fresnel(cos_theta: f32, f0: Vector3) -> Vector3;
    fn microfacet_distribution(n_dot_h: f32, m: f32) -> f32;
    fn geometric_attenuation(wo: Vector3, wi: Vector3, n: Vector3, m: f32) -> f32;
    fn importance_sample(wo: Vector3, roughness: f32) -> (Vector3, f32);
}


pub struct DefaultCookTorrance;

impl DefaultCookTorrance {
    pub fn roughness_to_alpha(roughness: f32) -> f32 {
        let x = roughness.max(1e-3);
        return x * x * 2.0f32.sqrt();
    }
}

impl CookTorrance for DefaultCookTorrance {
    /// Schlick Approximation
    fn fresnel(cos_theta: f32, f0: Vector3) -> Vector3 {
        return f0 + (Vector3::new(1.0, 1.0, 1.0) - f0) * (1.0 - cos_theta).powf(5.0);
    }

    // Beckman distribution
    // https://digibug.ugr.es/bitstream/handle/10481/19751/rmontes_LSI-2012-001TR.pdf;jsessionid=519D0B94111D8A3C4F85A3AB22A92F6C?sequence=1
    fn microfacet_distribution(n_dot_h: f32, roughness: f32) -> f32 {
        let m = Self::roughness_to_alpha(roughness);

        let factor = 1.0 / (m * m * n_dot_h.powf(4.0));
        let exp_factor = (n_dot_h * n_dot_h - 1.0) / (m * m * n_dot_h * n_dot_h);
        return factor * exp_factor.exp();
    }

    // Original cook torrance attenuation
    // https://digibug.ugr.es/bitstream/handle/10481/19751/rmontes_LSI-2012-001TR.pdf;jsessionid=519D0B94111D8A3C4F85A3AB22A92F6C?sequence=1
    fn geometric_attenuation(wo: Vector3, wi: Vector3, n: Vector3, _: f32) -> f32 {
        let h = (wo + wi).normalize();
        let n_dot_h = n.dot(h);

        let out_attenuation = (2.0 * n_dot_h * n.dot(wo)) / wo.dot(h);
        let in_attenuation = (2.0 * n_dot_h * n.dot(wi)) / wo.dot(h);

        return out_attenuation.min(in_attenuation).min(1.0);
    }

    fn importance_sample(wo: Vector3, roughness: f32) -> (Vector3, f32) {
        let alpha = Self::roughness_to_alpha(roughness);

        let mut logSample = (1.0f32 - rand::random::<f32>()).ln();
        if logSample.is_infinite() { logSample = 0.0; };

        let tan2Theta = -(alpha * alpha) * logSample;
        let phi = 2.0 * PI * rand::random::<f32>();
        let cosTheta = 1.0 / (1.0 + tan2Theta).sqrt();
        let sinTheta = (1.0 - cosTheta * cosTheta).max(0.0).sqrt();

        let mut cartesian_wh = Vector3::new(sinTheta * phi.cos(), cosTheta, sinTheta * phi.sin());
        // reflect into same hemisphere
        if cartesian_wh.y * wo.y < 0.0 { cartesian_wh = -cartesian_wh };

        let wi = -wo + 2.0 * wo.dot(cartesian_wh) * cartesian_wh;

        let d_pdf = cosTheta.abs() * Self::microfacet_distribution(cartesian_wh.dot(Vector3::new(0.0, 1.0, 0.0)), roughness);
        let adjust_dist_pdf = d_pdf / (4.0 * cartesian_wh.dot(wo));

        return (wi, adjust_dist_pdf)
    }
}


#[cfg(test)]
mod tests {
    use cgmath::InnerSpace;

    use crate::Vector3;
    use crate::brdf::cook_torrance::{DefaultCookTorrance, CookTorrance};
    use crate::geometry::Ray;

    #[test]
    pub fn verify_integral_of_microfacet_sums_to_1() {
        const SAMPLES: usize = 500000;

        for roughness in 0..10 {
            let roughness = (roughness as f32 / 10.0).max(0.04);
            let normal = Vector3::new(0.0, 1.0, 0.0);
            
            let sum = (0..SAMPLES).map(|_|  {
                let (incoming_dir, _) = Ray::random_direction_over_hemisphere();
                let (view_dir, _) = Ray::random_direction_over_hemisphere();
                let halfway = (view_dir + incoming_dir).normalize();
                
                DefaultCookTorrance::microfacet_distribution(normal.dot(halfway), roughness)
            }).sum::<f32>();

            assert!((1.0 - (sum / SAMPLES as f32).abs() < 0.1), "Expect sum of normal distributon for roughness {} to be close to 1, actual: {}", roughness, sum / SAMPLES as f32);
        }
    }
}
