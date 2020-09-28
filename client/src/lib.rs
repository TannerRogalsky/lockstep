use serde::{Deserialize, Serialize};
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
    /// The most recent frame index that we've received from the server.
    server_frame: shared::FrameIndex,
}

#[wasm_bindgen]
impl State {
    #[wasm_bindgen(constructor)]
    pub fn new(connection: Connection) -> State {
        Self::new_with_state(shared::State::new(), connection)
    }

    #[wasm_bindgen]
    pub fn from_raw(data: &[u8], connection: Connection) -> Result<State, JsValue> {
        let (hash, inner) = bincode::deserialize::<(u64, shared::State)>(data)
            .map_err(|e| JsValue::from_str(&format!("{}", e)))?;
        assert_eq!(hash, inner.hash());
        Ok(Self::new_with_state(inner, connection))
    }

    fn new_with_state(inner: shared::State, connection: Connection) -> State {
        let server_frame = inner.frame_index;
        Self {
            inner,
            connection,
            latency_buffer: LatencyBuffer::with_timeout(std::time::Duration::from_secs(1)),
            hashes: Default::default(),
            server_frame,
        }
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<(), JsValue> {
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
                        self.server_frame = self.server_frame.max(frame_index);
                        // TODO: clear out old hashes
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

        match bincode::serialize(&shared::Send::Ping(self.inner.frame_index)) {
            Ok(buf) => self.connection.send(&buf)?,
            Err(err) => log::error!("serialization error: {}", err),
        }
        self.latency_buffer.send(self.inner.frame_index);

        if self.inner.frame_index > self.target_frame() {
            return Ok(());
        }

        let hash_pair = (self.inner.frame_index, self.inner.hash());
        self.hashes.push(hash_pair);
        self.inner.step();
        // TODO: if client is behind the server, run multiple steps
        Ok(())
    }

    #[wasm_bindgen]
    pub fn mouse_click_event(&mut self, down_x: f32, down_y: f32, mass: f32, up_x: f32, up_y: f32) {
        // TODO: it this magic number is reasonable but it should really be tied to the simulation
        const VEL_SCALE: f32 = 0.01;
        let dx = (up_x - down_x) * VEL_SCALE;
        let dy = (up_y - down_y) * VEL_SCALE;
        let input_event = shared::IndexedState {
            frame_index: self.inner.frame_index + shared::INPUT_BUFFER_FRAMES,
            state: shared::AddBodyEvent::new_with_velocity(down_x, down_y, mass, dx, dy),
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
        let render_data = self
            .inner
            .simulation
            .bodies
            .iter()
            .map(|body| RenderDataBody {
                x: body.position.x.to_num(),
                y: body.position.y.to_num(),
                radius: body.radius().to_num(),
                mass: body.mass.to_num(),
            })
            .collect::<Box<[RenderDataBody]>>();
        serde_wasm_bindgen::to_value(&render_data).map_err(Into::into)
    }

    #[wasm_bindgen]
    pub fn latency_secs(&self) -> f32 {
        self.latency_buffer.average_latency().as_secs_f32()
    }

    #[wasm_bindgen]
    pub fn packet_loss(&self) -> f32 {
        self.latency_buffer.packet_loss()
    }

    #[wasm_bindgen]
    pub fn current_frame(&self) -> shared::FrameIndex {
        self.inner.frame_index
    }

    #[wasm_bindgen]
    pub fn target_frame(&self) -> shared::FrameIndex {
        self.server_frame + self.latency_buffer.average_latency().as_millis() as u32 / 60
    }
}

#[derive(Serialize, Deserialize)]
struct RenderDataBody {
    x: f32,
    y: f32,
    radius: f32,
    mass: f32,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
