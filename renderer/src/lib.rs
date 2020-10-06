pub extern crate graphics;

#[derive(graphics::vertex::Vertex)]
#[repr(C)]
struct Vertex2D {
    position: [f32; 2],
    color: [f32; 4],
}

#[derive(graphics::vertex::Vertex)]
#[repr(C)]
struct Position {
    position: [f32; 2],
}

#[derive(graphics::vertex::Vertex)]
#[repr(C)]
struct Instance {
    color: [f32; 4],
    offset: [f32; 2],
    scale: f32,
}

fn to_num_point(p: shared::nbody::Point2D) -> nalgebra::Point2<f32> {
    nalgebra::Point2::new(p.x.to_num(), p.y.to_num())
}

fn to_num_vec(v: shared::nbody::Vector2D) -> nalgebra::Vector2<f32> {
    nalgebra::Vector2::new(v.x.to_num(), v.y.to_num())
}

struct LineBuffer {
    inner: graphics::mesh::MappedVertexMesh<Vertex2D>,
    offset: usize,
}

impl LineBuffer {
    pub fn new(context: &mut graphics::Context) -> Result<Self, graphics::GraphicsError> {
        let inner = graphics::mesh::MappedVertexMesh::new(context, 1000)?;
        Ok(Self { inner, offset: 0 })
    }

    pub fn add(&mut self, body: &shared::nbody::Body) {
        const VEC_SCALE: f32 = 100.;
        let origin = to_num_point(body.position);
        let accel = origin + to_num_vec(body.acceleration) * VEC_SCALE;
        let vel = origin + to_num_vec(body.velocity) * VEC_SCALE;

        let green = [0., 1., 0., 1.];
        let red = [1., 0., 0., 1.];

        let vertices = [
            Vertex2D {
                position: [origin.x, origin.y],
                color: green,
            },
            Vertex2D {
                position: [accel.x, accel.y],
                color: green,
            },
            Vertex2D {
                position: [origin.x, origin.y],
                color: red,
            },
            Vertex2D {
                position: [vel.x, vel.y],
                color: red,
            },
        ];

        self.inner.set_vertices(&vertices, self.offset);
        self.offset += 4;
    }

    pub fn unmap(
        &mut self,
        context: &mut graphics::Context,
    ) -> graphics::Geometry<&graphics::mesh::VertexMesh<Vertex2D>> {
        let draw_range = 0..self.offset;
        self.offset = 0;
        graphics::Geometry {
            mesh: self.inner.unmap(context),
            draw_range,
            draw_mode: graphics::DrawMode::Lines,
            instance_count: 1,
        }
    }
}

pub struct Renderer {
    context: graphics::Context,
    shader: graphics::shader::DynamicShader,
    instanced_shader: graphics::shader::DynamicShader,
    dimensions: (u32, u32),
    circle: graphics::mesh::VertexMesh<Position>,
    vectors: LineBuffer,
    camera_position: nalgebra::Point2<f32>,
    instances: graphics::mesh::MappedVertexMesh<Instance>,
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
        let instanced_shader = {
            const SRC: &str = include_str!("instanced.glsl");
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

                    Position {
                        position: [x, y]
                    }
                })
                .collect::<Box<_>>();

            graphics::mesh::VertexMesh::with_data(&mut context, &vertices)?
        };

        let instances = graphics::mesh::MappedVertexMesh::new(&mut context, 10000)?;

        let vectors = LineBuffer::new(&mut context)?;

        Ok(Self {
            context,
            shader,
            instanced_shader,
            dimensions,
            circle,
            vectors,
            camera_position: nalgebra::Point2::new(0., 0.),
            instances,
        })
    }

    pub fn camera_position(&self) -> &nalgebra::Point2<f32> {
        &self.camera_position
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.dimensions = (width, height);
    }

    pub fn render(&mut self, state: &shared::State) {
        self.context.clear_color(0., 0., 0., 1.);
        self.context.clear();

        let (width, height) = self.dimensions;
        self.context.set_viewport(0, 0, width as _, height as _);
        self.camera_position = to_num_point(state.simulation.center_of_mass())
            - nalgebra::Vector2::new(width as f32 / 2., height as f32 / 2.);

        set_uniforms(
            &mut self.context,
            &self.instanced_shader,
            self.dimensions,
            &self.camera_position,
        );
        set_uniforms(
            &mut self.context,
            &self.shader,
            self.dimensions,
            &self.camera_position,
        );

        for (index, body) in state.simulation.bodies.iter().enumerate() {
            self.instances.set_vertices(
                &[Instance {
                    color: [1., 1., 1., 1.],
                    offset: [body.position.x.to_num(), body.position.y.to_num()],
                    scale: body.radius().to_num(),
                }],
                index,
            );

            self.vectors.add(body);
        }
        let instances = self.instances.unmap(&mut self.context);
        let attached = graphics::mesh::MeshAttacher::attach_with_step(&self.circle, instances, 1);
        graphics::Renderer::draw(
            &mut self.context,
            &self.instanced_shader,
            &graphics::Geometry {
                mesh: attached,
                draw_range: self.circle.draw_range(),
                draw_mode: graphics::DrawMode::TriangleFan,
                instance_count: state.simulation.bodies.len() as u32,
            },
            graphics::PipelineSettings::default(),
        );

        self.context.use_shader(Some(&self.shader));
        self.context.set_uniform_by_location(
            &self.shader.get_uniform_by_name("uModel").unwrap().location,
            &graphics::shader::RawUniformValue::Mat4(nalgebra::Matrix4::<f32>::identity().into()),
        );
        let geometry = self.vectors.unmap(&mut self.context);
        graphics::Renderer::draw(
            &mut self.context,
            &self.shader,
            &geometry,
            graphics::PipelineSettings::default(),
        )
    }
}

fn set_uniforms(
    context: &mut graphics::Context,
    shader: &graphics::shader::DynamicShader,
    dimensions: (u32, u32),
    camera_position: &nalgebra::Point2<f32>,
) {
    let (width, height) = dimensions;
    context.use_shader(Some(shader));
    context.set_uniform_by_location(
        &shader.get_uniform_by_name("uProjection").unwrap().location,
        &graphics::shader::RawUniformValue::Mat4(
            nalgebra::geometry::Orthographic3::new(0., width as _, height as _, 0., 0., 100.)
                .into_inner()
                .into(),
        ),
    );
    context.set_uniform_by_location(
        &shader.get_uniform_by_name("uModel").unwrap().location,
        &graphics::shader::RawUniformValue::Mat4(nalgebra::Matrix4::<f32>::identity().into()),
    );
    context.set_uniform_by_location(
        &shader.get_uniform_by_name("uView").unwrap().location,
        &graphics::shader::RawUniformValue::Mat4(
            nalgebra::Matrix4::new_translation(&nalgebra::Vector3::new(
                -camera_position.x,
                -camera_position.y,
                0.,
            ))
            .into(),
        ),
    );
    context.set_uniform_by_location(
        &shader.get_uniform_by_name("uColor").unwrap().location,
        &graphics::shader::RawUniformValue::Vec4([1., 1., 1., 1.].into()),
    );
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
