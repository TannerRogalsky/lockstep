pub extern crate nbody;

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
    FullState(State),
}

#[wasm_bindgen]
#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct InputState {
    mouse_x: nbody::Float,
    mouse_y: nbody::Float,
    mouse_down: nbody::Float,
}

#[wasm_bindgen]
impl InputState {
    #[wasm_bindgen(constructor)]
    pub fn new(mouse_x: f32, mouse_y: f32, mouse_down: bool) -> Self {
        Self {
            mouse_x: nbody::Float::from_num(mouse_x),
            mouse_y: nbody::Float::from_num(mouse_y),
            mouse_down: nbody::Float::from_num(mouse_down),
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize, Deserialize)]
pub struct IndexedState<T> {
    pub frame_index: FrameIndex,
    pub state: T,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct State {
    pub simulation: nbody::Simulation,
    pub frame_index: FrameIndex,
}

impl State {
    pub fn new() -> Self {
        Self {
            simulation: Default::default(),
            frame_index: 0,
        }
    }

    pub fn step(&mut self) {
        self.simulation.step();
        self.frame_index += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut state = State::new();

        for _ in 0..100 {
            state.step();
        }
    }
}
