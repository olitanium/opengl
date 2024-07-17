mod array_maths;
use self::array_maths::IsUnit;
use core::{
    array,
    ops::{Index, IndexMut},
};
use std::iter;

use super::matrix::Matrix;

#[derive(Copy, Clone, Debug)]
pub enum Inner<const N: usize> {
    Normal([f32; N]),
    NonNormal([f32; N]),
}

impl<const N: usize> Default for Inner<N> {
    #[inline]
    fn default() -> Self {
        Self::NonNormal([0.0; N])
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Vector<const N: usize>(Inner<N>);

impl<const N: usize> Vector<N> {
    #[must_use]
    #[inline]
    pub const fn new(input: [f32; N]) -> Self {
        Self(Inner::NonNormal(input))
    }

    #[must_use]
    #[inline]
    pub fn new_zero() -> Self {
        Self::new([0.0; N])
    }

    #[must_use]
    #[inline]
    pub fn new_checked(input: [f32; N]) -> Self {
        use array_maths::IsUnit as U;
        match array_maths::is_unit(&input) {
            U::True => Self(Inner::Normal(input)),
            U::False(_) => Self(Inner::NonNormal(input)),
        }
    }

    /// # Safety
    ///
    /// The vector must be have a magnitude of 1.0, or this may break future optimisations
    #[must_use]
    #[inline]
    pub const unsafe fn new_normal_unchecked(input: [f32; N]) -> Self {
        Self(Inner::Normal(input))
    }

    #[must_use]
    #[inline]
    pub const fn into_inner(self) -> [f32; N] {
        match self.0 {
            Inner::NonNormal(x) | Inner::Normal(x) => x,
        }
    }

    #[must_use]
    #[inline]
    pub const fn inner(&self) -> &[f32; N] {
        match &self.0 {
            Inner::NonNormal(x) | Inner::Normal(x) => x,
        }
    }

    #[must_use]
    #[inline]
    pub fn inner_mut(&mut self) -> &mut [f32; N] {
        match &mut self.0 {
            Inner::NonNormal(x) | Inner::Normal(x) => x,
        }
    }

    #[inline]
    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut f32> {
        self.inner_mut().iter_mut()
    }

    #[must_use]
    #[inline]
    pub fn map(self, op: impl Fn(f32) -> f32) -> Self {
        let array = self.into_inner().map(op);
        Self::new(array)
    }

    /// # Panics
    #[must_use]
    #[inline]
    pub fn normalize(self) -> Self {
        match &self.0 {
            Inner::Normal(_) => self,
            Inner::NonNormal(x) => {
                if array_maths::is_zero(x) {
                    panic!("Cannot normalize a Zero vector")
                } else {
                    match array_maths::is_unit(x) {
                        IsUnit::True => unsafe { Self::new_normal_unchecked(*x) },
                        IsUnit::False(magnitude) => {
                            let one_len = 1.0 / magnitude;
                            let inner = x.map(|x| x * one_len);
                            unsafe { Self::new_normal_unchecked(inner) }
                        }
                    }
                }
            }
        }
    }

    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        array_maths::is_zero(self.inner())
    }

    #[inline]
    #[must_use]
    pub fn is_unit(&self) -> IsUnit {
        array_maths::is_unit(self.inner())
    }

    #[inline]
    #[must_use]
    pub fn dot(&self, rhs: &Self) -> f32 {
        array_maths::dot(self.inner(), rhs.inner())
    }

    #[inline]
    #[must_use]
    pub fn from_to(from: Self, to: Self) -> Self {
        to - from
    }

    #[must_use]
    #[inline]
    pub fn distance_squared(self, to: Self) -> f32 {
        let direction = Self::from_to(self, to);
        direction.dot(&direction)
    }

    #[inline]
    #[must_use]
    pub fn scale(self, scalar: f32) -> Self {
        let scalar: f32 = scalar;
        self.map(|elem| scalar * elem)
    }

    #[inline]
    #[must_use]
    pub fn flip(mut self) -> Self {
        for x in self.inner_mut() {
            *x = -*x;
        }
        self
    }

    #[must_use]
    #[expect(clippy::missing_panics_doc)]
    #[inline]
    pub fn truncate<const M: usize>(self) -> Vector<M> {
        let x = self
            .into_inner()
            .into_iter()
            .chain(iter::repeat(0.0))
            .take(M)
            .collect::<Box<_>>();
        Vector::try_from(x.as_ref()).expect("This boxed_slice is guaranteed to be M long")
    }
}

impl Vector<3> {
    #[inline]
    #[must_use]
    pub fn cross(self, rhs: Self) -> Self {
        // Preserves Normalisation
        let out = [
            self[1].mul_add(rhs[2], -(self[2] * rhs[1])), // self[1] * rhs[2] - self[2] * rhs[1],
            self[2].mul_add(rhs[0], -(self[0] * rhs[2])), // self[2] * rhs[0] - self[0] * rhs[2],
            self[0].mul_add(rhs[1], -(self[1] * rhs[0])), // self[0] * rhs[1] - self[1] * rhs[0],
        ];
        Self::new_checked(out)
    }
}

macro_rules! impl_ops {
    ($trait:ident::$fn:ident) => {
        use self::array_maths::$fn;
        impl<const N: usize> core::ops::$trait for Vector<N> {
            type Output = Self;
            #[inline]
            fn $fn(self, rhs: Self) -> Self {
                Self::new($fn(self.into_inner(), rhs.into_inner()))
            }
        }
    };
}

impl_ops!(Add::add);
impl_ops!(Sub::sub);

impl<const R: usize> From<Matrix<R, 1>> for Vector<R> {
    #[inline]
    fn from(value: Matrix<R, 1>) -> Self {
        value.into_inner()[0]
    }
}

impl<const N: usize> From<[f32; N]> for Vector<N> {
    #[inline]
    fn from(value: [f32; N]) -> Self {
        Self::new(value)
    }
}

impl<const N: usize> From<Vector<N>> for [f32; N] {
    #[inline]
    fn from(value: Vector<N>) -> Self {
        value.into_inner()
    }
}

impl<'a, const N: usize> TryFrom<&'a [f32]> for Vector<N> {
    type Error = <[f32; N] as TryFrom<&'a [f32]>>::Error;

