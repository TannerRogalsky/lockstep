use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct StateWrapper(super::State);

#[wasm_bindgen]
impl StateWrapper {
    #[wasm_bindgen(constructor)]
    pub fn new(
        ctx: web_sys::WebGlRenderingContext,
        width: u32,
        height: u32,
    ) -> Result<Self, JsValue> {
        let glow_ctx = graphics::glow::Context::from_webgl1_context(ctx);
        let context = graphics::Context::new(glow_ctx);
        let inner = super::State::new(context, width, height)?;
        Ok(StateWrapper(inner))
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.0.resize(glutin::dpi::PhysicalSize::new(width, height))
    }
}
