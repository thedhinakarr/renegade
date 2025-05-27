use crate::constants::*;

#[derive(Clone)]
pub struct Player {
    pub angle: f64,
    pub speed: f64,
}

impl Player {
    pub fn new() -> Self {
        Player {
            angle: 0.0,
            speed: 0.02,
        }
    }
    
    pub fn get_position(&self) -> (f64, f64) {
        let x = CANVAS_WIDTH / 2.0 + self.angle.cos() * ORBIT_RADIUS;
        let y = CANVAS_HEIGHT / 2.0 + self.angle.sin() * ORBIT_RADIUS;
        (x, y)
    }
}

pub struct Threat {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub radius: f64,
}

pub struct Projectile {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub radius: f64,
}

pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub size: f64,
    pub lifetime: f64,
    pub max_lifetime: f64,
    pub color: (u8, u8, u8),
}