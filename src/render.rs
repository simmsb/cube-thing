use crate::{animation::MaybeTerminatingAnimation, backends::backend::Backend, frame::Frame};

pub struct Driver<Anim, Backend> {
    animation: Anim,
    backend: Backend,
    frame: Frame,
}

impl<A, B> Driver<A, B>
where
    A: MaybeTerminatingAnimation + Send + Sync,
    B: Backend + Send + Sync,
{
    pub fn new(animation: A, backend: B) -> Self {
        let frame = Frame::new();

        Self {
            animation,
            backend,
            frame,
        }
    }

    pub fn reset(&mut self) {
        self.animation.reset();
    }

    pub fn step(&mut self) {
        if self.animation.maybe_ended() {
            self.animation.reset();
        }

        self.animation.next_frame(&mut self.frame);
        self.backend.display_frame(&self.frame);
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }
}

pub struct DynDriver {
    animation: Box<dyn MaybeTerminatingAnimation + Send + Sync + 'static>,
    backend: Box<dyn Backend + Send + Sync + 'static>,
    frame: Frame,
}

impl DynDriver {
    pub fn new<A, B>(animation: A, backend: B) -> Self
    where
        A: MaybeTerminatingAnimation + Send + Sync + 'static,
        B: Backend + Send + Sync + 'static,
    {
        let frame = Frame::new();
        let animation = Box::new(animation);
        let backend = Box::new(backend);

        Self {
            animation,
            backend,
            frame,
        }
    }

    pub fn reset(&mut self) {
        self.animation.reset();
    }

    pub fn step(&mut self) {
        if self.animation.maybe_ended() {
            self.animation.reset();
        }

        self.animation.next_frame(&mut self.frame);
        self.backend.display_frame(&self.frame);
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }
}
