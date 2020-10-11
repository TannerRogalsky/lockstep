pub extern crate solstice;

mod line_buffer;

const MAX_PARTICLES: usize = 10_000;

#[derive(solstice::vertex::Vertex)]
#[repr(C)]
struct Position {
    position: [f32; 2],
}

#[derive(solstice::vertex::Vertex)]
#[repr(C)]
struct Instance {
    color: [f32; 4],
    offset: [f32; 2],
    scale: f32,

#[derive(Default)]
pub struct Options {
    debug_vectors: bool,
}

pub struct Renderer {
    context: solstice::Context,
    shader: solstice::shader::DynamicShader,
    instanced_shader: solstice::shader::DynamicShader,
    dimensions: (u32, u32),
    circle: solstice::mesh::VertexMesh<Position>,
    vectors: line_buffer::LineBuffer,
    instances: solstice::mesh::MappedVertexMesh<Instance>,
    camera_position: nalgebra::Point2<f32>,
    zoom: i32,
    options: Options,
}

impl Renderer {
    pub fn new(
        mut context: solstice::Context,
        width: u32,
        height: u32,
    ) -> Result<Self, solstice::GraphicsError> {
        let shader = {
            const SRC: &str = include_str!("shader.glsl");
            let (vert, frag) = solstice::shader::DynamicShader::create_source(SRC, SRC);
            solstice::shader::DynamicShader::new(&mut context, &vert, &frag)?
        };
        let instanced_shader = {
            const SRC: &str = include_str!("instanced.glsl");
            let (vert, frag) = solstice::shader::DynamicShader::create_source(SRC, SRC);
            solstice::shader::DynamicShader::new(&mut context, &vert, &frag)?
        };

        let dimensions = (width, height);

        let circle = {
            const POINTS: u32 = 50;
            let vertices = (0..POINTS)
                .map(|i| {
                    let r = i as f32 / POINTS as f32;
                    let phi = r * std::f32::consts::PI * 2.;

                    let (x, y) = phi.sin_cos();

                    Position { position: [x, y] }
                })
                .collect::<Box<_>>();

            solstice::mesh::VertexMesh::with_data(&mut context, &vertices)?
        };

        let instances = solstice::mesh::MappedVertexMesh::new(&mut context, MAX_PARTICLES)?;

        let vectors = line_buffer::LineBuffer::new(&mut context)?;

        Ok(Self {
            context,
            shader,
            instanced_shader,
            dimensions,
            circle,
            vectors,
            camera_position: nalgebra::Point2::new(0., 0.),
            instances,
            zoom: 0,
            options: Default::default(),
        })
    }

    pub fn screen_to_world(&self, x: f32, y: f32) -> (f32, f32) {
        let (width, height) = self.dimensions;
        let (width, height) = (width as f32, height as f32);
        let projection = nalgebra::geometry::Orthographic3::new(
            -width / 2.,
            width / 2.,
            height / 2.,
            -height / 2.,
            0.,
            100.,
        )
        .into_inner();
        let zoom = (self.zoom as f32).exp();
        let camera = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
            -self.camera_position.x,
            -self.camera_position.y,
            0.,
        )) * nalgebra::Matrix4::new_scaling(zoom);
        let p = (projection * camera).try_inverse().unwrap()
            * nalgebra::Vector4::new((x / width - 0.5) * 2., (1. - y / height - 0.5) * 2., 0., 1.);
        (p.x, p.y)
    }

    pub fn camera_position(&self) -> &nalgebra::Point2<f32> {
        &self.camera_position
    }

    pub fn move_camera(&mut self, dx: f32, dy: f32) {
        self.camera_position.x -= dx;
        self.camera_position.y -= dy;
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.dimensions = (width, height);
    }

    pub fn zoom_in(&mut self) {
        self.zoom += 1;
    }

    pub fn zoom_out(&mut self) {
        self.zoom -= 1;
    }

    pub fn render(&mut self, state: &shared::State) {
        self.context.clear_color(0., 0., 0., 1.);
        self.context.clear();

        let (width, height) = self.dimensions;
        self.context.set_viewport(0, 0, width as _, height as _);

        if state.simulation.bodies.is_empty() {
            return;
        }

        for shader in &[&self.instanced_shader, &self.shader] {
            set_uniforms(
                &mut self.context,
                *shader,
                self.dimensions,
                &self.camera_position,
                (self.zoom as f32).exp(),
            );
        }

        for (index, body) in state.simulation.bodies.iter().enumerate() {
            self.instances.set_vertices(
                &[Instance {
                    color: [1., 1., 1., 1.],
                    offset: [body.position.x.to_num(), body.position.y.to_num()],
                    scale: body.radius().to_num(),
                }],
                index,
            );
            if self.options.debug_vectors {
                self.vectors.add(body);
            }
        }
        let instances = self.instances.unmap(&mut self.context);
        let attached = solstice::mesh::MeshAttacher::attach_with_step(&self.circle, instances, 1);
        solstice::Renderer::draw(
            &mut self.context,
            &self.instanced_shader,
            &solstice::Geometry {
                mesh: attached,
                draw_range: self.circle.draw_range(),
                draw_mode: solstice::DrawMode::TriangleFan,
                instance_count: state.simulation.bodies.len() as u32,
            },
            solstice::PipelineSettings::default(),
        );

        if self.options.debug_vectors {
            self.context.use_shader(Some(&self.shader));
            self.context.set_uniform_by_location(
                &self.shader.get_uniform_by_name("uModel").unwrap().location,
                &solstice::shader::RawUniformValue::Mat4(
                    nalgebra::Matrix4::<f32>::identity().into(),
                ),
            );
            let geometry = self.vectors.unmap(&mut self.context);
            solstice::Renderer::draw(
                &mut self.context,
                &self.shader,
                &geometry,
                solstice::PipelineSettings::default(),
            )
        }
    }
}

fn set_uniforms(
    context: &mut solstice::Context,
    shader: &solstice::shader::DynamicShader,
    dimensions: (u32, u32),
    camera_position: &nalgebra::Point2<f32>,
    zoom: f32,
) {
    let (width, height) = dimensions;
    let (width, height) = (width as f32, height as f32);
    context.use_shader(Some(shader));
    context.set_uniform_by_location(
        &shader.get_uniform_by_name("uProjection").unwrap().location,
        &solstice::shader::RawUniformValue::Mat4(
            nalgebra::geometry::Orthographic3::new(
                -width / 2.,
                width / 2.,
                height / 2.,
                -height / 2.,
                0.,
                100.,
            )
            .into_inner()
            .into(),
        ),
    );
    context.set_uniform_by_location(
        &shader.get_uniform_by_name("uModel").unwrap().location,
        &solstice::shader::RawUniformValue::Mat4(nalgebra::Matrix4::<f32>::identity().into()),
    );
    let camera = nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
        -camera_position.x,
        -camera_position.y,
        0.,
    )) * nalgebra::Matrix4::new_scaling(zoom);
    context.set_uniform_by_location(
        &shader.get_uniform_by_name("uView").unwrap().location,
        &solstice::shader::RawUniformValue::Mat4(camera.into()),
    );
    context.set_uniform_by_location(
        &shader.get_uniform_by_name("uColor").unwrap().location,
        &solstice::shader::RawUniformValue::Vec4([1., 1., 1., 1.].into()),
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
