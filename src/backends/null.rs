use super::backend::Backend;

pub struct NullBackend;

impl Backend for NullBackend {
    fn display_frame(&mut self, _frame: &crate::frame::Frame) {
    }
}
