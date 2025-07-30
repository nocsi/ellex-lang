use serde::{Deserialize, Serialize};

// Turtle graphics state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurtleGraphics {
    x: f64,         // Current x position
    y: f64,         // Current y position
    angle: f64,     // Direction in degrees
    pen_down: bool, // Whether pen is drawing
    color: String,  // Pen color (e.g., "red")
}

impl TurtleGraphics {
    pub fn new() -> Self {
        TurtleGraphics {
            x: 0.0,
            y: 0.0,
            angle: 0.0,
            pen_down: true,
            color: "black".to_string(),
        }
    }

    pub fn forward(&mut self, distance: f64) {
        let radians = self.angle.to_radians();
        self.x += distance * radians.cos();
        self.y += distance * radians.sin();
        // In a real app, draw a line if pen_down (to be hooked into a canvas)
    }

    pub fn turn(&mut self, degrees: f64) {
        self.angle = (self.angle + degrees) % 360.0;
    }

    pub fn set_pen(&mut self, down: bool) {
        self.pen_down = down;
    }

    pub fn set_color(&mut self, color: &str) {
        self.color = color.to_string();
    }

    pub fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }
}
