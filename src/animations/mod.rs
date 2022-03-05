use std::time::Duration;

use crate::render::Animation;

pub mod lines;
pub mod sine_thing;

pub fn current_config() -> impl Animation {
    lines::SpinningLines::default()
        .with_fps(60.0)
        .with_duration(Duration::from_secs(60 * 30))
        .chain(
            sine_thing::SineThing::default()
                .with_fps(60.0)
                .with_duration(Duration::from_secs(60 * 30)),
        )
}
