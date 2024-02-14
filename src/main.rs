pub mod bounding_box;
pub mod celestial_body;
pub mod celestial_object;
pub mod cosmic_system;
pub mod simulation;
pub mod vec3_extensions;

use bounding_box::BoundingBox;
use celestial_body::{CelestialBody, CelestialBodyDrawing};
use celestial_object::CelestialObject;
use comfy::{num_traits::Float, *};
use cosmic_system::CosmicSystem;
use glam::DVec3;

simple_game!("Cosmic System", GameState, setup, update);

pub struct GameState {
    pub bounding_box: BoundingBox,
    pub bodies: Vec<CelestialBody>,
    pub particles: Entity,
}
/*
// Only if the tracing feature is enabled
#[cfg(feature = "tracing")] */

impl GameState {
    pub fn new(_c: &EngineState) -> Self {
        Self {
            bounding_box: BoundingBox::new(
                DVec3::ONE * -4.0 * simulation::AU,
                DVec3::ONE * 4.0 * simulation::AU,
            ),
            bodies: vec![],
            particles: Entity::DANGLING,
        }
    }
}
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

fn setup(state: &mut GameState, c: &mut EngineContext) {
    c.load_texture_from_bytes(
        "1px",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/1px.png")),
    );

    let body_count = 10001;
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
                ) * simulation::AU,
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
    state.bodies = bodies;

    let particles = world().reserve_entity();
    let mut particles_component = ParticleSystem::with_spawn_rate(body_count, 0.0, || Particle {
        texture: texture_id("1px"),
        position: random_circle(5.0),
        //position: Vec2::ZERO,
        direction: Vec2::ZERO,

        velocity: 0.0,
        velocity_end: 0.0,
        lifetime_max: 1000.0, // Infinity breaks the engine
        size: splat(1.0),

        velocity_curve: linear,
        size_curve: linear,
        color_curve: linear,
        fade_in_duration: FadeInDuration::None,
        fade_type: FadeType::None,
        ..Default::default()
    });
    particles_component.spawn_rate = None;

    for (particle, body) in particles_component
        .particles
        .iter_mut()
        .zip(bodies_drawing.into_iter())
    {
        particle.size = Vec2::splat(body.get_drawing_radius());
        particle.color_start = body.color;
        particle.color_end = body.color;
    }

    world_mut()
        .insert(
            particles,
            (particles_component, Transform::position(Vec2::ZERO)),
        )
        .unwrap();
    state.particles = particles;
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    // create tree
    let cosmic_system = {
        let mut cosmic_system = CosmicSystem::new(state.bounding_box, state.bodies.len());
        for body in state.bodies.iter() {
            cosmic_system.add(body.celestial_object.clone());
        }
        cosmic_system
    };

    // for each body: compute the total force exerted on it.
    // this is the bottleneck, but we only read things from the tree
    // so we can easily multithread it
    {
        for body in state.bodies.iter_mut() {
            let force = cosmic_system.gravitational_force(&body.celestial_object);
            body.current_force = force;
        }
    }

    // move bodies with the force
    // has to be done separately, because you can't move bodies while still computing gravity
    {
        for body in state.bodies.iter_mut() {
            body.update();
        }
    }

    // Render
    {
        let mut world = world_mut();
        let particles = world
            .query_one_mut::<&mut ParticleSystem>(state.particles)
            .unwrap();
        let inverse_world_size = 1.0 / (8.0 * simulation::AU);
        for (particle, body) in particles.particles.iter_mut().zip(state.bodies.iter()) {
            particle.lifetime_current = particle.lifetime_max;
            let position = body.celestial_object.position * inverse_world_size;
            particle.position = Vec2::new(position.x as f32, position.y as f32);
        }
    }
}
