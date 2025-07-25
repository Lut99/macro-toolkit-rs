//  UTILITIES.rs
//    by Lut99
//
//  Description:
//!   Defines some utilities used across crates.
//

use proc_macro2::{
    Delimiter as Delimiter2, Group as Group2, Ident as Ident2, Literal as Literal2, Punct as Punct2, Spacing as Spacing2, Span as Span2,
    TokenStream as TokenStream2, TokenTree as TokenTree2,
};


/***** LIBRARY *****/
/// A generic helper struct that pulls some struct-time specialization for a convenient interface.
struct With<T>(std::marker::PhantomData<T>);
impl<const LEN: usize> With<[TokenTree2; LEN]> {
    /// Given a list of [`TokenTree2`], sets all of their spans to the given one.
    ///
    /// # Arguments
    /// - `span`: The span to set it with.
    /// - `tts`: The list of token trees to set.
    ///
    /// # Returns
    /// The same list, but with all spans updated.
    pub fn span(span: Span2, mut tts: [TokenTree2; LEN]) -> [TokenTree2; LEN] {
        for tt in &mut tts {
            tt.set_span(span);
        }
        tts
    }
}



/// Generates a [`TokenStream2`] encoding an error.
///
/// # Arguments
/// - `span`: Some [`Span2`] to have the error point to.
/// - `message`: Some message to show with the error.
///
/// # Returns
/// A [`TokenStream2`] that encodes a [`compile_error!()`](::core::compile_error!) pointing to your
/// `span` with your `message`.
pub fn error2(span: Span2, message: &str) -> TokenStream2 {
    let mut res = TokenStream2::new();
    res.extend(With::span(span, [
        TokenTree2::Punct(Punct2::new(':', Spacing2::Joint)),
        TokenTree2::Punct(Punct2::new(':', Spacing2::Alone)),
        TokenTree2::Ident(Ident2::new("core", span)),
        TokenTree2::Punct(Punct2::new(':', Spacing2::Joint)),
        TokenTree2::Punct(Punct2::new(':', Spacing2::Alone)),
        TokenTree2::Ident(Ident2::new("compile_error", span)),
        TokenTree2::Punct(Punct2::new('!', Spacing2::Alone)),
        TokenTree2::Group(Group2::new(Delimiter2::Parenthesis, {
            let mut res = TokenStream2::new();
            res.extend([TokenTree2::Literal(Literal2::string(message))]);
            res
        })),
    ]));
    res
}
