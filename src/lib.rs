#![cfg_attr(docsrs, feature(doc_cfg))]
//  LIB.rs
//    by Lut99
//
//  Description:
//!   A collection of procedural macros that generally help writing declarative macros.
//!
//!
//!   # Macros
//!   This crate provides the following macros:
//!   - `match_lit!()`: A helper macro that can further match `literal` tokens passed to declarative macros.
//!
//!
//!   # Usage
//!   To use this crate, simply add it to your workspace as a dependency:
//!   ```toml
//!   [dependency]
//!   macro-toolkit = { git = "https://github.com/Lut99/macro-toolkit-rs" }
//!   ```
//!
//!   Optionally, you can commit to a specific tag:
//!   ```toml
//!   [dependency]
//!   macro-toolkit = { git = "https://github.com/Lut99/macro-toolkit-rs", tag = "v0.1.0" }
//!   ```
//!
//!   To see documentation, clone the repo and run:
//!   ```sh
//!   cargo doc --no-deps --open
//!   ```
//!
//!
//!   # Features
//!   This crate has the following features:
//!   - `macro_lit`: Enables the compilation of the `macro_lit!()`-macro _(default)._
//!
//!
//!   # Contribution
//!   Contributions to this crate are welcome! Simply [raise an issue](https://github.com/Lut99/macro-toolkit-rs/issues) or [create a PR](https://github.com/Lut99/macro-toolkit-rs/pulls).
//!
//!
//!   # License
//!   This crate is licensed under Apache 2.0. See [`LICENSE`](./LICENSE) for more information.
//

// Modules
#[cfg(feature = "idents")]
mod idents;
#[cfg(feature = "match_lit")]
mod match_lit;
mod utils;

// Imports
#[allow(unused)]
use proc_macro::TokenStream;


/***** PROCEDURAL MACROS *****/
#[cfg(feature = "match_lit")]
#[cfg_attr(docsrs, doc(cfg(feature = "match_lit")))]
#[doc = include_str!("../docs/match_lit.md")]
#[inline]
#[proc_macro]
pub fn match_lit(input: TokenStream) -> TokenStream {
    match match_lit::match_lit(input.into()) {
        Ok(res) => res.into(),
        Err(err) => err.into(),
    }
}



#[cfg(feature = "idents")]
#[cfg_attr(docsrs, doc(cfg(feature = "idents")))]
#[doc = include_str!("../docs/idents.md")]
#[inline]
#[proc_macro]
pub fn idents(input: TokenStream) -> TokenStream {
    match idents::idents(input) {
        Ok(res) => res,
        Err(err) => err,
    }
}
