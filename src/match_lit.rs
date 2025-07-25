//  MATCH LITERALS.rs
//    by Tim Müller
//
//  Description:
//!   Provides a tiny, high-performance macro for switching on literal types.
//

use std::iter::Peekable;

use proc_macro2::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};
use syn::{Lit, LitBool};

use crate::utils::error2;


/***** HELPERS *****/
/// Abstractly sets properties on any [`proc_macro`] item.
struct With<T>(std::marker::PhantomData<T>);
impl<const LEN: usize> With<[TokenTree; LEN]> {
    /// Sets the span of a list of arbitrary token trees.
    ///
    /// # Arguments
    /// - `tts`: Some [`TokenTree`] to set the span of.
    /// - `span`: A [`Span`] to set.
    ///
    /// # Returns
    /// The given `tts`, but then spanned with the given `span`.
    #[inline]
    pub fn span(mut tts: [TokenTree; LEN], span: Span) -> [TokenTree; LEN] {
        for tt in &mut tts {
            tt.set_span(span);
        }
        tts
    }
}





/***** TOKEN PARSING *****/
/// Defines how to treat every possible literal type.
struct Branches {
    /// The literal to match on.
    lit:      Lit,
    /// The branches to match, in the order they're given.
    branches: Vec<Branch>,
}
impl Branches {
    /// Constructor for the Branches that will initialize it as empty.
    ///
    /// # Arguments
    /// - `lit`: The [`Lit`] that we will use to decide.
    ///
    /// # Returns
    /// A Branches that will decide on the given `lit`eral, but will decide to error for every
    /// possible outcome.
    #[inline]
    const fn new(lit: Lit) -> Self { Self { lit, branches: Vec::new() } }
}
impl Branches {
    /// Implements a little parsing state machine for parsing the input.
    ///
    /// # Arguments
    /// - `input`: The input [`TokenStream`] to parse.
    ///
    /// # Returns
    /// A Branches encoding how to match every arm.
    ///
    /// # Errors
    /// This function can error if the input was invalid.
    fn parse(input: TokenStream) -> Result<Self, TokenStream> {
        // Parse the top tree into the literal and group
        let (lit, group): (Lit, Group) = Self::parse_lit_group(input)?;

        // Then parse the group into branches
        let mut iter = group.stream().into_iter().peekable();
        let mut res = Self::new(lit);
        loop {
            // Get the next branch
            match Branch::parse(&mut iter)? {
                Some(branch) => res.branches.push(branch),
                None => return Ok(res),
            }
        }
    }

    /// Parses the initial stream into the literal and the group of match statements.
    ///
    /// # Arguments
    /// - `input`: The input [`TokenStream`] to parse.
    ///
    /// # Returns
    /// A tuple of the parsed [`Literal`] and [`Group`].
    ///
    /// # Errors
    /// This function can error if the input was invalid.
    fn parse_lit_group(input: TokenStream) -> Result<(Lit, Group), TokenStream> {
        enum State {
            /// Initial state.
            Start,
            /// Parsed the initial literal
            Lit(Lit),
            /// Parsed the phrase group
            Group(Lit, Group),
        }

        fn parse_lit(tree: TokenTree) -> Result<Lit, TokenStream> {
            match tree {
                // These are the literals we really match
                TokenTree::Literal(lit) => Ok(Lit::new(lit)),
                TokenTree::Ident(ident) => {
                    let sident = ident.to_string();
                    if sident == "true" {
                        Ok(Lit::Bool(LitBool { value: true, span: ident.span() }))
                    } else if sident == "false" {
                        Ok(Lit::Bool(LitBool { value: false, span: ident.span() }))
                    } else {
                        return Err(error2(ident.span(), "Expected a literal"));
                    }
                },

                // This may occur when given macro input; attempt to recurse into it as single token
                TokenTree::Group(g) if g.delimiter() == Delimiter::None => {
                    // Extract the only token
                    let mut stream = g.stream().into_iter();
                    let tree: TokenTree = stream.next().ok_or_else(|| error2(g.span(), "Expected a literal"))?;
                    if stream.next().is_some() {
                        return Err(error2(g.span(), "Expected a literal"));
                    }

                    // Try to parse *that*
                    parse_lit(tree)
                },

                // Otherwise, it's BAD
                _ => return Err(error2(tree.span(), "Expected a literal")),
            }
        }


        // Go through the input
        let mut state = State::Start;
        for tree in input {
            match state {
                State::Start => state = State::Lit(parse_lit(tree)?),

                State::Lit(lit) => {
                    // Expect the phrase group
                    if let TokenTree::Group(group) = tree {
                        state = State::Group(lit, group);
                        continue;
                    } else {
                        return Err(error2(tree.span(), "Expected match branches wrapped in `{}`"));
                    }
                },

                State::Group(_, _) => return Err(error2(tree.span(), "Expected nothing after the match branches")),
            }
        }
        match state {
            State::Group(lit, group) => Ok((lit, group)),
            _ => Err(error2(Span::mixed_site(), "Expected a literal and then match branches wrapped in `{}`")),
        }
    }
}



