use comfy::Color;
use glam::DVec3;

use crate::celestial_object::CelestialObject;

pub struct CelestialBody {
    pub celestial_object: CelestialObject,

    pub current_force: DVec3,
    pub current_movement: DVec3,
}

pub struct CelestialBodyDrawing {
    /// for drawing the body.
    pub color: Color,

    /// for drawing the body.
    pub radius: f64,
}

impl CelestialBodyDrawing {
    pub fn get_drawing_radius(&self) -> f32 {
        (self.radius.log10() * 0.02) as f32
    }
}

impl CelestialBody {
    pub fn update(&mut self) {
        let delta = self.current_force * (1.0 / self.celestial_object.mass);
        self.current_movement += delta;
        self.celestial_object.position += self.current_movement;
    }
}
