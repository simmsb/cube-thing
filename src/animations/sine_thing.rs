use palette::{FromColor, Hsva, RgbHue, Srgba};

use crate::{animation::Animation, frame::Frame};

#[derive(Default)]
#[cfg_attr(feature = "visual", derive(bevy_inspector_egui::Inspectable))]
pub struct SineThing {
    step: f32,
}

impl std::fmt::Debug for SineThing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SineThing").finish()
    }
}

impl Animation for SineThing {
    fn next_frame(&mut self, frame: &mut Frame) {
        for (x, y, z, pix) in frame.pixels_mut() {
            let dist =
                ((4.0 - x as f32).powi(2) + (4.0 - y as f32).powi(2) + (4.0 - z as f32).powi(2))
                    .sqrt();

            let alpha = (((self.step * 3.0 + dist) * 40.0).to_radians().sin() + 1.0) / 2.0;
            let hue = RgbHue::from_radians(((self.step + dist) * 30.0).to_radians());
            let hsv = Hsva::new(hue, 1.0, 1.0, alpha);
            *pix = Srgba::from_color(hsv).into_linear()
        }

        self.step += 0.1;
        self.step %= 360.0 / 30.0;
    }

    fn reset(&mut self) {
        self.step = 0.0;
    }
}
