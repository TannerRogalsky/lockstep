#[derive(solstice::vertex::Vertex)]
#[repr(C)]
pub struct Vertex2D {
    position: [f32; 2],
    color: [f32; 4],
}

fn to_num_vec(v: shared::nbody::Vector2D) -> nalgebra::Vector2<f32> {
    nalgebra::Vector2::new(v.x.to_num(), v.y.to_num())
}

pub struct LineBuffer {
    inner: solstice::mesh::MappedVertexMesh<Vertex2D>,
    offset: usize,
}

impl LineBuffer {
    pub fn new(context: &mut solstice::Context) -> Result<Self, solstice::GraphicsError> {
        let inner = solstice::mesh::MappedVertexMesh::new(context, super::MAX_PARTICLES * 4)?;
        Ok(Self { inner, offset: 0 })
    }

    pub fn line(&mut self, start: [f32; 2], end: [f32; 2], color: [f32; 4]) {
        let vertices = [
            Vertex2D {
                position: start,
                color,
            },
            Vertex2D {
                position: end,
                color,
            },
        ];

        self.inner.set_vertices(&vertices, self.offset);
        self.offset += 2;
    }

    pub fn add(&mut self, body: &shared::nbody::Body) {
        const VEC_SCALE: f32 = 100.;
        let origin = nalgebra::Point2::new(body.position.x.to_num(), body.position.y.to_num());
        let accel = origin + to_num_vec(body.acceleration) * VEC_SCALE;
        let vel = origin + to_num_vec(body.velocity) * VEC_SCALE;

        let green = [0., 1., 0., 1.];
        let red = [1., 0., 0., 1.];

        let origin = [origin.x, origin.y];
        let accel = [accel.x, accel.y];
        let vel = [vel.x, vel.y];

        self.line(origin, accel, green);
        self.line(origin, vel, red);
    }

    pub fn unmap(
        &mut self,
        context: &mut solstice::Context,
    ) -> solstice::Geometry<&solstice::mesh::VertexMesh<Vertex2D>> {
        let draw_range = 0..self.offset;
        self.offset = 0;
        solstice::Geometry {
            mesh: self.inner.unmap(context),
            draw_range,
            draw_mode: solstice::DrawMode::Lines,
            instance_count: 1,
        }
    }
}
