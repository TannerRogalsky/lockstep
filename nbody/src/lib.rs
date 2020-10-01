pub use fixed::types::I32F32 as Float;
use serde::{Deserialize, Serialize};

pub type Point2D = nalgebra::Point2<Float>;
pub type Vector2D = nalgebra::Vector2<Float>;

fixed::const_fixed_from_int! {
    const DENSITY: Float = 1;
    const TICK: Float = 1;
}
static ID_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn zero_vec() -> Vector2D {
    Vector2D::new(Float::from_bits(0), Float::from_bits(0))
}

#[derive(Copy, Clone, Debug, Eq, Serialize, Deserialize)]
pub struct Body {
    id: usize,
    pub position: Point2D,
    pub velocity: Vector2D,
    pub acceleration: Vector2D,
    pub mass: Float,
}

impl Body {
    pub fn new(x: Float, y: Float, mass: Float) -> Self {
        Self {
            id: ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst),
            position: Point2D::new(x, y),
            velocity: Vector2D::new(Float::from_bits(0), Float::from_bits(0)),
            acceleration: Vector2D::new(Float::from_bits(0), Float::from_bits(0)),
            mass,
        }
    }

    pub fn new_lossy(x: f32, y: f32, mass: f32) -> Self {
        Self::new(
            Float::from_num(x),
            Float::from_num(y),
            Float::from_num(mass),
        )
    }
}

impl std::cmp::PartialEq for Body {
    fn eq(&self, other: &Self) -> bool {
        self.id.eq(&other.id)
    }
}

impl std::hash::Hash for Body {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.x.hash(state);
        self.position.y.hash(state);
        self.velocity.x.hash(state);
        self.velocity.y.hash(state);
        self.acceleration.x.hash(state);
        self.acceleration.y.hash(state);
        self.mass.hash(state);
    }
}

fn distance_squared(p1: &Point2D, p2: &Point2D) -> Float {
    let dx = p2.x - p1.x;
    let dy = p2.y - p1.y;
    dx.saturating_mul(dx).saturating_add(dy.saturating_mul(dy))
}

fn magnitude(v: Vector2D) -> Float {
    let x = v.x.saturating_mul(v.x);
    let y = v.y.saturating_mul(v.y);
    let acc = x.saturating_add(y);
    fixed_sqrt::FixedSqrt::sqrt(acc)
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

    pub fn force_from(&self, other: &Body) -> Vector2D {
        let diff: Vector2D = other.position.coords - self.position.coords;
        let r = magnitude(diff);
        if r == Float::from_bits(0) {
            zero_vec()
        } else {
            let gravity = Float::from_num(0.1);
            diff * gravity * other.mass / r.saturating_mul(r).saturating_mul(r)
        }
    }
}

#[derive(Clone, Debug, Eq, Default, Serialize, Deserialize)]
pub struct Simulation {
    pub bodies: Vec<Body>,
}

impl std::cmp::PartialEq for Simulation {
    fn eq(&self, other: &Self) -> bool {
        self.bodies.eq(&other.bodies)
    }
}

impl std::hash::Hash for Simulation {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for body in self.bodies.iter() {
            body.hash(state);
        }
    }
}

impl Simulation {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn add_body(&mut self, body: Body) {
        self.bodies.push(body)
    }

    pub fn step(&mut self) {
        let mut collisions = std::collections::HashSet::new();
        for body1 in self.bodies.iter() {
            for body2 in self.bodies.iter() {
                if !std::ptr::eq(body1, body2)
                    && body1.collides_with(body2)
                    && !collisions.contains(&(body2.id, body1.id))
                {
                    collisions.insert((body1.id, body2.id));
                }
            }
        }

        for (id1, id2) in collisions {
            if let Some(body2_index) = self.bodies.iter().position(|body| body.id == id2) {
                let body2 = self.bodies.swap_remove(body2_index);
                if let Some(body1) = self.bodies.iter_mut().find(|body| body.id == id1) {
                    body1.position = ((body1.position * body1.mass)
                        + (body2.position * body2.mass).coords)
                        / (body1.mass + body2.mass);
                    body1.velocity = ((body1.velocity * body1.mass)
                        + (body2.velocity * body2.mass))
                        / (body1.mass + body2.mass);
                    body1.mass += body2.mass;
                } else {
                    log::error!("Found only one side of a collision.")
                }
            }
        }

        // update accelerations
        for i in 0..self.bodies.len() {
            self.bodies[i].acceleration = {
                let body = &self.bodies[i];
                let mut acc = Vector2D::new(Float::from_bits(0), Float::from_bits(0));
                for other in self.bodies.iter() {
                    if body.id != other.id {
                        acc += body.force_from(other);
                    }
                }
                acc
            };
        }

        // update velocities & positions
        for body in self.bodies.iter_mut() {
            body.velocity += body.acceleration * TICK;
            body.position += body.velocity * TICK;
        }
    }

