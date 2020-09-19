use simba::scalar::FixedI32F32 as Float;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

type Point2D = nalgebra::Point2<Float>;
type Vector2D = nalgebra::Vector2<Float>;

mod consts {
    fixed::const_fixed_from_int! {
        const DENSITY_TEMP: fixed::types::I32F32 = 1;
        const GRAVITY_TEMP: fixed::types::I32F32 = 1;
    }

    const TICK_TEMP: fixed::types::I32F32 = fixed::types::I32F32::from_le_bytes([68, 4, 0, 0, 0, 0, 0, 0]);
    pub const DENSITY: super::Float =
        simba::scalar::FixedI64::<fixed::types::extra::U32>(DENSITY_TEMP);
    pub const GRAVITY: super::Float =
        simba::scalar::FixedI64::<fixed::types::extra::U32>(GRAVITY_TEMP);
    pub const TICK: super::Float = simba::scalar::FixedI64::<fixed::types::extra::U32>(TICK_TEMP);
}
use consts::*;
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
            position_x: self.position.x.0.to_num(),
            position_y: self.position.y.0.to_num(),
            radius: self.radius().0.to_num(),
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
    (dx * dx) + (dy * dy)
}

impl Body {
    pub fn radius(&self) -> Float {
        // FIXME: what is the actual relationship between mass and area (AKA radius)?
        self.mass * DENSITY
    }

    pub fn collides_with(&self, other: &Body) -> bool {
        let d = distance_squared(&self.position, &other.position);
        let r = self.radius() + other.radius();
        d <= (r * r)
    }

    pub fn force_from(&self, other: &Body) -> Float {
        let softening_constant = Float::from_num(0.15);
        let distance = distance_squared(&self.position, &other.position);
        let d2 = nalgebra::ComplexField::try_sqrt(distance + softening_constant).unwrap();
        (GRAVITY * other.mass) /  simba::scalar::FixedI64::<fixed::types::extra::U32>(distance.0.saturating_mul(d2.0))
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
                let mut acc = Vector2D::zeros();
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
            body.velocity += &body.acceleration;
            body.position += &body.velocity;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn collision() {
        let b1 = Body::new(0., 0., 1.);
        let b2 = Body::new(2., 0., 0.5);
        let b3 = Body::new(1., 0., 0.75);

        assert!(!b1.collides_with(&b2));
        assert!(b1.collides_with(&b3));

        println!(
            "{}, {}",
            fixed::types::I32F32::MAX,
            fixed::types::I32F32::MIN
        );
    }

    #[test]
    fn sim_acceleration() {
        let b1 = Body::new(0., 0., 1.);
        let b2 = Body::new(2., 0., 1.);
        assert!(b1.force_from(&b2) > Float::from_num(0.));

        let mut sim = Simulation::default();
        sim.bodies.push(b1);
        sim.bodies.push(b2);

        let d1 = nalgebra::distance(&sim.bodies[0].position, &sim.bodies[1].position);
        sim.step();
        let d2 = nalgebra::distance(&sim.bodies[0].position, &sim.bodies[1].position);
        assert!(d1 > d2);
    }

    #[test]
    fn sim() {
        let mut sim = Simulation::new();
        let mut rng = rand::thread_rng();
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
