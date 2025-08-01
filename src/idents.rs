//  IDENTIFIERS.rs
//    by Lut99
//
//  Description:
//!   Defines a more powerful alternative for the excellent `paste!()`-macro.
//

use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, Span, TokenStream, TokenTree};

use crate::utils::error;


/***** TOKEN PARSING *****/
/// Parses this IdentMacro from the contents of a `[]`.
///
/// # Arguments
/// - `input`: The [`TokenStream`] to parse from.
///
/// # Returns
/// A [`Result`] encoding a successfully parsed IdentMacro or a reason why it was illegal; or
/// [`None`] if the inside didn't start with `<` (i.e., it's not a macro).
pub fn parse_brace_contents(input: TokenStream) -> Option<Result<TokenStream, TokenStream>> {
    // Check if it begins with `<`
    let mut iter = input.into_iter();
    if let Some(TokenTree::Punct(punct)) = iter.next() {
        if punct.as_char() != '<' {
            return None;
        }
    } else {
        return None;
    }

    // Optionally, parse the pattern
    let mut pat: Vec<Pat> = Vec::new();


    // It does. The remainder of the iterator is our contents, ending with `>`
    let mut i: usize = 0;
    let mut output = TokenStream::new();
    for token in &mut iter {
        // Check if we need to stop
        match token {
            // Pop commas
            TokenTree::Punct(punct) if punct.as_char() == ',' => output.extend([TokenTree::Punct(punct)]),
            // Stop token
            TokenTree::Punct(punct) if punct.as_char() == '>' => break,

            // The rest maps one-to-one to identifiers
            // We rely on macro rules to give like, invisible groups here to pass e.g. expressions
            token => {
                output.extend([TokenTree::Ident(Ident::new(&format!("T{i}"), token.span()))]);
                i += 1
            },
        }
    }
    if let Some(token) = iter.next() {
        return Some(Err(error(token.span(), "Expected nothing after '>'")));
    }

    // Done
    Some(Ok(output))
}





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
pub fn idents(input: TokenStream) -> Result<TokenStream, TokenStream> {
    // Start to quantify through the input to write it to the output
    let mut output: TokenStream = TokenStream::new();
    for token in input {
        // We look for bracketed areas
        let TokenTree::Group(group) = token else {
            output.extend([token]);
            continue;
        };
        if group.delimiter() != Delimiter::Brace {
            // Recurse into the nested areas
            let mut group = Group::new(group.delimiter(), idents(group.stream())?);
            group.set_span(group.span());
            output.extend([TokenTree::Group(group)]);
            continue;
        }

        // If we have one, further parse it as an identifier macro
        match parse_brace_contents(group.stream()) {
            // We recognized it as ours, but it may be faulty
            Some(res) => output.extend(res?),
            // It's not a macro identifier at all
            None => {
                output.extend([TokenTree::Group(group)]);
                continue;
            },
        };
    }
    Ok(output)
}
