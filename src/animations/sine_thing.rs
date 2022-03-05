use crate::render::{Animation, Frame};

#[derive(Default)]
pub struct SineThing {
    step: f32,
}

impl Animation for SineThing {
    fn next_frame(&mut self, frame: &mut Frame) {
        for layer in 0..Frame::LAYERS {
            let brightness =
                ((((self.step + layer as f32) * 8.0).to_radians().sin() + 1.0) * 127.0) as u8;

            frame.layer_mut(layer).fill([brightness; 8]);
        }

        self.step += 1.0;
        self.step %= 360.0 / 8.0;
    }

    fn reset(&mut self) {
        self.step = 0.0;
    }
}
