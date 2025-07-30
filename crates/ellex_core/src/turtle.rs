use serde::{Deserialize, Serialize};

// Turtle graphics state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurtleGraphics {
    x: f64,         // Current x position
    y: f64,         // Current y position
    angle: f64,     // Direction in degrees
    pen_down: bool, // Whether pen is drawing
    color: String,  // Pen color (e.g., "red")
    lines: Vec<Line>, // Drawing history
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub from: (f64, f64),
    pub to: (f64, f64),
    pub color: String,
}

impl TurtleGraphics {
    pub fn new() -> Self {
        TurtleGraphics {
            x: 0.0,
            y: 0.0,
            angle: 0.0,
            pen_down: true,
            color: "black".to_string(),
            lines: Vec::new(),
        }
    }

    pub fn move_forward(&mut self, distance: f64) {
        let start_pos = (self.x, self.y);
        let radians = self.angle.to_radians();
        self.x += distance * radians.cos();
        self.y += distance * radians.sin();
        
        // Record line if pen is down
        if self.pen_down {
            self.lines.push(Line {
                from: start_pos,
                to: (self.x, self.y),
                color: self.color.clone(),
            });
        }
    }

    pub fn forward(&mut self, distance: f64) {
        self.move_forward(distance);
    }

    pub fn turn(&mut self, degrees: f64) {
        self.angle = (self.angle + degrees) % 360.0;
    }

    pub fn turn_right(&mut self, degrees: f64) {
        self.turn(degrees);
    }

    pub fn turn_left(&mut self, degrees: f64) {
        self.turn(-degrees);
    }

    pub fn set_pen(&mut self, down: bool) {
        self.pen_down = down;
    }

    pub fn pen_up(&mut self) {
        self.set_pen(false);
    }

    pub fn pen_down(&mut self) {
        self.set_pen(true);
    }

    pub fn set_color(&mut self, color: &str) {
        self.color = color.to_string();
    }

    pub fn position(&self) -> (f64, f64) {
        (self.x, self.y)
    }

    pub fn angle(&self) -> f64 {
        self.angle
    }

    pub fn is_pen_down(&self) -> bool {
        self.pen_down
    }

    pub fn get_color(&self) -> &str {
        &self.color
    }

    pub fn get_lines(&self) -> &[Line] {
        &self.lines
    }

    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.angle = 0.0;
        self.pen_down = true;
        self.color = "black".to_string();
        self.lines.clear();
    }

    pub fn clear_drawing(&mut self) {
        self.lines.clear();
    }

    pub fn home(&mut self) {
        let start_pos = (self.x, self.y);
        self.x = 0.0;
        self.y = 0.0;
        
        // Draw line to home if pen is down
        if self.pen_down {
            self.lines.push(Line {
                from: start_pos,
                to: (0.0, 0.0),
                color: self.color.clone(),
            });
        }
    }

    pub fn goto(&mut self, x: f64, y: f64) {
        let start_pos = (self.x, self.y);
        self.x = x;
        self.y = y;
        
        // Draw line to new position if pen is down
        if self.pen_down {
            self.lines.push(Line {
                from: start_pos,
                to: (x, y),
                color: self.color.clone(),
            });
        }
    }
}
