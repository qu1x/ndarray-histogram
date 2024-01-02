# ndarray-histogram

[![Build][]](https://github.com/qu1x/ndarray-histogram/actions/workflows/build.yml)
[![Documentation][]](https://docs.rs/ndarray-histogram)
[![Downloads][]](https://crates.io/crates/ndarray-histogram)
[![Version][]](https://crates.io/crates/ndarray-histogram)
[![Rust][]](https://www.rust-lang.org)
[![License][]](https://opensource.org/licenses)

[Build]: https://github.com/qu1x/ndarray-histogram/actions/workflows/build.yml/badge.svg
[Documentation]: https://docs.rs/ndarray-histogram/badge.svg
[Downloads]: https://img.shields.io/crates/d/ndarray-histogram.svg
[Version]: https://img.shields.io/crates/v/ndarray-histogram.svg
[Rust]: https://img.shields.io/badge/rust-v1.70.0-brightgreen.svg
[License]: https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg

Histogram support for n-dimensional arrays ([`ndarray`]).

See the [release history](RELEASES.md) to keep track of the development.

## Features

  * `rayon` for parallel sorting and bulk-selection as part of histogram computations.

# License

Copyright © 2023-2024 Rouven Spreckels <rs@qu1x.dev>

Copyright © 2018–2022 [`ndarray-stats`] Developers

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSES/Apache-2.0](LICENSES/Apache-2.0) or
   https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSES/MIT](LICENSES/MIT) or https://opensource.org/licenses/MIT)

at your option.

# Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this project by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.

[`ndarray`]: https://docs.rs/ndarray
[`ndarray-stats`]: https://docs.rs/ndarray-stats
