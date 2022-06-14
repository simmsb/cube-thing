use crate::{
    animation::Animation,
    animations::utils::{random_colour, random_rotation},
    frame::Frame,
    sdf::{render_sdf, MultiUnion},
};
use nalgebra::{vector, Rotation3, Vector3};
use palette::LinSrgba;
use rand::Rng;

#[derive(Default)]
pub struct SpinningLines {
    /// (rotation, offset)
    lines: Vec<(Rotation3<f32>, Vector3<f32>, LinSrgba)>,

    line_cache: Vec<sdfu::Line<f32, Vector3<f32>>>,
}

#[cfg(feature = "visual")]
impl bevy_inspector_egui::Inspectable for SpinningLines {
    type Attributes = <() as bevy_inspector_egui::Inspectable>::Attributes;

    fn ui(
        &mut self,
        ui: &mut bevy_inspector_egui::egui::Ui,
        options: Self::Attributes,
        context: &mut bevy_inspector_egui::Context,
    ) -> bool {
        <() as bevy_inspector_egui::Inspectable>::ui(&mut (), ui, options, context)
    }
}

impl std::fmt::Debug for SpinningLines {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SpinningLines").finish()
    }
}

impl Animation for SpinningLines {
    fn next_frame(&mut self, frame: &mut Frame) {
        let mut rng = rand::thread_rng();

        const MIN_LINES: usize = 3;

        let to_add = if self.lines.len() < MIN_LINES {
            MIN_LINES - self.lines.len()
        } else if self.lines.len() < 10 && rng.gen_bool(0.002) {
            1
        } else {
            0
        };

        let to_remove = if self.lines.len() > 3 && rng.gen_bool(0.002) {
            1
        } else {
            0
        };

        for _ in 0..to_add {
            let rotation = random_rotation();

            let translation = vector![
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0),
                rng.gen_range(-2.0..2.0)
            ];

            let colour = random_colour(1.0, 1.0, 1.0);

            self.lines.push((rotation, translation, colour));
        }

        for _ in 0..to_remove {
            self.lines.remove(rng.gen_range(0..self.lines.len()));
        }

        for line in &mut self.lines {
            line.0 = Rotation3::from_axis_angle(&Vector3::y_axis(), 0.01) * line.0;
        }

        self.line_cache.clear();
        self.line_cache
            .extend(self.lines.iter().cloned().map(|(rot, trans, colour)| {
                let base_trans = vector![4.0, 4.0, 4.0];

                let start = rot * (vector![0.0, 0.0, 100.0] + trans) + base_trans;
                let end = rot * (vector![0.0, 0.0, -100.0] + trans) + base_trans;

                sdfu::Line::new(start, end, 0.0, colour)
            }));

        let union = MultiUnion::hard(&self.line_cache);

        render_sdf(union, frame);
    }

    fn reset(&mut self) {
        self.lines.clear();
        self.line_cache.clear();
    }
}
