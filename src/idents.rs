//  IDENTIFIERS.rs
//    by Lut99
//
//  Description:
//!   Defines a more powerful alternative for the excellent `paste!()`-macro.
//

use proc_macro::{Delimiter, Group, Ident, Span, TokenStream, TokenTree};

use crate::utils::error;


/***** PASTE TOKEN PARSING *****/
/// Parses the contents of a `{}`.
///
/// Either this recognizes nothing, passing [`None`] back, or else it will recognize it as `{<>}`
/// and do the generic identifier generation.
///
/// # Arguments
/// - `input`: The [`TokenStream`] to parse from.
///
/// # Returns
/// A [`Result`] encoding a successfully parsed identifier or a reason why it was illegal; or
/// [`None`] if the inside didn't start with `<` (i.e., it's not a macro).
fn parse_bracket_contents(input: TokenStream) -> Option<Result<Ident, TokenStream>> {
    // Check if it begins with `<`
    let mut iter = input.into_iter();
    if let Some(TokenTree::Punct(punct)) = iter.next() {
        if punct.as_char() != '<' {
            return None;
        }
    } else {
        return None;
    }

    // It does. The remainder of the iterator is identifier things
    let mut name = String::new();
    let mut span: Option<Span> = None;
    for token in &mut iter {
        // Check if we need to stop
        match token {
            // Identifiers...
            TokenTree::Ident(ident) => {
                name.push_str(&ident.to_string());
                if span.is_none() {
                    span = Some(ident.span());
                }
            },
            // Literals...
            TokenTree::Literal(lit) => {
                name.push_str(&lit.to_string());
                if span.is_none() {
                    span = Some(lit.span());
                }
            },
            // Accepted punctuation...
            TokenTree::Punct(punct) if punct.as_char() == '_' => {
                name.push(punct.as_char());
                if span.is_none() {
                    span = Some(punct.span());
                }
            },

            // Literal computation
            TokenTree::Group(group) if group.delimiter() == Delimiter::Parenthesis => {},

            // Invisible groups
            TokenTree::Group(group) if group.delimiter() == Delimiter::None => {},
            // Quitting `>`
        }
    }
    if let Some(token) = iter.next() {
        return Some(Err(error(token.span(), "Expected nothing after '>'")));
    }

    // Done
    Some(Ok(output))
}





/***** GENERIC TOKEN PARSING *****/
/// Represents the "pattern" at the start that determines how to serialize identifiers.
enum Pat {
    Ident(String),
    Placeholder,
}



/// Parses a pattern (e.g., `T$`) from a stream if it's there.
///
/// Always parses `...` at the end.
///
/// # Returns
fn parse_pattern_and_dots(iter: &mut impl Iterator<Item = TokenTree>) -> Result<Vec<Pat>, TokenStream> {
    let mut dot_count: usize = 0;
    let mut pat: Vec<Pat> = Vec::new();
    for token in iter {
        match token {
            // Parse identifiers and others
            TokenTree::Ident(ident) => {
                if dot_count > 0 {
                    return Err(error(ident.span(), "Expected three dots to end pattern"));
                }
                if let Some(Pat::Ident(name)) = pat.last_mut() {
                    name.push_str(&ident.to_string());
                } else {
                    pat.push(Pat::Ident(ident.to_string()));
                }
            },
            TokenTree::Literal(lit) => {
                if dot_count > 0 {
                    return Err(error(lit.span(), "Expected three dots to end pattern"));
                }
                if let Some(Pat::Ident(name)) = pat.last_mut() {
                    name.push_str(&lit.to_string());
                } else {
                    pat.push(Pat::Ident(lit.to_string()));
                }
            },

            // Parse the placeholder
            TokenTree::Punct(punct) if punct.as_char() == '@' => {
                if dot_count > 0 {
                    return Err(error(punct.span(), "Expected three dots to end pattern"));
                }
                pat.push(Pat::Placeholder);
            },

            // Parse the the invisible group
            TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
                if dot_count > 0 {
                    return Err(error(group.span(), "Expected three dots to end pattern"));
                }
                pat.extend(parse_pattern_and_dots(&mut group.stream().into_iter())?);
            },

            // Parse the three dots
            TokenTree::Punct(punct) if punct.as_char() == '.' => {
                dot_count += 1;
                if dot_count == 3 {
                    break;
                }
            },

            // The rest is just unexpected
            token => return Err(error(token.span(), "Expected identifier pattern OR three dots before ident list")),
        }
    }

    // Double check we've had all dots
    if dot_count != 3 {
        return Err(error(Span::mixed_site(), "Expected three dots before ident list"));
    }

    // Done
    Ok(pat)
}

/// Parses the contents of a `{}`.
///
/// Either this recognizes nothing, passing [`None`] back, or else it will recognize it as `{<>}`
/// and do the generic identifier generation.
///
/// # Arguments
/// - `input`: The [`TokenStream`] to parse from.
///
/// # Returns
/// A [`Result`] encoding a stream of identifiers or a reason why it was illegal; or
/// [`None`] if the inside didn't start with `<` (i.e., it's not a macro).
fn parse_brace_contents(input: TokenStream) -> Option<Result<TokenStream, TokenStream>> {
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
    let pat: Vec<Pat> = match parse_pattern_and_dots(&mut iter) {
        Ok(pat) => pat,
        Err(err) => return Some(Err(err)),
    };

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
                // Build the identifier first
                let ident: Ident = if !pat.is_empty() {
                    let si: String = i.to_string();
                    let mut name = String::new();
                    for pat in &pat {
                        match pat {
                            Pat::Ident(n) => {
                                name.push_str(n);
                            },
                            Pat::Placeholder => {
                                name.push_str(&si);
                            },
                        }
                    }
                    Ident::new(&name, token.span())
                } else {
                    Ident::new(&format!("T{i}"), token.span())
                };

                // Now add it as the replacement
                output.extend([TokenTree::Ident(ident)]);
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
        // `paste`-like idents
        if group.delimiter() == Delimiter::Bracket {
            // If we have one, further parse it as an identifier macro
            match parse_bracket_contents(group.stream()) {
                // We recognized it as ours, but it may be faulty
                Some(res) => output.extend([TokenTree::Ident(res?)]),
                // It's not a macro identifier at all
                None => output.extend([TokenTree::Group(group)]),
            };
            continue;
        }
        // generics generator-idents
        if group.delimiter() == Delimiter::Brace {
            // If we have one, further parse it as an identifier macro
            match parse_brace_contents(group.stream()) {
                // We recognized it as ours, but it may be faulty
                Some(res) => output.extend(res?),
                // It's not a macro identifier at all
                None => output.extend([TokenTree::Group(group)]),
            };
            continue;
        }

        // Recurse into other nested areas
        let mut group = Group::new(group.delimiter(), idents(group.stream())?);
        group.set_span(group.span());
        output.extend([TokenTree::Group(group)]);
    }
    Ok(output)
}
