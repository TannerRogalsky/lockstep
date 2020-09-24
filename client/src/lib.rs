use wasm_bindgen::prelude::*;

mod connection;
mod latency_buffer;

use connection::Connection;
use latency_buffer::LatencyBuffer;

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
}

#[wasm_bindgen]
pub struct State {
    inner: shared::State,
    input_state: shared::InputState,
    connection: Connection,
    latency_buffer: LatencyBuffer,
    hashes: Vec<(shared::FrameIndex, u64)>,
}

#[wasm_bindgen]
impl State {
    #[wasm_bindgen(constructor)]
    pub fn new(connection: Connection) -> State {
        Self::new_with_state(shared::State::new(), connection)
    }

    #[wasm_bindgen]
    pub fn from_raw(data: &[u8], connection: Connection) -> Result<State, JsValue> {
        let inner: shared::State =
            bincode::deserialize(data).map_err(|e| JsValue::from_str(&format!("{}", e)))?;
        Ok(Self::new_with_state(inner, connection))
    }

    fn new_with_state(inner: shared::State, connection: Connection) -> State {
        Self {
            inner,
            input_state: Default::default(),
            connection,
            latency_buffer: LatencyBuffer::with_timeout(std::time::Duration::from_secs(1)),
            hashes: Default::default(),
        }
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<(), JsValue> {
        self.connection
            .send(&bincode::serialize(&shared::Send::Ping(self.inner.frame_index)).unwrap())?;
        self.latency_buffer.send(self.inner.frame_index);
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
                                        "Hash mismatch for frame {}. Expected {:x} but found {:x}",
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
            let mut hasher = twox_hash::XxHash64::with_seed(0);
            std::hash::Hash::hash(&self.inner.simulation, &mut hasher);
            let hash = std::hash::Hasher::finish(&hasher);
            self.hashes.push((self.inner.frame_index, hash));
        }
        self.inner.step();
        r
    }

    #[wasm_bindgen]
    pub fn input_state_changed(&mut self, input_state: shared::InputState) {
        self.input_state = input_state;
    }

    #[wasm_bindgen]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        serde_wasm_bindgen::to_value(&self.inner).map_err(Into::into)
    }

    #[wasm_bindgen]
    pub fn latency_secs(&self) -> f32 {
        self.latency_buffer.average_latency().as_secs_f32()
    }

    #[wasm_bindgen]
    pub fn packet_loss(&self) -> f32 {
        self.latency_buffer.packet_loss()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
