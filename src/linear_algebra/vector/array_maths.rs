macro_rules! elem_wise {
    ($trait:ident::$fn:ident) => {
        #[inline]
        pub fn $fn<const N: usize>(lhs: [f32; N], rhs: [f32; N]) -> [f32; N] {
            let mut out = [0.0; N];
            for (dest, (v1i, v2i)) in core::iter::zip(&mut out, core::iter::zip(lhs, rhs)) {
                *dest = core::ops::$trait::$fn(v1i, v2i);
            }
            out
        }
    };
}

elem_wise!(Add::add);
elem_wise!(Sub::sub);
elem_wise!(Mul::mul);
//elem_wise!(Div::div);

#[inline]
pub fn dot<const N: usize>(v1: &[f32; N], v2: &[f32; N]) -> f32 {
    mul(*v1, *v2).into_iter().sum()
}

#[inline]
pub fn is_zero(v1: &[f32]) -> bool {
    v1.iter().all(|x| x.abs() <= f32::EPSILON)
}

pub enum IsUnit {
    True,
    False(f32),
}

pub fn is_unit<const N: usize>(x: &[f32; N]) -> IsUnit {
    let magnitude_sq = dot(x, x);
    if (magnitude_sq - 1.0).abs() <= f32::EPSILON {
        IsUnit::True
    } else {
        IsUnit::False(magnitude_sq.sqrt())
    }
}
