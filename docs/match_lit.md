Given any literal, will conditionally return a given stream of tokens based on its type.

This can be used to further match on `literal`s given to procedural macros. See [below](#examples) for examples.


# Syntax
This macro attempts to emulate match-like syntax:
```plain
$lit:literal { $($match:match_id => $($tokens:tt)*),* $(,)? }
```
Or, in human language:
- First, give the literal to match;
- Then, open a curly bracket;
- Give a list of zero or more `branches`:
  - Give a so-called "match identifier" first (see [below](#match-identifiers));
  - Then write the `=>`;
  - Write any number of tokens that should be generated when this branch is matched; and
  - Finally, end the branch with a comma (or the end of the list).
- End the input with a closing curly bracket.

Every branch given is prefixed by some identifier that will match a certain group of literals. These identifiers are given [below](#match-identifiers).


# Match identifiers
This is a list of all the match identifiers you can use at the head of branches:
- _Boolean literals_
  - `bool` | `boollike`: Matches boolean literals (`true` or `false`).
- _Integer literals_
  - `int` | `intlike`: Matches *any* integer literal.
  - `int_`: Matches integer literals that specifically _don't_ have a suffix (e.g., `42`).
  - `int8`: Matches any integer literal that is marked as 1 byte in width (e.g., `42i8` or `42u8`).
  - `int16`: Matches any integer literal that is marked as 2 bytes in width (e.g., `42i16` or `42u16`).
  - `int32`: Matches any integer literal that is marked as 4 bytes in width (e.g., `42i32` or `42u32`).
  - `int64`: Matches any integer literal that is marked as 8 bytes in width (e.g., `42i64` or `42u64`).
  - `int128`: Matches any integer literal that is marked as 16 bytes in width (e.g., `42i128` or `42u128`).
  - `size`: Matches any integer literal that has the host address width (e.g., `42isize` or `42usize`).
  - `sint`: Matches any signed integer literal (e.g., `42i8`, `42i32` or `42isize`).
  - `i8`: Matches any signed, single byte integer literal (e.g., `42i8`).
  - `i16`: Matches any signed, double byte integer literal (e.g., `42i16`).
  - `i32`: Matches any signed, quadruple byte integer literal (e.g., `42i32`).
  - `i64`: Matches any signed, octuple byte integer literal (e.g., `42i64`).
  - `i128`: Matches any signed, seduple byte integer literal (e.g., `42i128`).
  - `isize`: Matches any signed, system address-compatible integer literal (e.g., `42isize`).
  - `uint`: Matches any unsigned integer literal (e.g., `42u8`, `42u32` or `42usize`).
  - `u8`: Matches any unsigned, single byte integer literal (e.g., `42u8`).
  - `u16`: Matches any unsigned, double byte integer literal (e.g., `42u16`).
  - `u32`: Matches any unsigned, quadruple byte integer literal (e.g., `42u32`).
  - `u64`: Matches any unsigned, octuple byte integer literal (e.g., `42u64`).
  - `u128`: Matches any unsigned, seduple byte integer literal (e.g., `42u128`).
  - `usize`: Matches any unsigned, system address-compatible integer literal (e.g., `42usize`).
- _Floating-point literals_
  - `float` | `floatlike`: Matches *any* floating-point literal.
  - `float_`: Matches float literals that specifically _don't_ have a suffix (e.g., `42.0`).
  - `f32`: Matches single-precision floating-point literals (e.g., `42.0f32`).
  - `f64`: Matches double-precision floating-point literals (e.g., `42.0f32`).
- _Character literals_
  - `charlike`: Matches *any* character literal.
  - `char`: Matches string characters (e.g., `'4'`).
  - `byte`: Matches byte characters (e.g., `b'2'`).
- _String literals_
  - `stringlike`: Matches *any* string literal.
  - `bytes` | `bstring`: Matches byte string literals (e.g., `b"42"`).
  - `text`: Matches Rust- or C-style string literals (e.g., `"42"` or `c"42"`).
  - `string`: Matches Rust-style string literals (e.g., `"42"`).
  - `cstring`: Matches C-style string literals (e.g., `c"42"`).
- _Miscellaneous_
  - `_`: Matches any literal. Usually useful as a generic catch-all.


# Examples
The basic usage looks as follows:
```rust
use macro_toolkit::match_lit;

// Let's define a macro that does different things based on input!
macro_rules! print {
    ($ident:ident) => { "ident" };
    (()) => { "parens" };

    // Now for literals...
    ($lit:literal) => {
        match_lit!($lit {
            int => "int",
            string => "string",
            _ => "other",
        });
    };
}

assert_eq!(print!(foo), "ident");
assert_eq!(print!(()), "parens");
assert_eq!(print!(42), "int");
assert_eq!(print!("Hello, world!"), "string");
assert_eq!(print!(42.0), "other");
```

The branches are matched by order, i.e., the first branch that matches is selected. For example:
```rust
use macro_toolkit::match_lit;

macro_rules! print {
    ($lit:literal) => {
        match_lit!($lit {
            i32 => "sint32",
            int32 => "int32",
            int => "int",
            _ => "other",
        });
    };
}

assert_eq!(print!(42i32), "sint32");
assert_eq!(print!(42u32), "int32");
assert_eq!(print!(42i16), "int");
assert_eq!(print!(42.0), "other");
```

If you leave any branch unmatched, it will cause compile errors, but _only_ when no match is found:
```rust
use macro_toolkit::match_lit;

macro_rules! print {
    ($lit:literal) => {
        match_lit!($lit {
            int => "int",
        });
    };
}
assert_eq!(print!(42), "int"); // Fine!
```
```compile_fail
use macro_toolkit::match_lit;

macro_rules! print {
    ($lit:literal) => {
        match_lit!($lit {
            int => "int",
        });
    };
}
assert_eq!(print!(42.0), "???"); // Error!
```

Finally, note that Rust's declarative macro semantics also allows us to still use the input token in the output:
```rust
use macro_toolkit::match_lit;

macro_rules! print {
    ($lit:literal) => {
        match_lit!($lit {
            int => format!("int {}", $lit),
            string => $lit,
        });
    };
}

assert_eq!(print!(42), "int 42");
assert_eq!(print!("Hello, world!"), "Hello, world!");
```
