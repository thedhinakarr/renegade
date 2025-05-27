use crate::game::entities::*;
use crate::constants::*;

pub fn check_collision(x1: f64, y1: f64, r1: f64, x2: f64, y2: f64, r2: f64) -> bool {
    let dx = x1 - x2;
    let dy = y1 - y2;
    let distance = (dx * dx + dy * dy).sqrt();
    distance < r1 + r2
}

pub fn is_threat_hitting_planet(threat: &Threat) -> bool {
    let dist_to_center = ((threat.x - CANVAS_WIDTH/2.0).powi(2) + 
                         (threat.y - CANVAS_HEIGHT/2.0).powi(2)).sqrt();
    dist_to_center < PLANET_RADIUS
}

pub fn is_on_screen(x: f64, y: f64, margin: f64) -> bool {
    x > -margin && x < CANVAS_WIDTH + margin && 
    y > -margin && y < CANVAS_HEIGHT + margin
}