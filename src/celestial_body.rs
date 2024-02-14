use comfy::{Color, Vec3};

use crate::celestial_object::CelestialObject;

pub struct CelestialBody {
    pub celestial_object: CelestialObject,

    pub current_force: Vec3,
    pub current_movement: Vec3,

    /// for drawing the body.
    pub color: Color,

    /// for drawing the body.
    pub radius: f32,
}

impl CelestialBody {
    pub fn update(&mut self) {
        let delta = self.current_force * (1.0 / self.celestial_object.mass);
        self.current_movement += delta;
        self.celestial_object.position += self.current_movement;
    }

    pub fn get_drawing_radius(&self) -> f32 {
        self.radius.log10() * 0.02
    }
}