/// Defines the possible branches to parse.
struct Branch {
    /// The matcher for this branch.
    matcher: LitMatcher,
    /// The stream to compile to when matched.
    tokens:  TokenStream,
}
impl Branch {
    /// Parses this Branch from an iterator over [`TokenTree`]s.
    ///
    /// # Arguments
    /// - `iter`: The iterator yielding remaining tokens.
    ///
    /// # Returns
    /// A Branch once we parsed enough to parse a branch. If there was nothing left to parse,
    /// returns [`None`].
    ///
    /// # Errors
    /// If the input did not have a valid branch at the head, returns an error.
    fn parse(iter: &mut Peekable<impl Iterator<Item = TokenTree>>) -> Result<Option<Self>, TokenStream> {
        // Match on the specific identifier on the head
        let ident: Ident = match iter.next() {
            Some(TokenTree::Ident(ident)) => ident,
            Some(tt) => return Err(error2(tt.span(), "Expected a match identifier")),
            None => return Ok(None),
        };
        // Match the `=>`
        match iter.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '=' && punct.spacing() == Spacing::Joint => {},
            Some(punct) => return Err(error2(punct.span(), "Expected '=>'")),
            None => return Err(error2(Span::mixed_site(), "Expected '=>'")),
        }
        match iter.next() {
            Some(TokenTree::Punct(punct)) if punct.as_char() == '>' && punct.spacing() == Spacing::Alone => {},
            Some(punct) => return Err(error2(punct.span(), "Expected '=>'")),
            None => return Err(error2(Span::mixed_site(), "Expected '=>'")),
        }
        // Match until a `,` OR the end
        let mut tokens = TokenStream::new();
        while let Some(tt) = iter.next() {
            // This check only exists in the empty case
            if let TokenTree::Punct(p) = tt {
                if p.as_char() == ',' {
                    break;
                }
                tokens.extend([TokenTree::Punct(p)]);
            } else {
                tokens.extend([tt]);
            }
        }

        // Now we have all the components, match the identifier
        Ok(Some(Self { matcher: LitMatcher::parse(ident)?, tokens }))
    }
}

/// Describes all the possible matchers to specify.
enum LitMatcher {
    // Any
    Any,

    // Booleans
    /// Any boolean literal.
    Bool,

    // Integers
    /// Any integer literal.
    Int,
    /// Unsized int, specifically.
    IntUns,
    /// Any 8-bit wide literal.
    Int8,
    /// Any 16-bit wide literal.
    Int16,
    /// Any 32-bit wide literal.
    Int32,
    /// Any 64-bit wide literal.
    Int64,
    /// Any 128-bit wide literal.
    Int128,
    /// Any system-address wide literal.
    IntSize,
    /// Any signed literal.
    IntS,
    /// 8-bit wide, signed literal.
    IntS8,
    /// 16-bit wide, signed literal.
    IntS16,
    /// 32-bit wide, signed literal.
    IntS32,
    /// 64-bit wide, signed literal.
    IntS64,
    /// 128-bit wide, signed literal.
    IntS128,
    /// System-address wide, signed literal.
    IntSSize,
    /// Any unsigned literal.
    IntU,
    /// 8-bit wide, unsigned literal.
    IntU8,
    /// 16-bit wide, unsigned literal.
    IntU16,
    /// 32-bit wide, unsigned literal.
    IntU32,
    /// 64-bit wide, unsigned literal.
    IntU64,
    /// 128-bit wide, unsigned literal.
    IntU128,
    /// System-address wide, unsigned literal.
    IntUSize,

    // Floats
    /// Any floating-point.
    Float,
    /// Unsized floating-point, specifically.
    FloatUns,
    /// 32-bit wide floating-point.
    Float32,
    /// 64-bit wide floating-point.
    Float64,

    // Characters
    /// Any character-like literal.
    Char,
    /// Specifically byte character literal.
    CharByte,
    /// Specifically string character literal.
    CharStr,

