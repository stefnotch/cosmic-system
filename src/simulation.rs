use crate::{
    bounding_box::BoundingBox,
    celestial_body::{CelestialBody, CelestialBodyDrawing},
    celestial_object::CelestialObject,
    cosmic_system::CosmicSystem,
};
use comfy::{num_traits::Float, *};
use glam::DVec3;

pub const G: f64 = 6.6743e-11;
pub const AU: f64 = 150e9;

/// Box-Muller transform
/// https://en.wikipedia.org/wiki/Box%E2%80%93Muller_transform
fn random_gaussian(mu: f64, sigma: f64) -> f64 {
    let mut u1 = gen_range(0.0, 1.0);
    while u1 == 0.0 {
        u1 = gen_range(0.0, 1.0);
    }
    let u2 = gen_range(0.0, 1.0);

    let mag = sigma * (-2.0 * u1.ln()).sqrt();
    mag * (u2 * std::f64::consts::PI * 2.0).cos() + mu
}

pub fn create_bodies(body_count: usize) -> (Vec<CelestialBody>, Vec<CelestialBodyDrawing>) {
    srand(125245337);
    let predefined_colors = vec![RED, BLUE, CYAN, MAGENTA, PINK, GREEN, DARK_GRAY];
    let mut bodies = Vec::with_capacity(body_count);
    let mut bodies_drawing = Vec::with_capacity(body_count);
    for i in 0..body_count {
        bodies.push(CelestialBody {
            celestial_object: CelestialObject::new(
                gen_range(5e20, 5e20 + 5e20),
                DVec3::new(
                    (random_gaussian(0., 1.) * 8. - 4.) * 0.01 + if i % 2 == 0 { 2. } else { -2. },
                    (random_gaussian(0., 1.) * 8. - 4.) * 0.01,
                    (random_gaussian(0., 1.) * 8. - 4.) * 0.01,
                ) * crate::simulation::AU,
            ),
            current_movement: DVec3::new(
                random_gaussian(0., 1.),
                random_gaussian(0., 1.),
                random_gaussian(0., 1.),
            ) * 1e9,
            current_force: DVec3::ZERO,
        });
        bodies_drawing.push(CelestialBodyDrawing {
            radius: gen_range(10000., 800000.),
            color: predefined_colors[random_usize(0, predefined_colors.len())],
        });
    }
    bodies[0] = CelestialBody {
        celestial_object: CelestialObject::new(1e40, DVec3::ZERO),
        current_movement: DVec3::ZERO,
        current_force: DVec3::ZERO,
    };
    bodies_drawing[0] = CelestialBodyDrawing {
        radius: 7000000000.,
        color: WHITE,
    };

    (bodies, bodies_drawing)
}

pub fn update_bodies(bounding_box: BoundingBox, bodies: &mut Vec<CelestialBody>) {
    let cosmic_system = {
        let _span = span!("Create tree");
        let mut cosmic_system = CosmicSystem::new(bounding_box, bodies.len());
        for body in bodies.iter() {
            cosmic_system.add(body.celestial_object.clone());
        }
        cosmic_system
    };

    // for each body: compute the total force exerted on it.
    // this is the bottleneck, but we only read things from the tree
    // so we can easily multithread it
    {
        let _span = span!("Compute forces");
        bodies.par_iter_mut().for_each(|body| {
            let force = cosmic_system.gravitational_force(&body.celestial_object);
            body.current_force = force;
        });
    }

    // move bodies with the force
    // has to be done separately, because you can't move bodies while still computing gravity
    {
        let _span = span!("Update bodies");
        for body in bodies.iter_mut() {
            body.update();
        }
    }
}
