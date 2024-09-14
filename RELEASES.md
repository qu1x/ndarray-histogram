# Version 0.4.0 (2024-09-14)

  * Bump dependencies.

# Version 0.3.1 (2024-06-13)

  * Bump dependencies.

# Version 0.3.0 (2024-01-02)

  * Migrate from `noisy_float` to `ordered-float`.

# Version 0.2.0 (2023-04-21)

  * Make `FreedmanDiaconis` strategy more robust. Use improper IQR and Scott's
    rule as asymptotic resort. Introduce maximum number of bins with `u16::MAX`
    as default. Compute `n_bins` arithmetically instead of iteratively.

# Version 0.1.0 (2023-04-13)

  * Fix binning strategies by using [`ndarray-slice`], which as well comes with
    parallelized sorting and bulk-selection behind the [`rayon`] feature gate.
  * Fork [`ndarray-stats`].

[`ndarray-slice`]: https://docs.rs/ndarray-slice
[`ndarray-stats`]: https://docs.rs/ndarray-stats
[`rayon`]: https://docs.rs/rayon
