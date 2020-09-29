#[cfg(not(target_arch = "wasm32"))]
pub extern crate glutin;
pub extern crate graphics;

#[derive(graphics::vertex::Vertex)]
#[repr(C)]
struct Vertex2D {
    position: [f32; 2],
    color: [f32; 4],
    uv: [f32; 2],
}

pub struct Renderer {
    context: graphics::Context,
    shader: graphics::shader::DynamicShader,
    dimensions: (u32, u32),
    circle: graphics::mesh::VertexMesh<Vertex2D>,
}

impl Renderer {
    pub fn new(
        mut context: graphics::Context,
        width: u32,
        height: u32,
    ) -> Result<Self, graphics::GraphicsError> {
        let shader = {
            const SRC: &str = include_str!("shader.glsl");
            let (vert, frag) = graphics::shader::DynamicShader::create_source(SRC, SRC);
            graphics::shader::DynamicShader::new(&mut context, &vert, &frag)?
        };
        let dimensions = (width, height);

        let circle = {
            const POINTS: u32 = 50;
            let vertices = (0..POINTS)
                .map(|i| {
                    let r = i as f32 / POINTS as f32;
                    let phi = r * std::f32::consts::PI * 2.;

                    let (x, y) = phi.sin_cos();

                    Vertex2D {
                        position: [x, y],
                        color: [1., 1., 1., 1.],
                        uv: [0.5, 0.5],
                    }
                })
                .collect::<Box<_>>();

            let mut mesh = graphics::mesh::VertexMesh::with_data(&mut context, &vertices)?;
            mesh.set_draw_mode(graphics::DrawMode::TriangleFan);
            mesh
        };

        Ok(Self {
            context,
            shader,
            dimensions,
            circle,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.dimensions = (width, height);
    }

    pub fn render(&mut self, state: &shared::State) {
        self.context.clear_color(1., 0., 0., 1.);
        self.context.clear();

        let (width, height) = self.dimensions;
        self.context.set_viewport(0, 0, width as _, height as _);

        self.context.use_shader(Some(&self.shader));
        self.context.set_uniform_by_location(
            &self
                .shader
                .get_uniform_by_name("uProjection")
                .unwrap()
                .location,
            &graphics::shader::RawUniformValue::Mat4(
                nalgebra::geometry::Orthographic3::new(0., width as _, height as _, 0., 0., 100.)
                    .into_inner()
                    .into(),
            ),
        );
        self.context.set_uniform_by_location(
            &self.shader.get_uniform_by_name("uView").unwrap().location,
            &graphics::shader::RawUniformValue::Mat4(nalgebra::Matrix4::<f32>::identity().into()),
        );
        self.context.set_uniform_by_location(
            &self.shader.get_uniform_by_name("uColor").unwrap().location,
            &graphics::shader::RawUniformValue::Vec4([1., 1., 1., 1.].into()),
        );

        for body in state.simulation.bodies.iter() {
            let translation = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                body.position.x.to_num(),
                body.position.y.to_num(),
                0.,
            ));
            let scale = nalgebra::Matrix4::new_scaling(body.radius().to_num());
            let transform = translation * scale;
            self.context.set_uniform_by_location(
                &self.shader.get_uniform_by_name("uModel").unwrap().location,
                &graphics::shader::RawUniformValue::Mat4(transform.into()),
            );
            graphics::Renderer::draw(
                &mut self.context,
                &self.shader,
                graphics::Geometry {
                    mesh: &self.circle,
                    draw_range: 0..1,
                    draw_mode: graphics::DrawMode::TriangleFan,
                    instance_count: 1,
                },
                graphics::PipelineSettings::default(),
            )
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
