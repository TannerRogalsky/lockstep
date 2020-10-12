type Vector3 = nalgebra::Vector3<f32>;
type Vector2 = nalgebra::Vector2<f32>;

#[derive(Debug, Default)]
pub struct PointMass {
    pub position: Vector3,
    pub velocity: Vector3,
    pub inverse_mass: f32,
    acceleration: Vector3,
    damping: f32,
}

impl PointMass {
    pub fn new(position: Vector3, inverse_mass: f32) -> Self {
        Self {
            position,
            velocity: Vector3::new(0., 0., 0.),
            acceleration: Vector3::new(0., 0., 0.),
            inverse_mass,
            damping: 0.98,
        }
    }

    pub fn apply_force(&mut self, force: &Vector3) {
        self.acceleration += force * self.inverse_mass;
    }

    pub fn increase_damping(&mut self, factor: f32) {
        self.damping *= factor;
    }

    pub fn update(&mut self) {
        self.velocity += self.acceleration;
        self.position += self.velocity;
        self.acceleration = Vector3::new(0., 0., 0.);

        if self.velocity.magnitude_squared() < 0.001 * 0.001 {
            self.velocity = Vector3::new(0., 0., 0.);
        }

        self.velocity *= self.damping;
        self.damping = 0.98;
    }
}

pub struct Spring {
    pub end_index1: usize,
    pub end_index2: usize,
    pub target_length: f32,
    pub stiffness: f32,
    pub damping: f32,
}

impl Spring {
    pub fn update(&mut self, points: &mut [PointMass]) {
        let p1 = &points[self.end_index1];
        let p2 = &points[self.end_index2];

        let x = p1.position - p2.position;
        let length = x.magnitude();

        if length > self.target_length {
            let x = (x / length) * (length - self.target_length);
            let dv = p2.velocity - p1.velocity;
            let force = self.stiffness * x - dv * self.damping;

            points[self.end_index1].apply_force(&-force);
            points[self.end_index2].apply_force(&force);
        }
    }
}

pub struct Grid {
    pub springs: Vec<Spring>,
    pub points: Vec<PointMass>,
    pub screen_dimensions: Vector2,
    pub grid_dimensions: (usize, usize),
}

impl Grid {
    pub fn new(
        screen_dimensions: Vector2,
        viewport: &solstice::viewport::Viewport<f32>,
        spacing: &Vector2,
    ) -> Self {
        let num_columns = (viewport.width() / spacing.x) as usize + 1;
        let num_rows = (viewport.height() / spacing.y) as usize + 1;
        let grid_dimensions = (num_columns, num_rows);

        let mut points = Vec::with_capacity(num_columns * num_rows * 2);
        let mut springs = Vec::new();

        let cart_to_index = |x: usize, y: usize| -> usize { y * num_columns + x };

        let gen_points = |points: &mut Vec<PointMass>, inverse_mass: f32| {
            for row in 0..num_rows {
                for col in 0..num_columns {
                    let y_ratio = row as f32 / (num_rows - 1) as f32;
                    let x_ratio = col as f32 / (num_columns - 1) as f32;
                    let y = viewport.y() + viewport.height() * y_ratio;
                    let x = viewport.x() + viewport.width() * x_ratio;

                    points.push(PointMass::new(Vector3::new(x, y, 0.), inverse_mass));
                }
            }
        };
        gen_points(&mut points, 1.);
        gen_points(&mut points, 0.);

        for y in 0..num_rows {
            for x in 0..num_columns {
                if x == 0 || y == 0 || x == num_columns - 1 || y == num_rows - 1 {
                    let end_index1 = (num_rows * num_columns) + cart_to_index(x, y);
                    let end_index2 = cart_to_index(x, y);
                    let p1 = &points[end_index1];
                    let p2 = &points[end_index2];
                    springs.push(Spring {
                        end_index1,
                        end_index2,
                        target_length: (&p1.position - &p2.position).norm() * 0.95,
                        stiffness: 0.1,
                        damping: 0.1,
                    });
                } else if x % 3 == 0 && y % 3 == 0 {
                    let end_index1 = (num_rows * num_columns) + cart_to_index(x, y);
                    let end_index2 = cart_to_index(x, y);
                    let p1 = &points[end_index1];
                    let p2 = &points[end_index2];
                    springs.push(Spring {
                        end_index1,
                        end_index2,
                        target_length: (&p1.position - &p2.position).norm() * 0.95,
                        stiffness: 0.002,
                        damping: 0.02,
                    });
                }

                if x > 0 {
                    let end_index1 = cart_to_index(x - 1, y);
                    let end_index2 = cart_to_index(x, y);
                    let p1 = &points[end_index1];
                    let p2 = &points[end_index2];
                    springs.push(Spring {
                        end_index1,
                        end_index2,
                        target_length: (&p1.position - &p2.position).norm() * 0.95,
                        stiffness: 0.28,
                        damping: 0.06,
                    });
                }

                if y > 0 {
                    let end_index1 = cart_to_index(x, y - 1);
                    let end_index2 = cart_to_index(x, y);
                    let p1 = &points[end_index1];
                    let p2 = &points[end_index2];
                    springs.push(Spring {
                        end_index1,
                        end_index2,
                        target_length: (&p1.position - &p2.position).norm() * 0.95,
                        stiffness: 0.28,
                        damping: 0.06,
                    });
                }
            }
        }

        Self {
            springs,
            points,
            screen_dimensions,
            grid_dimensions,
        }
    }

