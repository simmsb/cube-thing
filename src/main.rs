#![allow(incomplete_features)]
#![feature(specialization)]
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
        #[cfg(feature = "rpi_out")]
        {
            use backends::rpi::RpiBackend;
            RpiBackend::new().unwrap()
        }
        #[cfg(not(feature = "rpi_out"))]
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
