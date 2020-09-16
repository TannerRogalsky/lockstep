use wasm_bindgen::prelude::*;

mod connection;
pub use connection::Connection;

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub struct State {
    inner: shared::State<()>,
    input_state: shared::InputState,
    connection: Connection,
    latency_buffer: LatencyBuffer,
    hashes: Vec<(shared::FrameIndex, u64)>,
}

#[wasm_bindgen]
impl State {
    #[wasm_bindgen(constructor)]
    pub fn new(connection: Connection) -> State {
        Self::new_with_state(shared::State::new(()), connection)
    }

    #[wasm_bindgen]
    pub fn with_physics_raw(data: &[u8], connection: Connection) -> Result<State, JsValue> {
        let physics_state: shared::IndexedState<shared::PhysicsState> =
            bincode::deserialize(data).map_err(|e| JsValue::from_str(&format!("{}", e)))?;
        let mut state = shared::State::new_with_state(physics_state.state, ());
        state.frame_index = physics_state.frame_index;
        Ok(Self::new_with_state(state, connection))
    }

    fn new_with_state(inner: shared::State<()>, connection: Connection) -> State {
        Self {
            inner,
            input_state: Default::default(),
            connection,
            latency_buffer: Default::default(),
            hashes: Default::default(),
        }
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<(), JsValue> {
        self.connection
            .send(&bincode::serialize(&shared::Send::Ping(self.inner.frame_index)).unwrap())?;
        self.latency_buffer
            .send(self.inner.frame_index, instant::Instant::now());
        while let Some(input) = self.connection.recv() {
            if let Ok(input) = bincode::deserialize::<shared::Recv>(&input) {
                match input {
                    shared::Recv::Pong(frame_index) => {
                        self.latency_buffer.recv(frame_index);
                    }
                    shared::Recv::StateHash(shared::IndexedState {
                        frame_index,
                        state: hash,
                    }) => {
                        for (other_frame_index, other_hash) in self.hashes.iter() {
                            if other_frame_index == &frame_index {
                                if other_hash != &hash {
                                    log::error!(
                                        "Hash mismatch for frame {}. Expected {} but found {}",
                                        frame_index,
                                        hash,
                                        other_hash
                                    );
                                }
                                break;
                            }
                        }
                    }
                    shared::Recv::FullState(_) => unimplemented!(),
                }
            }
        }

        // TODO: error-prone: frame changes when?
        let to_send = shared::Send::InputState(shared::IndexedState {
            frame_index: self.inner.frame_index,
            state: self.input_state,
        });
        let r = match bincode::serialize(&to_send) {
            Ok(state) => self.connection.send(&state),
            Err(err) => Err(JsValue::from_str(&err.to_string())),
        };
        {
            let buf = bincode::serialize(&self.inner.physics_state).unwrap();
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            std::hash::Hash::hash(&buf, &mut hasher);
            let hash = std::hash::Hasher::finish(&hasher);
            self.hashes.push((self.inner.frame_index, hash));
        }
        self.inner.step(self.input_state);
        r
    }

    #[wasm_bindgen]
    pub fn input_state_changed(&mut self, input_state: shared::InputState) {
        self.input_state = input_state;
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner.physics_state.colliders).map_err(Into::into)
    }

    #[wasm_bindgen]
    pub fn latency_secs(&self) -> f32 {
        self.latency_buffer.average_latency().as_secs_f32()
    }
}

#[derive(Clone, Debug, Default)]
struct LatencyBuffer {
    buffer: Vec<(shared::FrameIndex, instant::Instant)>,
    timings: Vec<std::time::Duration>,
}

impl LatencyBuffer {
    pub fn send(&mut self, index: shared::FrameIndex, time: instant::Instant) {
        self.buffer.push((index, time))
    }

    pub fn recv(&mut self, frame: shared::FrameIndex) -> std::time::Duration {
        for (index, (other, instant)) in self.buffer.iter().enumerate() {
            if frame == *other {
                let d = instant.elapsed();
                self.timings.push(d);
                self.buffer.swap_remove(index);
                return d;
            }
        }
        unreachable!()
    }

    pub fn average_latency(&self) -> std::time::Duration {
        if self.timings.is_empty() {
            std::time::Duration::default()
        } else {
            self.timings.iter().sum::<std::time::Duration>() / self.timings.len() as u32
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
