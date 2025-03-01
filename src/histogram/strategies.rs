//! Strategies used by [`GridBuilder`] to infer optimal parameters from data for building [`Bins`]
//! and [`Grid`] instances.
//!
//! The docs for each strategy have been taken almost verbatim from [`NumPy`].
//!
//! Each strategy specifies how to compute the optimal number of [`Bins`] or the optimal bin width.
//! For those strategies that prescribe the optimal number of [`Bins`], the optimal bin width is
//! computed by `bin_width = (max - min)/n`.
//!
//! Since all bins are left-closed and right-open, it is guaranteed to add an extra bin to include
//! the maximum value from the given data when necessary, so that no data is discarded.
//!
//! # Strategies
//!
//! Currently, the following strategies are implemented:
//!
//! - [`Auto`]: Maximum of the [`Sturges`] and [`FreedmanDiaconis`] strategies. Provides good all
//!   around performance.
//! - [`FreedmanDiaconis`]: Robust (resilient to outliers) strategy that takes into account data
//!   variability and data size.
//! - [`Rice`]: A strategy that does not take variability into account, only data size. Commonly
//!   overestimates number of bins required.
//! - [`Sqrt`]: Square root (of data size) strategy, used by Excel and other programs
//!   for its speed and simplicity.
//! - [`Sturges`]: R’s default strategy, only accounts for data size. Only optimal for gaussian data
//!   and underestimates number of bins for large non-gaussian datasets.
//!
//! # Notes
//!
//! In general, successful inference on optimal bin width and number of bins relies on
//! **variability** of data. In other word, the provided observations should not be empty or
//! constant.
//!
//! In addition, [`Auto`] and [`FreedmanDiaconis`] requires the [`interquartile range (IQR)`][iqr],
//! i.e. the difference between upper and lower quartiles, to be positive.
//!
//! [`GridBuilder`]: ../struct.GridBuilder.html
//! [`Bins`]: ../struct.Bins.html
//! [`Grid`]: ../struct.Grid.html
//! [`NumPy`]: https://docs.scipy.org/doc/numpy/reference/generated/numpy.histogram_bin_edges.html#numpy.histogram_bin_edges
//! [`Auto`]: struct.Auto.html
//! [`Sturges`]: struct.Sturges.html
//! [`FreedmanDiaconis`]: struct.FreedmanDiaconis.html
//! [`Rice`]: struct.Rice.html
//! [`Sqrt`]: struct.Sqrt.html
//! [iqr]: https://www.wikiwand.com/en/Interquartile_range

use crate::{
	histogram::{Bins, Edges, errors::BinsBuildError},
	quantile::{Quantile1dExt, QuantileExt, interpolate::Nearest},
};
use ndarray::{Data, prelude::*};
use num_traits::{FromPrimitive, NumOps, ToPrimitive, Zero};

/// A trait implemented by all strategies to build [`Bins`] with parameters inferred from
/// observations.
///
/// This is required by [`GridBuilder`] to know how to build a [`Grid`]'s projections on the
/// coordinate axes.
///
/// [`Bins`]: ../struct.Bins.html
/// [`GridBuilder`]: ../struct.GridBuilder.html
/// [`Grid`]: ../struct.Grid.html
pub trait BinsBuildingStrategy {
	#[allow(missing_docs)]
	type Elem: Ord + Send;
	/// Returns a strategy that has learnt the required parameter for building [`Bins`] for given
	/// 1-dimensional array, or an `Err` if it is not possible to infer the required parameter
	/// with the given data and specified strategy.
	///
	/// Calls [`Self::from_array_with_max`] with `max_n_bins` of [`u16::MAX`].
	///
	/// # Errors
	///
	/// See each of the `struct`-level documentation for details on errors an implementation may
	/// return.
	///
	/// [`Bins`]: ../struct.Bins.html
	fn from_array<S>(array: &ArrayBase<S, Ix1>) -> Result<Self, BinsBuildError>
	where
		S: Data<Elem = Self::Elem>,
		Self: std::marker::Sized,
	{
		Self::from_array_with_max(array, u16::MAX.into())
	}

