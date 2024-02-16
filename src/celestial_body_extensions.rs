use comfy::Color;

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
