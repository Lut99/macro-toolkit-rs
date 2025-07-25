//  MATCH LITERAL.rs
//    by Lut99
//
//  Description:
//!   Showcases & tests the `match_lit()`-macro.
//

use macro_toolkit::match_lit;


/***** TESTS *****/
#[test]
fn test_match_lit_simple() {
    assert_eq!(match_lit!(42 { int => "int", string => "string" }), "int");
    assert_eq!(match_lit!("42" { int => "int", string => "string" }), "string");
}

#[test]
fn test_match_lit_priority() {
    assert_eq!(match_lit!(42 { i32 => "int32", int => "int" }), "int");
    assert_eq!(match_lit!(42i32 { i32 => "int32", int => "int" }), "int32");
}

#[test]
fn test_match_lit_macro() {
    macro_rules! type_lit {
        ($lit:literal) => {
            match_lit!($lit {
                int => "int",
                string => "string",
            })
        };
    }

    assert_eq!(type_lit!(42), "int");
    assert_eq!(type_lit!("42"), "string");
}

#[test]
fn test_match_lit_nested() {
    macro_rules! type_lit {
        ($lit:literal) => {
            match_lit!($lit {
                int => $lit,
                string => "string",
            })
        };
    }

    assert_eq!(type_lit!(42), 42);
    assert_eq!(type_lit!("42"), "string");
}