	/// Returns a strategy that has learnt the required parameter for building [`Bins`] for given
	/// 1-dimensional array, or an `Err` if it is not possible to infer the required parameter
	/// with the given data and specified strategy.
	///
	/// # Errors
	///
	/// See each of the `struct`-level documentation for details on errors an implementation may
	/// return. Fails if the strategy requires more bins than `max_n_bins`.
	///
	/// [`Bins`]: ../struct.Bins.html
	fn from_array_with_max<S>(
		array: &ArrayBase<S, Ix1>,
		max_n_bins: usize,
	) -> Result<Self, BinsBuildError>
	where
		S: Data<Elem = Self::Elem>,
		Self: std::marker::Sized;

	/// Returns a [`Bins`] instance, according to parameters inferred from observations.
	///
	/// [`Bins`]: ../struct.Bins.html
	fn build(&self) -> Bins<Self::Elem>;

	/// Returns the optimal number of bins, according to parameters inferred from observations.
	fn n_bins(&self) -> usize;
}

#[derive(Debug)]
struct EquiSpaced<T> {
	bin_width: T,
	min: T,
	max: T,
}

/// Square root (of data size) strategy, used by Excel and other programs for its speed and
/// simplicity.
///
/// Let `n` be the number of observations. Then
///
/// `n_bins` = `sqrt(n)`
///
/// # Notes
///
/// This strategy requires the data
///
/// - not being empty
/// - not being constant
#[derive(Debug)]
pub struct Sqrt<T> {
	builder: EquiSpaced<T>,
}

/// A strategy that does not take variability into account, only data size. Commonly
/// overestimates number of bins required.
///
/// Let `n` be the number of observations and `n_bins` be the number of bins.
///
/// `n_bins` = 2`n`<sup>1/3</sup>
///
/// `n_bins` is only proportional to cube root of `n`. It tends to overestimate
/// the `n_bins` and it does not take into account data variability.
///
/// # Notes
///
/// This strategy requires the data
///
/// - not being empty
/// - not being constant
#[derive(Debug)]
pub struct Rice<T> {
	builder: EquiSpaced<T>,
}

/// R’s default strategy, only accounts for data size. Only optimal for gaussian data and
/// underestimates number of bins for large non-gaussian datasets.
///
/// Let `n` be the number of observations.
/// The number of bins is 1 plus the base 2 log of `n`. This estimator assumes normality of data and
/// is too conservative for larger, non-normal datasets.
///
/// This is the default method in R’s hist method.
///
/// # Notes
///
/// This strategy requires the data
///
/// - not being empty
/// - not being constant
#[derive(Debug)]
pub struct Sturges<T> {
	builder: EquiSpaced<T>,
}

/// Robust (resilient to outliers) strategy that takes into account data variability and data size.
///
/// Let `n` be the number of observations and `at = 1 / 4`.
///
/// `bin_width` = `IQR` × `n`<sup>−1/3</sup> / (1 - 2 × `at`)
///
/// The bin width is proportional to the interquartile range ([`IQR`]) from `at` to `1 - at` and
/// inversely proportional to cube root of `n`. It can be too conservative for small datasets, but
/// it is quite good for large datasets. In case the [`IQR`] is close to zero, `at` is halved and an
/// improper [`IQR`] is computed. This is repeated as long as `at >= 1 / 512`. If no [`IQR`] is
/// found by then, Scott's rule is used as asymptotic resort which is based on the standard
/// deviation (SD). If the SD is close to zero as well, this strategy fails with
/// [`BinsBuildError::Strategy`]. As there is no one-fit-all epsilon, whether the IQR or standard
/// deviation is close to zero is indirectly tested by requiring the computed number of bins to not
/// exceed `max_n_bins` with a default of [`u16::MAX`].
///
/// The [`IQR`] is very robust to outliers.
///
/// # Notes
///
/// This strategy requires the data
///
/// - not being empty
/// - not being constant
/// - having positive [`IQR`]
///
/// [`IQR`]: https://en.wikipedia.org/wiki/Interquartile_range
#[derive(Debug)]
pub struct FreedmanDiaconis<T> {
	builder: EquiSpaced<T>,
}

#[derive(Debug)]
enum SturgesOrFD<T> {
	Sturges(Sturges<T>),
	FreedmanDiaconis(FreedmanDiaconis<T>),
}

