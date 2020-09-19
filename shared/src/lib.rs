use rapier2d::dynamics::{IntegrationParameters, JointSet, RigidBodyBuilder, RigidBodySet};
use rapier2d::geometry::{BroadPhase, ColliderBuilder, ColliderSet, NarrowPhase};
use rapier2d::na::Vector2;
use rapier2d::pipeline::{EventHandler, PhysicsPipeline};
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub type FrameIndex = u32;

#[derive(Serialize, Deserialize)]
pub enum Send {
    Ping(FrameIndex),
    InputState(IndexedState<InputState>),
}

#[derive(Serialize, Deserialize)]
pub enum Recv {
    Pong(FrameIndex),
    StateHash(IndexedState<u64>),
    FullState(IndexedState<PhysicsState>),
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct InputState {
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub mouse_down: bool,
}

#[wasm_bindgen]
impl InputState {
    #[wasm_bindgen(constructor)]
    pub fn new(mouse_x: f32, mouse_y: f32, mouse_down: bool) -> Self {
        Self {
            mouse_x,
            mouse_y,
            mouse_down,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct PhysicsState {
    pub broad_phase: BroadPhase,
    pub narrow_phase: NarrowPhase,
    pub bodies: RigidBodySet,
    pub colliders: ColliderSet,
    pub joints: JointSet,
}

#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct SharedState {
    test_field1: u64,
    test_field2: simba::scalar::FixedI16F16,
    test_field3: Vector2<simba::scalar::FixedI16F16>,
}

#[wasm_bindgen]
impl SharedState {
    #[wasm_bindgen(constructor)]
    pub fn new(test_field1: u64, test_field2: f32) -> Self {
        let v = simba::scalar::FixedI16F16::from_num(test_field2);
        Self {
            test_field1,
            test_field2: v,
            test_field3: Vector2::new(v, v),
        }
    }

    #[wasm_bindgen]
    pub fn from_json(json: JsValue) -> Result<SharedState, JsValue> {
        serde_wasm_bindgen::from_value(json).map_err(Into::into)
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct IndexedState<T> {
    pub frame_index: FrameIndex,
    pub state: T,
}

pub struct State<E> {
    pub pipeline: PhysicsPipeline,
    pub gravity: Vector2<f32>,
    pub integration_parameters: IntegrationParameters,
    pub physics_state: PhysicsState,
    pub event_handler: E,

    pub frame_index: FrameIndex,
}

impl<E> State<E> {
    pub fn new_with_state(physics_state: PhysicsState, event_handler: E) -> Self {
        let pipeline = PhysicsPipeline::new();
        let gravity = Vector2::new(0f32, -9.81f32);
        let integration_parameters = IntegrationParameters::default();
        Self {
            pipeline,
            gravity,
            integration_parameters,
            physics_state,
            event_handler,
            frame_index: 0,
        }
    }

    pub fn new(event_handler: E) -> Self {
        let broad_phase = BroadPhase::new();
        let narrow_phase = NarrowPhase::new();
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let joints = JointSet::new();

        let ground_body = RigidBodyBuilder::new_static().translation(0., -2.).build();
        let ground_body_handle = bodies.insert(ground_body);
        let ground_collider = ColliderBuilder::cuboid(2., 0.5).build();
        colliders.insert(ground_collider, ground_body_handle, &mut bodies);

        let ball_body = RigidBodyBuilder::new_dynamic().translation(0., 0.).build();
        let ball_body_handle = bodies.insert(ball_body);
        let ball_collider = ColliderBuilder::ball(0.5).build();
        colliders.insert(ball_collider, ball_body_handle, &mut bodies);

        let physics_state = PhysicsState {
            broad_phase,
            narrow_phase,
            bodies,
            colliders,
            joints,
        };

        Self::new_with_state(physics_state, event_handler)
    }

    fn add_ball(&mut self, x: f32, y: f32) {
        let PhysicsState {
            bodies, colliders, ..
        } = &mut self.physics_state;
        let ball_body = RigidBodyBuilder::new_dynamic().translation(x, y).build();
        let ball_body_handle = bodies.insert(ball_body);
        let ball_collider = ColliderBuilder::ball(0.5).build();
        colliders.insert(ball_collider, ball_body_handle, bodies);
    }

    pub fn step(&mut self, input: InputState)
    where
        E: EventHandler,
    {
        if input.mouse_down {
            self.add_ball(input.mouse_x, input.mouse_y)
        }

        self.pipeline.step(
            &self.gravity,
            &self.integration_parameters,
            &mut self.physics_state.broad_phase,
            &mut self.physics_state.narrow_phase,
            &mut self.physics_state.bodies,
            &mut self.physics_state.colliders,
            &mut self.physics_state.joints,
            &self.event_handler,
        );
        self.frame_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut state = State::new(());

        for _ in 0..100 {
            println!("FRAME");
            for (_, body) in state.physics_state.bodies.iter_active_dynamic() {
                println!("{:?}", body.position);
            }
            state.step(InputState::default());
        }
    }

    #[test]
    fn serde() {
        let mut state = State::new(());
        state.step(InputState::default());
        println!("{:?}", serde_json::to_string_pretty(&state.physics_state));
    }
}