    #[allow(unused)]
    pub fn apply_directed_force_2d(&mut self, force: &Vector2, position: &Vector2, radius: f32) {
        self.apply_directed_force_3d(
            &Vector3::new(force.x, force.y, 0.),
            &Vector3::new(position.x, position.y, 0.),
            radius,
        )
    }
    pub fn apply_directed_force_3d(&mut self, force: &Vector3, position: &Vector3, radius: f32) {
        let (rows, cols) = self.grid_dimensions;
        for i in 0..(rows * cols) {
            if (position - &self.points[i].position).norm_squared() < radius * radius {
                let force = 10. * force / (10. + (position - &self.points[i].position).norm());
                self.points[i].apply_force(&force);
            }
        }
    }

    #[allow(unused)]
    pub fn apply_implosive_force_2d(&mut self, force: f32, position: &Vector2, radius: f32) {
        self.apply_implosive_force_3d(force, &Vector3::new(position.x, position.y, 0.), radius)
    }
    pub fn apply_implosive_force_3d(&mut self, force: f32, position: &Vector3, radius: f32) {
        let (rows, cols) = self.grid_dimensions;
        for i in 0..(rows * cols) {
            let dist2 = (position - &self.points[i].position).norm_squared();
            if dist2 < radius * radius {
                let force = 10. * force * (position - &self.points[i].position) / (100. + dist2);
                self.points[i].apply_force(&force);
                self.points[i].increase_damping(0.6);
            }
        }
    }

    pub fn apply_explosive_force_2d(&mut self, force: f32, position: &Vector2, radius: f32) {
        self.apply_explosive_force_3d(force, &Vector3::new(position.x, position.y, 0.), radius)
    }
    pub fn apply_explosive_force_3d(&mut self, force: f32, position: &Vector3, radius: f32) {
        let (rows, cols) = self.grid_dimensions;
        for i in 0..(rows * cols) {
            let dist2 = (position - &self.points[i].position).norm_squared();
            if dist2 < radius * radius {
                let force = 100. * force * (&self.points[i].position - position) / (10000. + dist2);
                self.points[i].apply_force(&force);
                self.points[i].increase_damping(0.6);
            }
        }
    }

    pub fn update(&mut self) {
        for spring in self.springs.iter_mut() {
            spring.update(&mut self.points);
        }

        // there are fixed point masses at the end of this vec
        let (rows, cols) = self.grid_dimensions;
        for point in self.points[0..(rows * cols)].iter_mut() {
            point.update();
        }
    }

    pub fn to_vec2(&self, v: &Vector3) -> Vector2 {
        let factor = (v.z + 2000.0) * 0.0005;
        return (Vector2::new(v.x, v.y) - self.screen_dimensions * 0.5) * factor
            + self.screen_dimensions * 0.5;
    }

    pub fn draw(&self, line_buffer: &mut super::line_buffer::LineBuffer) {
        let (rows, cols) = self.grid_dimensions;
        let points = |x: usize, y: usize| -> &PointMass { &self.points[y * cols + x] };

        // const COLOR: [f32; 4] = [0.12, 0.12, 0.55, 0.33];
        const COLOR: [f32; 4] = [1., 1., 1., 1.];

        for y in 0..rows {
            for x in 0..cols {
                let p = self.to_vec2(&points(x, y).position);
                if x >= 1 {
                    let left = self.to_vec2(&points(x - 1, y).position);
                    line_buffer.line(left.into(), p.into(), COLOR);
                }
                if y >= 1 {
                    let up = self.to_vec2(&points(x, y - 1).position);
                    line_buffer.line(up.into(), p.into(), COLOR);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_test() {
        let (x1, y1, x2, y2) = (0., 0., 100., 100.);

        let grid = Grid::new(
            nalgebra::Vector2::new(x2, y2),
            &solstice::viewport::Viewport::new(x1, y1, x2, y2),
            &Vector2::new(x2, y2),
        );

        assert_eq!(grid.grid_dimensions, (2, 2));

        let points = grid.points[0..4]
            .iter()
            .map(|p| grid.to_vec2(&p.position))
            .collect::<Vec<_>>();
        assert_eq!(points[0], nalgebra::Vector2::new(x1, y1));
        assert_eq!(points[1], nalgebra::Vector2::new(x2, y1));
        assert_eq!(points[2], nalgebra::Vector2::new(x1, y2));
        assert_eq!(points[3], nalgebra::Vector2::new(x2, y2));

        assert_eq!(grid.points.len(), 8);
    }
}
