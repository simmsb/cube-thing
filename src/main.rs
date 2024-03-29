#![allow(incomplete_features)]
#![feature(specialization)]
#![feature(vec_retain_mut)]
#![feature(mixed_integer_ops)]
#![feature(type_alias_impl_trait)]
#![cfg_attr(feature = "visual", feature(adt_const_params))]

mod animation;
mod animations;
mod frame;
mod render;
mod sdf;
mod backends;

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

    let backend = {
        {
            use backends::null::NullBackend;
            NullBackend
        }
    };

    let mut driver = crate::render::Driver::new(animation, backend);

    loop {
        driver.step();
    }
}
