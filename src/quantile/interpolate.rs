//! Interpolation strategies.
use num_traits::{Float, FromPrimitive, NumOps, ToPrimitive};

fn float_quantile_index<F: Float>(q: F, len: usize) -> F {
	q * (F::from(len - 1).unwrap())
}

/// Returns the fraction that the quantile is between the lower and higher indices.
///
/// This ranges from 0, where the quantile exactly corresponds the lower index,
/// to 1, where the quantile exactly corresponds to the higher index.
fn float_quantile_index_fraction<F: Float>(q: F, len: usize) -> F {
	float_quantile_index(q, len).fract()
}

/// Returns the index of the value on the lower side of the quantile.
pub(crate) fn lower_index<F: Float>(q: F, len: usize) -> usize {
	float_quantile_index(q, len).floor().to_usize().unwrap()
}

/// Returns the index of the value on the higher side of the quantile.
pub(crate) fn higher_index<F: Float>(q: F, len: usize) -> usize {
	float_quantile_index(q, len).ceil().to_usize().unwrap()
}

/// Used to provide an interpolation strategy to [`quantile_axis_mut`].
///
/// [`quantile_axis_mut`]: ../trait.QuantileExt.html#tymethod.quantile_axis_mut
pub trait Interpolate<T> {
	/// Returns `true` iff the lower value is needed to compute the
	/// interpolated value.
	#[doc(hidden)]
	fn needs_lower<F: Float>(q: F, len: usize) -> bool;

	/// Returns `true` iff the higher value is needed to compute the
	/// interpolated value.
	#[doc(hidden)]
	fn needs_higher<F: Float>(q: F, len: usize) -> bool;

	/// Computes the interpolated value.
	///
	/// **Panics** if `None` is provided for the lower value when it's needed
	/// or if `None` is provided for the higher value when it's needed.
	#[doc(hidden)]
	fn interpolate<F: Float>(lower: Option<T>, higher: Option<T>, q: F, len: usize) -> T;

	private_decl! {}
}

/// Select the higher value.
pub struct Higher;
/// Select the lower value.
pub struct Lower;
/// Select the nearest value.
pub struct Nearest;
/// Select the midpoint of the two values (`(lower + higher) / 2`).
pub struct Midpoint;
/// Linearly interpolate between the two values
/// (`lower + (higher - lower) * fraction`, where `fraction` is the
/// fractional part of the index surrounded by `lower` and `higher`).
pub struct Linear;

impl<T> Interpolate<T> for Higher {
	fn needs_lower<F: Float>(_q: F, _len: usize) -> bool {
		false
	}
	fn needs_higher<F: Float>(_q: F, _len: usize) -> bool {
		true
	}
	fn interpolate<F: Float>(_lower: Option<T>, higher: Option<T>, _q: F, _len: usize) -> T {
		higher.unwrap()
	}
	private_impl! {}
}

impl<T> Interpolate<T> for Lower {
	fn needs_lower<F: Float>(_q: F, _len: usize) -> bool {
		true
	}
	fn needs_higher<F: Float>(_q: F, _len: usize) -> bool {
		false
	}
	fn interpolate<F: Float>(lower: Option<T>, _higher: Option<T>, _q: F, _len: usize) -> T {
		lower.unwrap()
	}
	private_impl! {}
}

impl<T> Interpolate<T> for Nearest {
	fn needs_lower<F: Float>(q: F, len: usize) -> bool {
		float_quantile_index_fraction(q, len) < F::from(0.5).unwrap()
	}
	fn needs_higher<F: Float>(q: F, len: usize) -> bool {
		!<Self as Interpolate<T>>::needs_lower(q, len)
	}
	fn interpolate<F: Float>(lower: Option<T>, higher: Option<T>, q: F, len: usize) -> T {
		if <Self as Interpolate<T>>::needs_lower(q, len) {
			lower.unwrap()
		} else {
			higher.unwrap()
		}
	}
	private_impl! {}
}

impl<T> Interpolate<T> for Midpoint
where
	T: NumOps + Clone + FromPrimitive,
{
	fn needs_lower<F: Float>(_q: F, _len: usize) -> bool {
		true
	}
	fn needs_higher<F: Float>(_q: F, _len: usize) -> bool {
		true
	}
	fn interpolate<F: Float>(lower: Option<T>, higher: Option<T>, _q: F, _len: usize) -> T {
		let denom = T::from_u8(2).unwrap();
		let lower = lower.unwrap();
		let higher = higher.unwrap();
		lower.clone() + (higher - lower) / denom
	}
	private_impl! {}
}

impl<T> Interpolate<T> for Linear
where
	T: NumOps + Clone + FromPrimitive + ToPrimitive,
{
	fn needs_lower<F: Float>(_q: F, _len: usize) -> bool {
		true
	}
	fn needs_higher<F: Float>(_q: F, _len: usize) -> bool {
		true
	}
	fn interpolate<F: Float>(lower: Option<T>, higher: Option<T>, q: F, len: usize) -> T {
		let fraction = float_quantile_index_fraction(q, len).to_f64().unwrap();
		let lower = lower.unwrap();
		let higher = higher.unwrap();
		let lower_f64 = lower.to_f64().unwrap();
		let higher_f64 = higher.to_f64().unwrap();
		lower + T::from_f64(fraction * (higher_f64 - lower_f64)).unwrap()
	}
	private_impl! {}
}