    pub fn center_of_mass(&self) -> Point2D {
        let (pos, mass) = self.bodies.iter().fold(
            (
                Point2D::new(Float::from_bits(0), Float::from_bits(0)),
                Float::from_bits(0),
            ),
            |(acc_pos, acc_mass), body| {
                (
                    acc_pos + (body.position * body.mass).coords,
                    acc_mass + body.mass,
                )
            },
        );
        pos / mass
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn force_from_test() {
        let b1 = Body::new_lossy(0., 0., 1.);
        let b2 = Body::new_lossy(0., 0., 1.);

        assert_eq!(b1.force_from(&b2), zero_vec())
    }

    #[test]
    fn mass_volume_density_radius() {
        let b1 = Body::new_lossy(0., 0., 1.);
        assert_eq!(b1.volume(), Float::from_num(1));
    }

    #[test]
    fn collision() {
        let b1 = Body::new_lossy(0., 0., 1.);
        let b2 = Body::new_lossy(2., 0., 0.5);
        let b3 = Body::new_lossy(1., 0., 4.);

        assert!(!b1.collides_with(&b2));
        assert!(b1.collides_with(&b3));
        assert!(b2.collides_with(&b3));
    }

    #[test]
    fn sim_acceleration() {
        let b1 = Body::new_lossy(0., 0., 1.);
        let b2 = Body::new_lossy(3., 0., 1.);
        assert!(b1.force_from(&b2).x > Float::from_bits(0));

        let mut sim = Simulation::default();
        sim.bodies.push(b1);
        sim.bodies.push(b2);

        let d1 = distance_squared(&sim.bodies[0].position, &sim.bodies[1].position);
        sim.step();
        let d2 = distance_squared(&sim.bodies[0].position, &sim.bodies[1].position);
        assert!(d1 > d2, "{} isn't larger than {}", d1, d2);
    }

    #[test]
    fn sim_collision1() {
        let b1 = Body::new_lossy(0., 0., 1.);
        let b2 = Body::new_lossy(1., 0., 1.);
        assert!(b1.collides_with(&b2));

        let mut sim = Simulation::default();
        sim.bodies.push(b1);
        sim.bodies.push(b2);

        assert_eq!(sim.bodies.len(), 2);
        sim.step();
        assert_eq!(sim.bodies.len(), 1);
    }

    #[test]
    fn sim_collision2() {
        let b1 = Body::new_lossy(0., 0., 10.);
        let b2 = Body::new_lossy(1., 0., 10.);
        assert!(b1.collides_with(&b2));

        let mut sim = Simulation::default();
        sim.bodies.push(b1);
        sim.bodies.push(b2);

        assert_eq!(sim.bodies.len(), 2);
        sim.step();
        assert_eq!(sim.bodies.len(), 1);

        let b3 = sim.bodies.get(0).unwrap();
        assert!(b1.position.x < b3.position.x);
        assert!(b2.position.x > b3.position.x);
    }

    #[test]
    fn sim_rand() {
        use rand::prelude::*;

        let mut sim = Simulation::new();
        let mut rng = thread_rng();
        for _ in 0..100 {
            sim.add_body(Body::new_lossy(
                rng.gen_range(0., 480.),
                rng.gen_range(0., 480.),
                rng.gen_range(0., 0.2),
            ));
        }
        sim.step();
    }

    #[test]
    fn distance_attenuation() {
        let b1 = Body::new_lossy(0., 0., 1.);
        let b2 = Body::new_lossy(10., 0., 1.);
        let b3 = Body::new_lossy(100., 0., 1.);

        let f1 = b1.force_from(&b2);
        let f2 = b1.force_from(&b3);

        assert!(f1.x > f2.x, "{} <= {}", f1.x, f2.x);
    }

    #[test]
    fn data_test0() {
        let b1 = Body::new_lossy(305.72119140625, 141.26641845703125, 10021134.);
        let b2 = Body::new_lossy(529., 825., 10000.);

        assert!(
            b1.force_from(&b2) > zero_vec(),
            "{:?}",
            b1.force_from(&b2).data
        );
    }

    #[test]
    fn collision_integrations() {
        let b1 = Body::new_lossy(0., 0., 1.);
        let b2 = Body::new_lossy(10., 0., 1.);

        let mut sim = Simulation::new();
        sim.add_body(b1);
        sim.add_body(b2);

        while sim.bodies.len() > 1 {
            sim.step();
        }

        let b3 = sim.bodies.get(0).unwrap();
        assert!(b1.position.x < b3.position.x);
        assert!(b2.position.x > b3.position.x);
    }

    #[test]
    fn center_of_mass() {
        let b1 = Body::new_lossy(0., 0., 1.);
        let b2 = Body::new_lossy(10., 0., 1.);

        let mut sim = Simulation::new();
        sim.add_body(b1);
        sim.add_body(b2);

        assert_eq!(
            sim.center_of_mass(),
            Point2D::new(Float::from_num(5), Float::from_num(0))
        );
    }
}
