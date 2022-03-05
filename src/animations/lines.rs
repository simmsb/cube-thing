use crate::render::{Animation, Frame};
use crate::sdf::{render_sdf, MultiUnion};
use rand::Rng;
use ultraviolet as uv;

#[derive(Default)]
pub struct SpinningLines {
    /// (rotation, offset)
    lines: Vec<(uv::Rotor3, uv::Vec3)>,

    line_cache: Vec<sdfu::Line<f32, uv::Vec3>>,
}

impl Animation for SpinningLines {
    fn next_frame(&mut self, frame: &mut Frame) {
        let mut rng = rand::thread_rng();

        const MIN_LINES: usize = 3;

        let to_add = if self.lines.len() < MIN_LINES {
            MIN_LINES - self.lines.len()
        } else if self.lines.len() < 10 && rng.gen_range(0..500) < 1 {
            1
        } else {
            0
        };

        let to_remove = if self.lines.len() > 3 && rng.gen_range(0..500) < 1 {
            1
        } else {
            0
        };

        for _ in 0..to_add {
            let rotation = uv::Rotor3::from_euler_angles(
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
                rng.gen_range(0.0..std::f32::consts::TAU),
            );

            let translation = uv::Vec3::new(
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
            );

            self.lines.push((rotation, translation));
        }

        for _ in 0..to_remove {
            self.lines.remove(rng.gen_range(0..self.lines.len()));
        }

        for line in &mut self.lines {
            line.0 = uv::Rotor3::from_rotation_xz(0.01) * line.0;
        }

        self.line_cache.clear();
        self.line_cache
            .extend(self.lines.iter().cloned().map(|(rot, trans)| {
                let base_trans = uv::Vec3::new(4.0, 4.0, 4.0);

                let start = (uv::Vec3::new(0.0, 0.0, 100.0) + trans).rotated_by(rot) + base_trans;
                let end = (uv::Vec3::new(0.0, 0.0, -100.0) + trans).rotated_by(rot) + base_trans;

                sdfu::Line::new(start, end, 0.0)
            }));

        let union = MultiUnion::hard(&self.line_cache);

        render_sdf(union, frame);
    }

    fn reset(&mut self) {
        self.lines.clear();
        self.line_cache.clear();
    }
}