    // Strings
    /// Any string-like literal.
    String,
    /// Specifically byte literal.
    StringByte,
    /// Any of the two text literals.
    StringText,
    /// Specifically string literal.
    StringStr,
    /// Specifically c-string literal.
    StringCStr,
}
impl LitMatcher {
    /// Parses this litmatcher from an identifier.
    ///
    /// # Arguments
    /// - `ident`: The [`Ident`] to parse it from.
    ///
    /// # Returns
    /// A LitMatcher specified by the ident.
    ///
    /// # Errors
    /// This function may error if the ident doesn't match any of the matchers.
    #[inline]
    fn parse(ident: Ident) -> Result<Self, TokenStream> {
        match ident.to_string().as_str() {
            // Any
            "_" => Ok(Self::Any),

            // Boolean
            "bool" | "boollike" => Ok(Self::Bool),

            // Integer
            "int" | "intlike" => Ok(Self::Int),
            "int_" => Ok(Self::IntUns),
            "int8" => Ok(Self::Int8),
            "int16" => Ok(Self::Int16),
            "int32" => Ok(Self::Int32),
            "int64" => Ok(Self::Int64),
            "int128" => Ok(Self::Int128),
            "size" => Ok(Self::IntSize),
            "sint" => Ok(Self::IntS),
            "i8" => Ok(Self::IntS8),
            "i16" => Ok(Self::IntS16),
            "i32" => Ok(Self::IntS32),
            "i64" => Ok(Self::IntS64),
            "i128" => Ok(Self::IntS128),
            "isize" => Ok(Self::IntSSize),
            "uint" => Ok(Self::IntU),
            "u8" => Ok(Self::IntU8),
            "u16" => Ok(Self::IntU16),
            "u32" => Ok(Self::IntU32),
            "u64" => Ok(Self::IntU64),
            "u128" => Ok(Self::IntU128),
            "usize" => Ok(Self::IntUSize),

            // Floating-point
            "float" | "floatlike" => Ok(Self::Float),
            "float_" => Ok(Self::FloatUns),
            "f32" => Ok(Self::Float32),
            "f64" => Ok(Self::Float64),

            // Characters
            "charlike" => Ok(Self::Char),
            "char" => Ok(Self::CharStr),
            "byte" => Ok(Self::CharByte),

            // Strings
            "stringlike" => Ok(Self::String),
            "bytes" | "bstring" => Ok(Self::StringByte),
            "text" => Ok(Self::StringText),
            "string" => Ok(Self::StringStr),
            "cstring" => Ok(Self::StringCStr),

            // Any others are invalid
            _ => Err(error2(ident.span(), &format!("Expected a specific literal identifier"))),
        }
    }

    /// Checks whether this matcher matches a literal.
    ///
    /// # Arguments
    /// - `lit`: The [`Literal`] to match on.
    ///
    /// # Returns
    /// True if this matcher would match the given `lit`, or false otherwise.
    #[inline]
    fn match_lit(&self, lit: &Lit) -> bool {
        match (lit, self) {
            // Booleans
            (Lit::Bool(_), Self::Any | Self::Bool) => true,

            // Integers
            (Lit::Int(i), Self::Any | Self::Int | Self::IntUns) if i.suffix().is_empty() => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntS | Self::Int8 | Self::IntS8) if i.suffix() == "i8" => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntS | Self::Int16 | Self::IntS16) if i.suffix() == "i16" => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntS | Self::Int32 | Self::IntS32) if i.suffix() == "i32" => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntS | Self::Int64 | Self::IntS64) if i.suffix() == "i64" => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntS | Self::Int128 | Self::IntS128) if i.suffix() == "i128" => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntU | Self::Int8 | Self::IntU8) if i.suffix() == "u8" => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntU | Self::Int16 | Self::IntU16) if i.suffix() == "u16" => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntU | Self::Int32 | Self::IntU32) if i.suffix() == "u32" => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntU | Self::Int64 | Self::IntU64) if i.suffix() == "u64" => true,
            (Lit::Int(i), Self::Any | Self::Int | Self::IntU | Self::Int128 | Self::IntU128) if i.suffix() == "u128" => true,

            // Floats
            (Lit::Float(f), Self::Any | Self::Float | Self::FloatUns) if f.suffix().is_empty() => true,
            (Lit::Float(f), Self::Any | Self::Float | Self::Float32) if f.suffix() == "f32" => true,
            (Lit::Float(f), Self::Any | Self::Float | Self::Float64) if f.suffix() == "f64" => true,

            // Characters
            (Lit::Byte(_), Self::Any | Self::Char | Self::CharByte) => true,
            (Lit::Char(_), Self::Any | Self::Char | Self::CharStr) => true,

            // Strings
            (Lit::ByteStr(_), Self::Any | Self::String | Self::StringByte) => true,
            (Lit::Str(_), Self::Any | Self::String | Self::StringText | Self::StringStr) => true,
            (Lit::CStr(_), Self::Any | Self::String | Self::StringText | Self::StringCStr) => true,

            // Done
            (_, _) => false,
        }
    }
}





/***** LIBRARY *****/
/// Defines the implementation of the [`match_lit()`](super::match_lit())-macro.
///
/// # Arguments
/// - `input`: Some [`TokenStream`] to match for input.
///
/// # Returns
/// A new [`TokenStream`] with the correct output depending on the type of the literal in the
/// input.
///
/// # Errors
/// This function may error if the input is not valid for this macro.
pub fn match_lit(input: TokenStream) -> Result<TokenStream, TokenStream> {
    // Parse the input, first
    let Branches { lit, branches } = Branches::parse(input)?;

    // Find the first branch that matches
    for branch in branches {
        if !branch.matcher.match_lit(&lit) {
            continue;
        }

        // If we match, then serialize the branch
        return Ok(branch.tokens);
    }

    // If we failed to match any, then error
    Err(error2(lit.span(), "Unmatched literal type"))
}
