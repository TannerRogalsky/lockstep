use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct RendererResources {
    shader_2d: String,
    shader_instanced: String,
}

#[wasm_bindgen]
impl RendererResources {
    #[wasm_bindgen(constructor)]
    pub fn new(shader_2d: String, shader_instanced: String) -> Self {
        Self {
            shader_2d,
            shader_instanced,
        }
    }
}

#[wasm_bindgen]
pub struct Renderer(renderer::Renderer);

#[wasm_bindgen]
impl Renderer {
    #[wasm_bindgen(constructor)]
    pub fn new(
        ctx: web_sys::WebGlRenderingContext,
        resources: RendererResources,
        width: u32,
        height: u32,
    ) -> Result<Renderer, JsValue> {
        let glow_ctx = renderer::solstice::glow::Context::from_webgl1_context(ctx);
        let context = renderer::solstice::Context::new(glow_ctx);
        let resources = renderer::Resources {
            shader_2d_src: &resources.shader_2d,
            body_shader_src: &resources.shader_instanced,
        };
        let inner = renderer::Renderer::new(context, resources, width, height)
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
    pub fn move_camera(&mut self, dx: f32, dy: f32) {
        self.0.move_camera(dx, dy)
    }

    #[wasm_bindgen]
    pub fn zoom(&mut self, delta: f32) {
        if delta < 0. {
            self.0.zoom_in();
        } else {
            self.0.zoom_out();
        }
    }

    #[wasm_bindgen]
    pub fn screen_to_world(&self, x: f32, y: f32) -> Box<[f32]> {
        let (x, y) = self.0.screen_to_world(x, y);
        Box::new([x, y])
    }
}
