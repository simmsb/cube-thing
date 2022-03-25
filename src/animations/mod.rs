use std::time::Duration;

use crate::animation::{MaybeTerminatingAnimation, Animation};

pub mod bounce;
pub mod lines;
pub mod sine_thing;
pub mod waves;
mod utils;

pub fn current_config() -> impl MaybeTerminatingAnimation {
    let anim = bounce::Bounce::default()
        .with_fps(60.0)
        .with_duration(Duration::from_secs(60))
        .repeat(30)
        // .with_duration(Duration::from_secs(5))
        // .repeat(2)
        .chain(
            lines::SpinningLines::default()
                .with_fps(60.0)
                .with_duration(Duration::from_secs(60 * 30)),
                // .with_duration(Duration::from_secs(10)),
        )
        .chain(
            sine_thing::SineThing::default()
                .with_fps(60.0)
                .with_duration(Duration::from_secs(60 * 30)),
                // .with_duration(Duration::from_secs(10)),
        );

    let anim = waves::Waves::default()
        .with_fps(60.0);

    println!("{:#?}", anim);

    anim
}