/// Maximum of the [`Sturges`] and [`FreedmanDiaconis`] strategies. Provides good all around
/// performance.
///
/// A compromise to get a good value. For small datasets the [`Sturges`] value will usually be
/// chosen, while larger datasets will usually default to [`FreedmanDiaconis`]. Avoids the overly
/// conservative behaviour of [`FreedmanDiaconis`] and [`Sturges`] for small and large datasets
/// respectively.
///
/// # Notes
///
/// This strategy requires the data
///
/// - not being empty
/// - not being constant
/// - having positive [`IQR`]
///
/// [`Sturges`]: struct.Sturges.html
/// [`FreedmanDiaconis`]: struct.FreedmanDiaconis.html
/// [`IQR`]: https://en.wikipedia.org/wiki/Interquartile_range
#[derive(Debug)]
pub struct Auto<T> {
	builder: SturgesOrFD<T>,
}

impl<T> EquiSpaced<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	/// Returns `Err(BinsBuildError::Strategy)` if `bin_width<=0` or `min` >= `max`.
	/// Returns `Ok(Self)` otherwise.
	fn new(bin_width: T, min: T, max: T) -> Result<Self, BinsBuildError> {
		if (bin_width <= T::zero()) || (min >= max) {
			Err(BinsBuildError::Strategy)
		} else {
			Ok(Self {
				bin_width,
				min,
				max,
			})
		}
	}

	fn build(&self) -> Bins<T> {
		let n_bins = self.n_bins();
		let mut edges: Vec<T> = vec![];
		for i in 0..=n_bins {
			let edge = self.min.clone() + T::from_usize(i).unwrap() * self.bin_width.clone();
			edges.push(edge);
		}
		Bins::new(Edges::from(edges))
	}

	fn n_bins(&self) -> usize {
		let min = self.min.to_f64().unwrap();
		let max = self.max.to_f64().unwrap();
		let bin_width = self.bin_width.to_f64().unwrap();
		usize::from_f64(((max - min) / bin_width + 0.5).ceil()).unwrap_or(usize::MAX)
	}

	fn bin_width(&self) -> T {
		self.bin_width.clone()
	}
}

impl<T> BinsBuildingStrategy for Sqrt<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	type Elem = T;

	/// Returns `Err(BinsBuildError::Strategy)` if the array is constant.
	/// Returns `Err(BinsBuildError::EmptyInput)` if `a.len()==0`.
	/// Returns `Ok(Self)` otherwise.
	fn from_array_with_max<S>(
		a: &ArrayBase<S, Ix1>,
		max_n_bins: usize,
	) -> Result<Self, BinsBuildError>
	where
		S: Data<Elem = Self::Elem>,
	{
		let n_elems = a.len();
		// casting `n_elems: usize` to `f64` may casus off-by-one error here if `n_elems` > 2 ^ 53,
		// but it's not relevant here
		#[allow(clippy::cast_precision_loss)]
		// casting the rounded square root from `f64` to `usize` is safe
		#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
		let n_bins = (n_elems as f64).sqrt().round() as usize;
		let min = a.min()?;
		let max = a.max()?;
		let bin_width = compute_bin_width(min.clone(), max.clone(), n_bins);
		let builder = EquiSpaced::new(bin_width, min.clone(), max.clone())?;
		if builder.n_bins() > max_n_bins {
			Err(BinsBuildError::Strategy)
		} else {
			Ok(Self { builder })
		}
	}

	fn build(&self) -> Bins<T> {
		self.builder.build()
	}

	fn n_bins(&self) -> usize {
		self.builder.n_bins()
	}
}

impl<T> Sqrt<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	/// The bin width (or bin length) according to the fitted strategy.
	pub fn bin_width(&self) -> T {
		self.builder.bin_width()
	}
}

