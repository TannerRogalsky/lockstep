// use simba::scalar::FixedI32F32 as Float;
use fixed::types::I32F32 as Float;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

type Point2D = nalgebra::Point2<Float>;
type Vector2D = nalgebra::Vector2<Float>;

fixed::const_fixed_from_int! {
    const DENSITY: Float = 1;
    const GRAVITY: Float = 1;
    const TICK: Float = 1;
}
// const TICK: Float = Float::from_le_bytes([68, 4, 0, 0, 0, 0, 0, 0]);
static ID_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

#[wasm_bindgen(inspectable)]
pub struct BodyRenderData {
    pub position_x: f32,
    pub position_y: f32,
    pub radius: f32,
}

impl Into<BodyRenderData> for &Body {
    fn into(self) -> BodyRenderData {
        BodyRenderData {
            position_x: self.position.x.to_num(),
            position_y: self.position.y.to_num(),
            radius: self.radius().to_num(),
        }
    }
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Body {
    id: usize,
    position: Point2D,
    velocity: Vector2D,
    acceleration: Vector2D,
    mass: Float,
}

#[wasm_bindgen]
impl Body {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f32, y: f32, mass: f32) -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            position: Point2D::new(Float::from_num(x), Float::from_num(y)),
            velocity: Vector2D::new(Float::from_num(0.), Float::from_num(0.)),
            acceleration: Vector2D::new(Float::from_num(0.), Float::from_num(0.)),
            mass: Float::from_num(mass),
        }
    }
}

fn distance_squared(p1: &Point2D, p2: &Point2D) -> Float {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
}

fn lerp(v0: Float, v1: Float, t: Float) -> Float {
    (Float::from_bits(1 << 32) - t) * v0 + t * v1
}

fn lerp_point(v0: Point2D, v1: Point2D, t: Float) -> Point2D {
    Point2D::new(lerp(v0.x, v1.x, t), lerp(v0.y, v1.y, t))
}

// TODO: Need a cbrt implementation for fixed point floats
fn cbrt(v: Float) -> Float {
    let v: f32 = v.to_num();
    Float::from_num(v.cbrt())
}

impl Body {
    pub fn volume(&self) -> Float {
        self.mass / DENSITY
    }

    pub fn radius(&self) -> Float {
        let radius =
            self.volume() * Float::from_num(3) / Float::from_num(4. * std::f64::consts::PI);
        cbrt(radius)
    }

    pub fn collides_with(&self, other: &Body) -> bool {
        let d = distance_squared(&self.position, &other.position);
        let r = self.radius() + other.radius();
        d <= (r * r)
    }

    pub fn force_from(&self, other: &Body) -> Float {
        let softening_constant = Float::from_num(0.15);
        let distance = distance_squared(&self.position, &other.position);
        let d2 = distance.saturating_add(softening_constant);
        (GRAVITY * other.mass) / distance.saturating_mul(d2)
    }
}

#[wasm_bindgen]
#[derive(Clone, Debug, Eq, PartialEq, Hash, Default)]
pub struct Simulation {
    bodies: Vec<Body>,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    #[wasm_bindgen]
    pub fn body_count(&self) -> usize {
        self.bodies.len()
    }

    #[wasm_bindgen]
    pub fn render_data(&self, index: usize) -> Option<BodyRenderData> {
        self.bodies.get(index).map(Into::into)
    }