    #[inline]
    fn try_from(value: &[f32]) -> Result<Self, Self::Error> {
        value.try_into().map(Self::new)
    }
}

impl<const N: usize> AsRef<[f32; N]> for Vector<N> {
    #[inline]
    fn as_ref(&self) -> &[f32; N] {
        self.inner()
    }
}

impl<const N: usize> AsRef<[f32]> for Vector<N> {
    #[inline]
    fn as_ref(&self) -> &[f32] {
        self.inner()
    }
}

impl<const N: usize> AsRef<Self> for Vector<N> {
    #[inline]
    fn as_ref(&self) -> &Self {
        self
    }
}

impl<const N: usize> AsMut<[f32; N]> for Vector<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32; N] {
        self.inner_mut()
    }
}

impl<const N: usize> AsMut<[f32]> for Vector<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut [f32] {
        self.inner_mut()
    }
}

impl<const N: usize> AsMut<Self> for Vector<N> {
    #[inline]
    fn as_mut(&mut self) -> &mut Self {
        self
    }
}

impl<const N: usize> Index<usize> for Vector<N> {
    type Output = f32;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.inner().index(index)
    }
}

impl<const N: usize> IndexMut<usize> for Vector<N> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.inner_mut().index_mut(index)
    }
}

impl<const N: usize> IntoIterator for Vector<N> {
    type Item = f32;
    type IntoIter = array::IntoIter<Self::Item, N>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.into_inner().into_iter()
    }
}

impl<const N: usize> core::ops::Neg for Vector<N> {
    type Output = Self;
    #[inline]
    fn neg(self) -> Self::Output {
        self.flip()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UnitVector<const N: usize>(Vector<N>);

impl<const N: usize> UnitVector<N> {
    #[inline]
    fn new(input: Vector<N>) -> Self {
        Self(input.normalize())
    }

    #[inline]
    fn get(&self) -> Vector<N> {
        self.0
    }

    #[inline]
    pub fn set(&mut self, input: Vector<N>) {
        self.0 = input.normalize();
    }
}

impl<const N: usize> From<Vector<N>> for UnitVector<N> {
    #[inline]
    fn from(value: Vector<N>) -> Self {
        Self::new(value)
    }
}

impl<const N: usize> From<UnitVector<N>> for Vector<N> {
    #[inline]
    fn from(value: UnitVector<N>) -> Self {
        value.get()
    }
}

impl<const N: usize> AsRef<Vector<N>> for UnitVector<N> {
    #[inline]
    fn as_ref(&self) -> &Vector<N> {
        &self.0
    }
}
