use comfy::Color;
use glam::DVec3;

use crate::celestial_body::CelestialBody;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelestialBodyForces {
    pub current_force_zero_mass: DVec3,
    pub current_movement: DVec3,
}

impl CelestialBodyForces {
    pub fn update(&mut self, body: &mut CelestialBody) {
        self.current_movement += self.current_force_zero_mass;
        body.position += self.current_movement;
    }
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
