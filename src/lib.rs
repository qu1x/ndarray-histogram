//! Histogram support for n-dimensional arrays ([`ndarray`]).
//!
//! # Features
//!
//!   * `rayon` for parallel sorting and bulk-selection as part of histogram computations.

#![deny(
	missing_docs,
	rustdoc::broken_intra_doc_links,
	rustdoc::missing_crate_level_docs
)]
#![allow(clippy::tabs_in_doc_comments)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub use crate::histogram::HistogramExt;
pub use crate::maybe_nan::{MaybeNan, MaybeNanExt, N32, N64, O32, O64, n32, n64, o32, o64};
pub use crate::quantile::{Quantile1dExt, QuantileExt, interpolate};

pub use ndarray;

#[macro_use]
mod private {
	/// This is a public type in a private module, so it can be included in
	/// public APIs, but other crates can't access it.
	pub struct PrivateMarker;

	/// Defines an associated function for a trait that is impossible for other
	/// crates to implement. This makes it possible to add new associated
	/// types/functions/consts/etc. to the trait without breaking changes.
	macro_rules! private_decl {
		() => {
			/// This method makes this trait impossible to implement outside of
			/// `ndarray-stats` so that we can freely add new methods, etc., to
			/// this trait without breaking changes.
			///
			/// We don't anticipate any other crates needing to implement this
			/// trait, but if you do have such a use-case, please let us know.
			///
			/// **Warning** This method is not considered part of the public
			/// API, and client code should not rely on it being present. It
			/// may be removed in a non-breaking release.
			fn __private__(&self, _: crate::private::PrivateMarker);
		};
	}

	/// Implements the associated function defined by `private_decl!`.
	macro_rules! private_impl {
		() => {
			fn __private__(&self, _: crate::private::PrivateMarker) {}
		};
	}
}

pub mod errors;
pub mod histogram;
mod maybe_nan;
mod quantile;
