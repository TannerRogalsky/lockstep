pub extern crate nbody;

mod input_buffer;
pub use input_buffer::*;

use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub type FrameIndex = u32;

#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum Send {
    Ping(FrameIndex),
    InputState(IndexedState<MouseDownEvent>),
}

#[derive(Serialize, Deserialize)]
pub enum Recv {
    Pong(FrameIndex),
    StateHash(IndexedState<u64>),
    InputState(IndexedState<MouseDownEvent>),
    FullState(State),
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Default, Hash, Serialize, Deserialize, Eq, PartialEq)]
pub struct MouseDownEvent {
    mouse_x: nbody::Float,
    mouse_y: nbody::Float,
    mass: nbody::Float,
}

#[wasm_bindgen]
impl MouseDownEvent {
    #[wasm_bindgen(constructor)]
    pub fn new(mouse_x: f32, mouse_y: f32, mass: f32) -> Self {
        Self {
            mouse_x: nbody::Float::from_num(mouse_x),
            mouse_y: nbody::Float::from_num(mouse_y),
            mass: nbody::Float::from_num(mass),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct IndexedState<T> {
    pub frame_index: FrameIndex,
    pub state: T,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct State {
    pub simulation: nbody::Simulation,
    pub frame_index: FrameIndex,
    pub input_buffer: InputBuffer,
}

impl std::hash::Hash for State {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.simulation.hash(state);
        self.frame_index.hash(state);
    }
}

impl State {
    pub fn new() -> Self {
        Self {
            simulation: Default::default(),
            frame_index: 0,
            input_buffer: Default::default(),
        }
    }

    pub fn hash(&self) -> u64 {
        let mut hasher = twox_hash::XxHash64::with_seed(0);
        std::hash::Hash::hash(&self, &mut hasher);
        std::hash::Hasher::finish(&hasher)
    }

    fn handle_event(&mut self, event: MouseDownEvent) {
        let body = nbody::Body::new(event.mouse_x, event.mouse_y, event.mass);
        self.simulation.add_body(body)
    }

    pub fn step(&mut self) {
        while let Some(input) = self.input_buffer.next(self.frame_index) {
            if input.frame_index == self.frame_index {
                self.handle_event(input.state);
            } else if input.frame_index < self.frame_index {
                log::warn!(
                    "missed input for frame {}. current frame: {}",
                    input.frame_index,
                    self.frame_index
                );
            } else {
                break;
            }
        }
        self.simulation.step();
        self.frame_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_stepping() {
        let mut state = State::new();

        for _ in 0..100 {
            state.step();
        }
    }

    #[test]
    fn serde_sanity() {
        let send_control = vec![
            Send::Ping(6),
            Send::InputState(IndexedState {
                frame_index: 541093,
                state: MouseDownEvent::new(272., 335., 802.6582641602),
            }),
        ];
        let bin = bincode::serialize(&send_control).unwrap();
        let send: Vec<Send> = bincode::deserialize(&bin).unwrap();
        assert_eq!(send_control, send);
    }
}
