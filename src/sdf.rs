use std::ops::{Div, Sub};

use num::One;
use palette::{Blend, LinSrgba};
use palette::Mix;
use sdfu::{
    ops::{HardMin, MinFunction},
    SDF,
};

use crate::frame::Frame;

pub fn render_sdf<V, S>(sdf: S, frame: &mut Frame)
where
    V: sdfu::mathtypes::Vec3<f32>,
    S: SDF<f32, V>,
{
    for (x, y, z, pix) in frame.pixels_mut() {
        let pos = V::new(x as f32, y as f32, z as f32);
        let dist = sdf.dist(pos);
        let mut colour = sdf.colour(pos);

        if dist > 0.2 {
            let alpha = 1.0 / (dist).powi(5);
            colour.alpha *= alpha;
        }

        *pix = colour;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct MultiUnion<'a, T, S, M> {
    inner: &'a [S],
    min_func: M,
    _pd: std::marker::PhantomData<T>,
}

impl<'a, T, S> MultiUnion<'a, T, S, HardMin<T>> {
    pub fn hard(sdfs: &'a [S]) -> Self {
        assert!(!sdfs.is_empty());

        MultiUnion {
            inner: sdfs,
            min_func: HardMin::default(),
            _pd: std::marker::PhantomData,
        }
    }
}

fn blend_amount<T>(a: T, b: T, m: T) -> T
where
    T: Copy + Sub<T, Output = T> + Div<T, Output = T> + PartialOrd + One,
{
    if a < b {
        (m - a) / (b - a)
    } else {
        T::one() - ((m - b) / (a - b))
    }
}

impl<'a, T, V, S, M> SDF<T, V> for MultiUnion<'a, T, S, M>
where
    T: Copy + Sub<T, Output = T> + Div<T, Output = T> + PartialOrd + One,
    f32: From<T>,
    V: sdfu::mathtypes::Vec<T>,
    S: SDF<T, V>,
    M: MinFunction<T> + Copy,
{
    #[inline]
    fn dist(&self, p: V) -> T {
        let mut sdfs = self.inner.iter();

        let first = sdfs
            .next()
            .expect("The SDFs in the union should be nonzero");

        let mut dist = first.dist(p);

        for sdf in sdfs {
            dist = self.min_func.min(dist, sdf.dist(p))
        }

        dist
    }

    #[inline]
    fn colour(&self, p: V) -> LinSrgba {
        let mut sdfs = self.inner.iter();

        let first = sdfs
            .next()
            .expect("The SDFs in the union should be nonzero");

        let mut colour = first.colour(p).into_premultiplied();
        let mut dist = first.dist(p);

        for sdf in sdfs {
            let b_colour = sdf.colour(p).into_premultiplied();
            let b_dist = sdf.dist(p);
            let m = self.min_func.min(dist, b_dist);
            let blend = blend_amount(dist, b_dist, m);

            dist = m;
            colour = colour.mix(&b_colour, f32::from(blend));
        }

        LinSrgba::from_premultiplied(colour)
    }
}