impl<T> BinsBuildingStrategy for Rice<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	type Elem = T;

	/// Returns `Err(BinsBuildError::Strategy)` if the array is constant.
	/// Returns `Err(BinsBuildError::EmptyInput)` if `a.len()==0`.
	/// Returns `Ok(Self)` otherwise.
	fn from_array_with_max<S>(
		a: &ArrayBase<S, Ix1>,
		max_n_bins: usize,
	) -> Result<Self, BinsBuildError>
	where
		S: Data<Elem = Self::Elem>,
	{
		let n_elems = a.len();
		// casting `n_elems: usize` to `f64` may casus off-by-one error here if `n_elems` > 2 ^ 53,
		// but it's not relevant here
		#[allow(clippy::cast_precision_loss)]
		// casting the rounded cube root from `f64` to `usize` is safe
		#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
		let n_bins = (2. * (n_elems as f64).powf(1. / 3.)).round() as usize;
		let min = a.min()?;
		let max = a.max()?;
		let bin_width = compute_bin_width(min.clone(), max.clone(), n_bins);
		let builder = EquiSpaced::new(bin_width, min.clone(), max.clone())?;
		if builder.n_bins() > max_n_bins {
			Err(BinsBuildError::Strategy)
		} else {
			Ok(Self { builder })
		}
	}

	fn build(&self) -> Bins<T> {
		self.builder.build()
	}

	fn n_bins(&self) -> usize {
		self.builder.n_bins()
	}
}

impl<T> Rice<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	/// The bin width (or bin length) according to the fitted strategy.
	pub fn bin_width(&self) -> T {
		self.builder.bin_width()
	}
}

impl<T> BinsBuildingStrategy for Sturges<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	type Elem = T;

	/// Returns `Err(BinsBuildError::Strategy)` if the array is constant.
	/// Returns `Err(BinsBuildError::EmptyInput)` if `a.len()==0`.
	/// Returns `Ok(Self)` otherwise.
	fn from_array_with_max<S>(
		a: &ArrayBase<S, Ix1>,
		max_n_bins: usize,
	) -> Result<Self, BinsBuildError>
	where
		S: Data<Elem = Self::Elem>,
	{
		let n_elems = a.len();
		// casting `n_elems: usize` to `f64` may casus off-by-one error here if `n_elems` > 2 ^ 53,
		// but it's not relevant here
		#[allow(clippy::cast_precision_loss)]
		// casting the rounded base-2 log from `f64` to `usize` is safe
		#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
		let n_bins = (n_elems as f64).log2().round() as usize + 1;
		let min = a.min()?;
		let max = a.max()?;
		let bin_width = compute_bin_width(min.clone(), max.clone(), n_bins);
		let builder = EquiSpaced::new(bin_width, min.clone(), max.clone())?;
		if builder.n_bins() > max_n_bins {
			Err(BinsBuildError::Strategy)
		} else {
			Ok(Self { builder })
		}
	}

	fn build(&self) -> Bins<T> {
		self.builder.build()
	}

	fn n_bins(&self) -> usize {
		self.builder.n_bins()
	}
}

impl<T> Sturges<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	/// The bin width (or bin length) according to the fitted strategy.
	pub fn bin_width(&self) -> T {
		self.builder.bin_width()
	}
}

