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

        if dist < 0.2 {
            *pix = 255;
        } else {
            *pix = (255.0 / (dist * 2.0).powi(5)).clamp(0.0, 255.0) as u8;
        }
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

impl<'a, T, V, S, M> SDF<T, V> for MultiUnion<'a, T, S, M>
where
    T: Copy,
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
}
