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
        let (hash, inner): (u64, shared::State) =
            bincode::deserialize(data).map_err(|e| JsValue::from_str(&format!("{}", e)))?;
        assert_eq!(hash, inner.hash());
        Ok(Self::new_with_state(inner, connection))
    }

    fn new_with_state(inner: shared::State, connection: Connection) -> State {
        Self {
            inner,
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
                    shared::Recv::InputState(input) => self.inner.input_buffer.push(input),
                }
            }
        }

        {
            let hash = self.inner.hash();
            self.hashes.push((self.inner.frame_index, hash));
        }
        self.inner.step();
        Ok(())
    }

    #[wasm_bindgen]
    pub fn mouse_down(&mut self, x: f32, y: f32, mass: f32) {
        let input_event = shared::IndexedState {
            frame_index: self.inner.frame_index + shared::INPUT_BUFFER_FRAMES,
            state: shared::MouseDownEvent::new(x, y, mass),
        };
        self.inner.input_buffer.push(input_event);
        match bincode::serialize(&shared::Send::InputState(input_event)) {
            Ok(state) => {
                if let Err(err) = self.connection.send(&state) {
                    log::error!("failed send: {}", err.as_string().unwrap());
                }
            }
            Err(err) => log::error!("serialization error: {}", err),
        };
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