impl<T> BinsBuildingStrategy for FreedmanDiaconis<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	type Elem = T;

	/// Returns `Err(BinsBuildError::Strategy)` if improper IQR and SD are close to zero.
	/// Returns `Err(BinsBuildError::EmptyInput)` if `a.len()==0`.
	/// Returns `Ok(Self)` otherwise.
	fn from_array<S>(a: &ArrayBase<S, Ix1>) -> Result<Self, BinsBuildError>
	where
		S: Data<Elem = Self::Elem>,
	{
		Self::from_array_with_max(a, u16::MAX.into())
	}

	/// Returns `Err(BinsBuildError::Strategy)` if improper IQR and SD are close to zero.
	/// Returns `Err(BinsBuildError::EmptyInput)` if `a.len()==0`.
	/// Returns `Ok(Self)` otherwise.
	fn from_array_with_max<S>(
		a: &ArrayBase<S, Ix1>,
		max_n_bins: usize,
	) -> Result<Self, BinsBuildError>
	where
		S: Data<Elem = Self::Elem>,
	{
		let n_points = a.len();
		if n_points == 0 {
			return Err(BinsBuildError::EmptyInput);
		}

		let n_cbrt = f64::from_usize(n_points).unwrap().powf(1. / 3.);
		let min = a.min()?;
		let max = a.max()?;
		let mut a_copy = a.to_owned();
		// As there is no one-fit-all epsilon to decide whether IQR is zero, translate it into
		// number of bins and compare it against `max_n_bins`. More bins than `max_n_bins` is a hint
		// for an IQR close to zero. If so, deviate from proper Freedman-Diaconis rule by widening
		// percentiles range and try again with `at` of 1/8, 1/16, 1/32, 1/64, 1/128, 1/256, 1/512.
		let mut at = 0.5;
		while at >= 1. / 512. {
			at *= 0.5;
			let first_quartile = a_copy.quantile_mut(at, &Nearest).unwrap();
			let third_quartile = a_copy.quantile_mut(1. - at, &Nearest).unwrap();
			let iqr = third_quartile - first_quartile;
			let denom = T::from_f64((1. - 2. * at) * n_cbrt).unwrap();
			if denom == T::zero() {
				continue;
			}
			let bin_width = iqr.clone() / denom;
			let builder = EquiSpaced::new(bin_width, min.clone(), max.clone())?;
			if builder.n_bins() > max_n_bins {
				continue;
			}
			return Ok(Self { builder });
		}
		// If the improper IQR is still close to zero, use Scott's rule as asymptotic resort before
		// giving up where `m` is the mean and `s` its SD.
		let m = a.iter().cloned().fold(T::zero(), |s, v| s + v) / T::from_usize(n_points).unwrap();
		let s = a
			.iter()
			.cloned()
			.map(|v| (v.clone() - m.clone()) * (v - m.clone()))
			.fold(T::zero(), |s, v| s + v);
		let s = (s / T::from_usize(n_points - 1).unwrap())
			.to_f64()
			.unwrap()
			.sqrt();
		let bin_width = T::from_f64(3.49 * s).unwrap() / T::from_f64(n_cbrt).unwrap();
		let builder = EquiSpaced::new(bin_width, min.clone(), max.clone())?;
		if builder.n_bins() > max_n_bins {
			return Err(BinsBuildError::Strategy);
		}
		Ok(Self { builder })
	}

	fn build(&self) -> Bins<T> {
		self.builder.build()
	}

	fn n_bins(&self) -> usize {
		self.builder.n_bins()
	}
}

impl<T> FreedmanDiaconis<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	/// The bin width (or bin length) according to the fitted strategy.
	pub fn bin_width(&self) -> T {
		self.builder.bin_width()
	}
}

impl<T> BinsBuildingStrategy for Auto<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	type Elem = T;

	/// Returns `Err(BinsBuildError::Strategy)` if `IQR==0`.
	/// Returns `Err(BinsBuildError::EmptyInput)` if `a.len()==0`.
	/// Returns `Ok(Self)` otherwise.
	fn from_array_with_max<S>(
		a: &ArrayBase<S, Ix1>,
		max_n_bins: usize,
	) -> Result<Self, BinsBuildError>
	where
		S: Data<Elem = Self::Elem>,
	{
		let fd_builder = FreedmanDiaconis::from_array_with_max(a, max_n_bins);
		let sturges_builder = Sturges::from_array_with_max(a, max_n_bins);
		match (fd_builder, sturges_builder) {
			(Err(_), Ok(sturges_builder)) => {
				let builder = SturgesOrFD::Sturges(sturges_builder);
				Ok(Self { builder })
			}
			(Ok(fd_builder), Err(_)) => {
				let builder = SturgesOrFD::FreedmanDiaconis(fd_builder);
				Ok(Self { builder })
			}
			(Ok(fd_builder), Ok(sturges_builder)) => {
				let builder = if fd_builder.bin_width() > sturges_builder.bin_width() {
					SturgesOrFD::Sturges(sturges_builder)
				} else {
					SturgesOrFD::FreedmanDiaconis(fd_builder)
				};
				Ok(Self { builder })
			}
			(Err(err), Err(_)) => Err(err),
		}
	}

	fn build(&self) -> Bins<T> {
		// Ugly
		match &self.builder {
			SturgesOrFD::FreedmanDiaconis(b) => b.build(),
			SturgesOrFD::Sturges(b) => b.build(),
		}
	}

	fn n_bins(&self) -> usize {
		// Ugly
		match &self.builder {
			SturgesOrFD::FreedmanDiaconis(b) => b.n_bins(),
			SturgesOrFD::Sturges(b) => b.n_bins(),
		}
	}
}

