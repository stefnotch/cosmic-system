use cosmic_system::{
    celestial_body::CelestialBody,
    simulation::{self, CreateBodiesResult, UpdateBodies},
};
use std::thread;

use comfy::*;
use cosmic_system::bounding_box::BoundingBox;
use glam::DVec3;
use simulation::create_bodies;

simple_game!("Cosmic System", GameState, setup, update);

pub struct GameState {
    pub bounding_box: BoundingBox,
    pub bodies: Arc<Mutex<Vec<CelestialBody>>>,
    pub particles: Entity,
    pub handle: Option<thread::JoinHandle<()>>,
}

impl GameState {
    pub fn new(_c: &EngineState) -> Self {
        Self {
            bounding_box: BoundingBox::new(
                DVec3::ONE * -4.0 * simulation::AU,
                DVec3::ONE * 4.0 * simulation::AU,
            ),
            bodies: Default::default(),
            particles: Entity::DANGLING,
            handle: None,
        }
    }
}

fn setup(state: &mut GameState, c: &mut EngineContext) {
    c.load_texture_from_bytes(
        "1px",
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/1px.png")),
    );

    let CreateBodiesResult {
        cosmic_system,
        bodies,
        bodies_forces,
        bodies_drawing,
    } = create_bodies(10001);

    let particles = world().reserve_entity();
    let mut particles_component = ParticleSystem::with_spawn_rate(bodies.len(), 0.0, || Particle {
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

    state.bodies = Arc::new(Mutex::new(bodies));

    let handle = {
        let bodies = Arc::clone(&state.bodies);
        let mut update_bodies = UpdateBodies {
            bounding_box: (&state.bounding_box).clone(),
            cosmic_system,
            bodies_forces,
        };

        thread::spawn(move || loop {
            let mut bodies_lock = bodies.lock();
            update_bodies.update(&mut bodies_lock);
        })
    };

    state.handle = Some(handle);
}

fn update(state: &mut GameState, _c: &mut EngineContext) {
    // Render
    {
        span_with_timing!("Update world");
        let mut world = world_mut();
        let particles = world
            .query_one_mut::<&mut ParticleSystem>(state.particles)
            .unwrap();
        let inverse_world_size = 1.0 / (1.0 * simulation::AU);
        let bodies_lock = state.bodies.lock();
        for (particle, body) in particles.particles.iter_mut().zip(bodies_lock.iter()) {
            particle.lifetime_current = 500.;
            let position = body.position * inverse_world_size;
            particle.position = Vec2::new(position.x as f32, position.y as f32);
        }
    }
}
