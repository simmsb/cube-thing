use crate::frame::Frame;

pub trait Backend {
    fn display_frame(&mut self, frame: &Frame);
}
