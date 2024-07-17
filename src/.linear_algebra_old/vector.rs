use crate::{error_fmt, EngineError};

#[derive(Clone, Copy, Debug)]
pub(crate) enum VectorEnum<const N: usize> {
    Normal([f32; N]),
    NonNormal([f32; N]),
}
#[derive(Clone, Copy, Debug, Default)]
pub struct Vector<const N: usize>(VectorEnum<N>);

impl<const N: usize> Default for VectorEnum<N> {
    fn default() -> Self {
        Self::NonNormal([0.0; N])
    }
}

macro_rules! elem_wise {
    ($trait:ident::$fn:ident) => {
        #[inline]
        fn $fn<const N: usize>(lhs: [f32; N], rhs: [f32; N]) -> [f32; N] {
            let mut out = [0.0; N];
            for (dest, (v1i, v2i)) in core::iter::zip(&mut out, core::iter::zip(lhs, rhs)) {
                *dest = core::ops::$trait::$fn(v1i, v2i);
            }
            out
        }
    };
}

macro_rules! impl_ops {
    ($trait:ident::$fn:ident) => {
        impl<const N: usize> core::ops::$trait for Vector<N> {
            type Output = Self;

            #[inline]
            fn $fn(self, rhs: Self) -> Self {
                Self(VectorEnum::NonNormal($fn(self.as_array(), rhs.as_array())))
            }
        }
    };
}

elem_wise!(Add::add);
elem_wise!(Sub::sub);
elem_wise!(Mul::mul);
//elem_wise!(Div::div);

impl_ops!(Add::add);
impl_ops!(Sub::sub);

#[inline]
fn dot<const N: usize>(v1: [f32; N], v2: [f32; N]) -> f32 {
    mul(v1, v2).into_iter().sum()
}

#[inline]
fn is_zero<const N: usize>(v1: [f32; N]) -> bool {
    v1.into_iter().all(|x| x.abs() <= f32::EPSILON)
}

pub enum IsUnit {
    True,
    False(f32),
}

fn is_unit<const N: usize>(x: [f32; N]) -> IsUnit {
    let magnitude_sq = dot(x, x);
    if (magnitude_sq - 1.0).abs() <= f32::EPSILON {
        IsUnit::True
    } else {
        IsUnit::False(magnitude_sq.sqrt())
    }
}

impl<const N: usize> Vector<N> {
    pub fn new(inner: [f32; N]) -> Self {
        if N == 0 {
            panic!("cannot create zero length Vector");
        }
        match is_unit(inner) {
            IsUnit::True => Self(VectorEnum::Normal(inner)),
            IsUnit::False(_) => Self(VectorEnum::NonNormal(inner)),
        }
    }

    pub fn new_from_slice(inner: &[f32]) -> crate::Result<Self> {
        let inner: [f32; N] = inner.try_into().map_err(|_| EngineError::VectorErr(error_fmt!(Vector<N>, "Slice length {} different to const Vector length {}", inner.len(), N)))?;
        Ok(Self::new(inner))
    }

    pub fn new_non_normal(inner: [f32; N]) -> Self {
        if N == 0 {
            panic!("cannot create zero length Vector");
        }
        Self(VectorEnum::NonNormal(inner))
    }

    pub fn new_normal_unchecked(inner: [f32; N]) -> Self {
        if N == 0 {
            panic!("cannot create zero length Vector");
        }
        Self(VectorEnum::Normal(inner))
    }

    pub fn as_slice(&self) -> &[f32] {
        match &self.0 {
            VectorEnum::Normal(x) => x,
            VectorEnum::NonNormal(x) => x,
        }
    }

    pub fn as_array(self) -> [f32; N] {
        match self.0 {
            VectorEnum::Normal(x) => x,
            VectorEnum::NonNormal(x) => x,
        }
    }

    fn map(self, func: impl FnMut(f32) -> f32) -> Self {
        match self.0 {
            VectorEnum::Normal(x) => Self(VectorEnum::NonNormal(x.map(func))),
            VectorEnum::NonNormal(x) => Self(VectorEnum::NonNormal(x.map(func))),
        }
    }

    pub fn is_zero(self) -> bool {
        is_zero(self.as_array())
    }

    pub fn is_unit(self) -> IsUnit {
        is_unit(self.as_array())
    }

    pub fn dot(self, rhs: Self) -> f32 {
        dot(self.as_array(), rhs.as_array())
    }

    pub fn normalize(self) -> Self {
        match self.0 {
            VectorEnum::Normal(_) => self,
            VectorEnum::NonNormal(x) => {
                if is_zero(x) {
                    panic!("Cannot normalize a Zero vector")
                } else {
                    match is_unit(x) {
                        IsUnit::True => Self(VectorEnum::Normal(x)),
                        IsUnit::False(magnitude) => {
                            let one_len = 1.0 / magnitude;
                            let inner = x.map(|x| x * one_len);
                            Self(VectorEnum::Normal(inner))
                        }
                    }
                }
            }
        }
    }

    #[inline]
    pub fn from_to(from: Self, to: Self) -> Self {
        from - to
    }

    pub fn distance_squared(self, to: Self) -> f32 {
        let direction = Self::from_to(self, to);
        direction.dot(direction)
    }

    #[inline]
    pub fn scale(self, scalar: impl Into<f32>) -> Self {
        let scalar: f32 = scalar.into();
        self.map(|elem| scalar * elem)
    }

    #[inline]
    pub fn flip(self) -> Self {
        // Preserves Normalisation
        match self.0 {
            VectorEnum::Normal(x) => Self(VectorEnum::Normal(x.map(|x| -x))),
            VectorEnum::NonNormal(x) => Self(VectorEnum::NonNormal(x.map(|x| -x))),
        }
    }
}

impl Vector<3> {
    #[inline]
    pub fn cross(lhs: Self, rhs: Self) -> Self {
        // Preserves Normalisation
        let v1 = lhs.as_array();
        let v2 = rhs.as_array();
        let out = [
            v1[1] * v2[2] - v1[2] * v2[1],
            v1[2] * v2[0] - v1[0] * v2[2],
            v1[0] * v2[1] - v1[1] * v2[0],
        ];
        Self::new(out)
    }
}