    #[wasm_bindgen]
    pub fn add_body(&mut self, body: Body) {
        self.bodies.push(body)
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        // update accelerations
        for i in 0..self.bodies.len() {
            self.bodies[i].acceleration = {
                let body = &self.bodies[i];
                let mut acc = Vector2D::new(Float::from_bits(0), Float::from_bits(0));
                for other in self.bodies.iter() {
                    if body.id != other.id {
                        let d = other.position.coords - body.position.coords;
                        let force = body.force_from(other);
                        acc += d * force;
                    }
                }
                acc
            };
        }

        // update velocities & positions
        for body in self.bodies.iter_mut() {
            body.velocity += &body.acceleration * TICK;
            body.position += &body.velocity * TICK;
        }

        let mut collisions = std::collections::HashSet::new();
        for body1 in self.bodies.iter() {
            for body2 in self.bodies.iter() {
                if !std::ptr::eq(body1, body2) && body1.collides_with(body2) {
                    if !collisions.contains(&(body2.id, body1.id)) {
                        collisions.insert((body1.id, body2.id));
                    }
                }
            }
        }

        for (id1, id2) in collisions {
            if let Some(body2_index) = self.bodies.iter().position(|body| body.id == id2) {
                let body2 = self.bodies.swap_remove(body2_index);
                if let Some(body1) = self.bodies.iter_mut().find(|body| body.id == id1) {
                    body1.mass += body2.mass;
                    body1.position =
                        lerp_point(body1.position, body2.position, Float::from_num(0.5));
                    body1.velocity = Vector2D::new(Float::from_bits(0), Float::from_bits(0));
                } else {
                    log::error!("Found only one side of a collision.")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_test() {
        {
            let v0 = Float::from_num(0.);
            let v1 = Float::from_num(1.);
            let t = Float::from_num(0.5);
            assert_eq!(lerp(v0, v1, t), t);
        }

        {
            let v0 = Float::from_num(0.5);
            let v1 = Float::from_num(1.);
            let t = Float::from_num(0.5);
            let half = Float::from_num(0.75);
            assert_eq!(lerp(v0, v1, t), half);
        }
    }

    #[test]
    fn lerp_point_test() {
        let p1 = Point2D::new(Float::from(0), Float::from(0));
        let p2 = Point2D::new(Float::from(1), Float::from(1));
        let half = Float::from_num(0.5);
        assert_eq!(lerp_point(p1, p2, half), Point2D::new(half, half))
    }

    #[test]
    fn mass_volume_density_radius() {
        let b1 = Body::new(0., 0., 1.);
        assert_eq!(b1.volume(), Float::from_num(1));
    }

    #[test]
    fn collision() {
        let b1 = Body::new(0., 0., 1.);
        let b2 = Body::new(2., 0., 0.5);
        let b3 = Body::new(1., 0., 4.);

        assert!(!b1.collides_with(&b2));
        assert!(b1.collides_with(&b3));
        assert!(b2.collides_with(&b3));
    }

    #[test]
    fn sim_acceleration() {
        let b1 = Body::new(0., 0., 1.);
        let b2 = Body::new(3., 0., 1.);
        assert!(b1.force_from(&b2) > Float::from_num(0.));

        let mut sim = Simulation::default();
        sim.bodies.push(b1);
        sim.bodies.push(b2);

        let d1 = distance_squared(&sim.bodies[0].position, &sim.bodies[1].position);
        sim.step();
        let d2 = distance_squared(&sim.bodies[0].position, &sim.bodies[1].position);
        assert!(d1 > d2, "{} isn't larger than {}", d1, d2);
    }

    #[test]
    fn sim_collision() {
        let b1 = Body::new(0., 0., 1.);
        let b2 = Body::new(1., 0., 1.);
        assert!(b1.collides_with(&b2));

        let mut sim = Simulation::default();
        sim.bodies.push(b1);
        sim.bodies.push(b2);

        assert_eq!(sim.bodies.len(), 2);
        sim.step();
        assert_eq!(sim.bodies.len(), 1);
    }

    #[test]
    fn sim() {
        use rand::prelude::*;

        let mut sim = Simulation::new();
        let mut rng = thread_rng();
        for _ in 0..100 {
            sim.add_body(Body::new(
                rng.gen_range(0., 480.),
                rng.gen_range(0., 480.),
                rng.gen_range(0., 0.2),
            ));
        }
        sim.step();
    }
}
