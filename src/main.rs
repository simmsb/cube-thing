#![feature(specialization)]

mod animations;
mod render;
mod sdf;

#[cfg(feature = "visual")]
mod visual;

#[cfg(feature = "visual")]
fn main() {
    visual::main();
}

#[cfg(not(feature = "visual"))]
fn main() {
    use crate::animations::current_config;

    let animation = current_config();

    let mut driver = crate::render::Driver::new(animation);

    loop {
        driver.step();
    }
}
