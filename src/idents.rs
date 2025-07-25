//  IDENTIFIERS.rs
//    by Lut99
//
//  Description:
//!   Defines a more powerful alternative for the excellent `paste!()`-macro.
//

use proc_macro::TokenStream;


/***** LIBRARY *****/
/// Defines the implementation of the [`idents()`](super::idents())-macro.
///
/// # Arguments
/// - `input`: Some [`TokenStream`] to match for input.
///
/// # Returns
/// A new [`TokenStream`] that is the same as in, but with some identifiers replaced.
///
/// # Errors
/// This function may error if the input in between `[<` and `>]` is not valid for this macro.
pub fn idents(input: TokenStream) -> Result<TokenStream, TokenStream> {}
