use std::time::Duration;

use crate::animation::{MaybeTerminatingAnimation, Animation};

pub mod bounce;
pub mod lines;
pub mod sine_thing;
pub mod waves;
mod utils;

#[cfg(feature = "visual")]
type Anim = impl MaybeTerminatingAnimation + bevy_inspector_egui::Inspectable;
#[cfg(not(feature = "visual"))]
type Anim = impl MaybeTerminatingAnimation;

pub fn current_config() -> Anim {
    let anim = bounce::Bounce::default()
        .with_fps(60.0)
        .with_duration(Duration::from_secs(60))
        .repeat(30)
        // .with_duration(Duration::from_secs(5))
        // .repeat(2)
        .chain(
            waves::Waves::default()
               .with_fps(60.0)
                .with_duration(Duration::from_secs(60 * 30))
        )
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

    println!("{:#?}", anim);

    anim
}