impl<T> Auto<T>
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	/// The bin width (or bin length) according to the fitted strategy.
	pub fn bin_width(&self) -> T {
		// Ugly
		match &self.builder {
			SturgesOrFD::FreedmanDiaconis(b) => b.bin_width(),
			SturgesOrFD::Sturges(b) => b.bin_width(),
		}
	}
}

/// Returns the `bin_width`, given the two end points of a range (`max`, `min`), and the number of
/// bins, consuming endpoints
///
/// `bin_width = (max - min)/n`
///
/// **Panics** if `n_bins == 0` and division by 0 panics for `T`.
fn compute_bin_width<T>(min: T, max: T, n_bins: usize) -> T
where
	T: Ord + Send + Clone + FromPrimitive + ToPrimitive + NumOps + Zero,
{
	let range = max - min;
	range / T::from_usize(n_bins).unwrap()
}

#[cfg(test)]
mod equispaced_tests {
	use super::EquiSpaced;

	#[test]
	fn bin_width_has_to_be_positive() {
		assert!(EquiSpaced::new(0, 0, 200).is_err());
	}

	#[test]
	fn min_has_to_be_strictly_smaller_than_max() {
		assert!(EquiSpaced::new(10, 0, 0).is_err());
	}
}

#[cfg(test)]
mod sqrt_tests {
	use super::{BinsBuildingStrategy, Sqrt};
	use ndarray::array;

	#[test]
	fn constant_array_are_bad() {
		assert!(
			Sqrt::from_array(&array![1, 1, 1, 1, 1, 1, 1])
				.unwrap_err()
				.is_strategy()
		);
	}

	#[test]
	fn empty_arrays_are_bad() {
		assert!(
			Sqrt::<usize>::from_array(&array![])
				.unwrap_err()
				.is_empty_input()
		);
	}
}

#[cfg(test)]
mod rice_tests {
	use super::{BinsBuildingStrategy, Rice};
	use ndarray::array;

	#[test]
	fn constant_array_are_bad() {
		assert!(
			Rice::from_array(&array![1, 1, 1, 1, 1, 1, 1])
				.unwrap_err()
				.is_strategy()
		);
	}

	#[test]
	fn empty_arrays_are_bad() {
		assert!(
			Rice::<usize>::from_array(&array![])
				.unwrap_err()
				.is_empty_input()
		);
	}
}

#[cfg(test)]
mod sturges_tests {
	use super::{BinsBuildingStrategy, Sturges};
	use ndarray::array;

	#[test]
	fn constant_array_are_bad() {
		assert!(
			Sturges::from_array(&array![1, 1, 1, 1, 1, 1, 1])
				.unwrap_err()
				.is_strategy()
		);
	}

	#[test]
	fn empty_arrays_are_bad() {
		assert!(
			Sturges::<usize>::from_array(&array![])
				.unwrap_err()
				.is_empty_input()
		);
	}
}

#[cfg(test)]
mod fd_tests {
	use super::{BinsBuildingStrategy, FreedmanDiaconis};
	use ndarray::array;

	#[test]
	fn constant_array_are_bad() {
		assert!(
			FreedmanDiaconis::from_array(&array![1, 1, 1, 1, 1, 1, 1])
				.unwrap_err()
				.is_strategy()
		);
	}

	#[test]
	fn zero_iqr_is_bad() {
		assert!(
			FreedmanDiaconis::from_array(&array![-20, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 20])
				.unwrap_err()
				.is_strategy()
		);
	}

	#[test]
	fn empty_arrays_are_bad() {
		assert!(
			FreedmanDiaconis::<usize>::from_array(&array![])
				.unwrap_err()
				.is_empty_input()
		);
	}
}

#[cfg(test)]
mod auto_tests {
	use super::{Auto, BinsBuildingStrategy};
	use ndarray::array;

	#[test]
	fn constant_array_are_bad() {
		assert!(
			Auto::from_array(&array![1, 1, 1, 1, 1, 1, 1])
				.unwrap_err()
				.is_strategy()
		);
	}

	#[test]
	fn zero_iqr_is_handled_by_sturged() {
		assert!(Auto::from_array(&array![-20, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 20]).is_ok());
	}

	#[test]
	fn empty_arrays_are_bad() {
		assert!(
			Auto::<usize>::from_array(&array![])
				.unwrap_err()
				.is_empty_input()
		);
	}
}
