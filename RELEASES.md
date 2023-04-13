# Version 0.1.0 (2023-04-13)

  * Fix binning strategies by using [`ndarray-slice`], which as well comes with
    parallelized sorting and bulk-selection behind the [`rayon`] feature gate.
  * Fork [`ndarray-stats`].

[`ndarray-slice`]: https://docs.rs/ndarray-slice
[`ndarray-stats`]: https://docs.rs/ndarray-stats
[`rayon`]: https://docs.rs/rayon
