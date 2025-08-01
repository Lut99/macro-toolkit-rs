//  IDENTS.rs
//    by Lut99
//
//  Description:
//!   Shows how the [`idents!()`]-macro is very comparable to the OG `paste!()`-macro, but with
//!   additional capabilities.
//

use macro_toolkit::idents;


/***** TESTS *****/
#[test]
fn test_idents_transparent() {
    // This does nothing
    idents! {
        #[derive(Debug, Eq, PartialEq)]
        struct Foo;
        impl Default for Foo {
            #[inline]
            fn default() -> Self { Foo }
        }
    }

    assert_eq!(Foo::default(), Foo);
}

#[test]
fn test_idents_replace() {
    // This will replace all input expressions with identifiers
    macro_rules! build_foo {
        ($($values:expr),*) => {{
            idents! {
                #[derive(Debug)]
                struct Foo<{<...$($values),*>}>({<...$($values),*>});
                Foo($($values),*)
            }
        }};
    }

    assert_eq!(format!("{:?}", build_foo!("Test", 42usize)), "Foo(\"Test\", 42)");
}

#[test]
fn test_idents_replace_named() {
    // This will replace all input expressions with identifiers
    macro_rules! build_foo {
        ($($values:expr),*) => {{
            idents! {
                #[derive(Debug)]
                struct Foo<{<...$($values),*>}>({<...$($values),*>});
                Foo($($values),*)
            }
        }};
    }

    assert_eq!(format!("{:?}", build_foo!("Test", 42usize)), "Foo(\"Test\", 42)");
}
