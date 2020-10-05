use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Renderer(renderer::Renderer);

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new(
        ctx: web_sys::WebGlRenderingContext,
        width: u32,
        height: u32,
    ) -> Result<Renderer, JsValue> {
        let glow_ctx = renderer::graphics::glow::Context::from_webgl1_context(ctx);
        let context = renderer::graphics::Context::new(glow_ctx);
        let inner = renderer::Renderer::new(context, width, height)
            .map_err(|err| JsValue::from_str(&format!("{:?}", err)))?;
        Ok(Renderer(inner))
    }

    #[wasm_bindgen]
    pub fn resize(&mut self, width: u32, height: u32) {
        self.0.resize(width, height)
    }

    #[wasm_bindgen]
    pub fn render(&mut self, state: &super::State) {
        self.0.render(&state.inner)
    }

    #[wasm_bindgen]
    pub fn transform_point(&self, x: f32, y: f32) -> Box<[f32]> {
        let offset = self.0.camera_position();
        Box::new([offset.x + x, offset.y + y])
    }
}
