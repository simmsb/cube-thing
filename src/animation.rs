use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use crate::frame::Frame;

pub trait Animation {
    fn next_frame(&mut self, frame: &mut Frame);

    fn reset(&mut self);

    fn with_fps(self, fps: f32) -> FixedFPSAnimation<Self>
    where
        Self: Sized,
    {
        FixedFPSAnimation {
            inner: self,
            interval: Duration::from_secs_f32(1.0 / fps),
            last_frame: Instant::now(),
        }
    }

    fn with_duration(self, duration: Duration) -> TimeLimitedAnimation<Self>
    where
        Self: Sized,
    {
        TimeLimitedAnimation {
            inner: self,
            duration,
            started: Instant::now(),
        }
    }

    fn chain<U: Animation>(self, other: U) -> ChainedAnimation<Self, U>
    where
        Self: Sized,
    {
        ChainedAnimation {
            a: self,
            b: other,
            current: false,
        }
    }

    fn repeat(self, times: usize) -> RepeatedAnimation<Self>
    where
        Self: Sized,
    {
        RepeatedAnimation {
            inner: self,
            loops: times,
            count: 0,
        }
    }
}

pub trait TerminatingAnimation: Animation {
    fn ended(&self) -> bool;
}

pub trait MaybeTerminatingAnimation: Animation {
    fn maybe_ended(&self) -> bool;
}

impl<T: Animation> MaybeTerminatingAnimation for T {
    default fn maybe_ended(&self) -> bool {
        false
    }
}

macro_rules! impl_maybe_term {
    ( $( $name:ident $(< $( $lt:tt $( : $clt:tt $(+ $dlt:tt )* )? ),+ >)? ),+ ) => {
        $(
            impl $(< $( $lt $( : $clt $(+ $dlt )* )? ),+ >)?
                MaybeTerminatingAnimation
            for $name
                $(< $( $lt ),+ >)?
            {
                fn maybe_ended(&self) -> bool {
                    self.ended()
                }
            }
        )+
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "visual", derive(bevy_inspector_egui::Inspectable))]
pub struct FixedFPSAnimation<T> {
    inner: T,
    #[cfg_attr(feature = "visual", inspectable(ignore))]
    interval: Duration,
    #[cfg_attr(feature = "visual", inspectable(ignore))]
    last_frame: Instant,
}

impl<T: Animation> Animation for FixedFPSAnimation<T> {
    fn next_frame(&mut self, frame: &mut Frame) {
        let now = Instant::now();
        if now.duration_since(self.last_frame) > self.interval {
            self.inner.next_frame(frame);
            self.last_frame = now;
        }
    }

    fn reset(&mut self) {
        self.inner.reset();
    }
}

impl<T: TerminatingAnimation> TerminatingAnimation for FixedFPSAnimation<T> {
    fn ended(&self) -> bool {
        self.inner.ended()
    }
}

impl_maybe_term!(FixedFPSAnimation<T: TerminatingAnimation>);

#[derive(Debug)]
#[cfg_attr(feature = "visual", derive(bevy_inspector_egui::Inspectable))]
pub struct TimeLimitedAnimation<T> {
    inner: T,
    #[cfg_attr(feature = "visual", inspectable(ignore))]
    duration: Duration,
    #[cfg_attr(feature = "visual", inspectable(ignore))]
    started: Instant,
}

impl<T: Animation> Animation for TimeLimitedAnimation<T> {
    fn next_frame(&mut self, frame: &mut Frame) {
        self.inner.next_frame(frame);
    }

    fn reset(&mut self) {
        self.inner.reset();
        self.started = Instant::now();
    }
}

impl<T: MaybeTerminatingAnimation> TerminatingAnimation for TimeLimitedAnimation<T> {
    fn ended(&self) -> bool {
        if Instant::now().duration_since(self.started) > self.duration {
            dbg!("resetting time animation", std::any::type_name::<T>());
            return true;
        }

        self.inner.maybe_ended()
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "visual", derive(bevy_inspector_egui::Inspectable))]
pub struct ChainedAnimation<T, U> {
    a: T,
    b: U,
    current: bool,
}

impl<T: TerminatingAnimation, U: Animation> Animation for ChainedAnimation<T, U> {
    fn next_frame(&mut self, frame: &mut Frame) {
        if !self.current {
            if self.a.ended() {
                dbg!(
                    "switching chain animation",
                    std::any::type_name::<T>(),
                    std::any::type_name::<U>()
                );
                self.current = !self.current;
                self.b.reset();
            } else {
                self.a.next_frame(frame)
            }
        }

        if self.current {
            self.b.next_frame(frame)
        }
    }

    fn reset(&mut self) {
        self.a.reset();
        self.current = false;
    }
}

// chained animations only terminate if the final animation terminates
impl<T: TerminatingAnimation, U: TerminatingAnimation> TerminatingAnimation
    for ChainedAnimation<T, U>
{
    fn ended(&self) -> bool {
        self.current && self.b.ended()
    }
}

impl_maybe_term!(ChainedAnimation<T: TerminatingAnimation, U: TerminatingAnimation>);

#[derive(Debug)]
#[cfg_attr(feature = "visual", derive(bevy_inspector_egui::Inspectable))]
pub struct RepeatedAnimation<T> {
    inner: T,
    loops: usize,
    count: usize,
}

impl<T: TerminatingAnimation> Animation for RepeatedAnimation<T> {
    fn next_frame(&mut self, frame: &mut Frame) {
        if self.inner.ended() && self.count < self.loops {
            self.inner.reset();
            self.count += 1;
            dbg!(
                "repeating repeat animation",
                std::any::type_name::<T>(),
                self.count
            );
        }

        self.inner.next_frame(frame);
    }

    fn reset(&mut self) {
        self.inner.reset();
        self.count = 0;
    }
}

impl<T: TerminatingAnimation> TerminatingAnimation for RepeatedAnimation<T> {
    fn ended(&self) -> bool {
        self.count >= self.loops && self.inner.ended()
    }
}

impl_maybe_term!(RepeatedAnimation<T: TerminatingAnimation>);

impl<T: Animation> Animation for RwLock<T> {
    fn next_frame(&mut self, frame: &mut Frame) {
        self.write().unwrap().next_frame(frame);
    }

    fn reset(&mut self) {
        self.write().unwrap().reset();
    }
}

impl<T: TerminatingAnimation> TerminatingAnimation for RwLock<T> {
    fn ended(&self) -> bool {
        self.read().unwrap().ended()
    }
}

impl<T: Animation> Animation for Arc<RwLock<T>> {
    fn next_frame(&mut self, frame: &mut Frame) {
        self.write().unwrap().next_frame(frame);
    }

    fn reset(&mut self) {
        self.write().unwrap().reset();
    }
}

impl<T: TerminatingAnimation> TerminatingAnimation for Arc<RwLock<T>> {
    fn ended(&self) -> bool {
        self.read().unwrap().ended()
    }
}
