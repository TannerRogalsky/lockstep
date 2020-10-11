use super::Vertex3D as Vertex;

#[derive(Debug)]
pub struct SphereGeometrySettings {
    pub radius: f32,
    pub width_segments: u32,
    pub height_segments: u32,
    pub phi_start: f32,
    pub phi_length: f32,
    pub theta_start: f32,
    pub theta_length: f32,
}

impl Default for SphereGeometrySettings {
    fn default() -> Self {
        Self {
            radius: 1.,
            width_segments: 8,
            height_segments: 6,
            phi_start: 0.0,
            phi_length: std::f32::consts::PI * 2.,
            theta_start: 0.0,
            theta_length: std::f32::consts::PI,
        }
    }
}

impl SphereGeometrySettings {
    pub fn build(&self) -> (Vec<Vertex>, Vec<u16>) {
        let mut vertices = vec![];
        let mut indices = vec![];

        let theta_end = std::f32::consts::PI.min(self.theta_start + self.theta_length);

        let mut index = 0;
        let mut grid = vec![];

        for y in 0..=self.height_segments {
            let mut vertices_row = vec![];
            let v = y as f32 / self.height_segments as f32;

            // special consideration for the poles
            let u_offset = if y == 0 && self.theta_start == 0. {
                0.5 / self.width_segments as f32
            } else if y == self.height_segments && theta_end == std::f32::consts::PI {
                -0.5 / self.width_segments as f32
            } else {
                0.
            };

            for x in 0..=self.width_segments {
                let u = x as f32 / self.width_segments as f32;

                let position = nalgebra::Vector3::new(
                    -self.radius
                        * (self.phi_start + u * self.phi_length).cos()
                        * (self.theta_start + v * self.theta_length).sin(),
                    self.radius * (self.theta_start + v * self.theta_length).cos(),
                    self.radius
                        * (self.phi_start + u * self.phi_length).sin()
                        * (self.theta_start + v * self.theta_length).sin(),
                );
                vertices.push(super::Vertex3D {
                    normal: position.normalize().into(),
                    position: position.into(),
                    uv: [u + u_offset, v],
                });
                vertices_row.push(index);
                index += 1;
            }
            grid.push(vertices_row);
        }

        for iy in 0..self.height_segments as usize {
            for ix in 0..self.width_segments as usize {
                let a = grid[iy][ix + 1];
                let b = grid[iy][ix];
                let c = grid[iy + 1][ix];
                let d = grid[iy + 1][ix + 1];

                if iy as u32 != 0 || self.theta_start > 0. {
                    indices.extend_from_slice(&[a, b, d]);
                }
                if iy as u32 != self.height_segments - 1 || theta_end < std::f32::consts::PI {
                    indices.extend_from_slice(&[b, c, d]);
                }
            }
        }

        (vertices, indices)
    }
}
