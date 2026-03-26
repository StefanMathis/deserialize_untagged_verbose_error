deserialize_untagged_verbose_error
==================================

[`DeserializeUntaggedVerboseError`]: https://docs.rs/deserialize_untagged_verbose_error/0.1.5/deserialize_untagged_verbose_error/derive.DeserializeUntaggedVerboseError.html
[`UntaggedEnumDeError`]: https://docs.rs/deserialize_untagged_verbose_error/0.1.5/deserialize_untagged_verbose_error/struct.UntaggedEnumDeError.html

[![Documentation](https://docs.rs/deserialize_untagged_verbose_error/badge.svg)](https://docs.rs/deserialize_untagged_verbose_error)

A library for creating verbose error messages when deserializing untagged enums.

The full API documentation is available at https://docs.rs/deserialize_untagged_verbose_error/0.1.5/deserialize_untagged_verbose_error.

> **Feedback welcome!**  
> Found a bug, missing docs, or have a feature request?  
> Please open an issue on GitHub.

In [serde](https://serde.rs), using the
[`untagged`](https://serde.rs/enum-representations.html#untagged)
representation of an enum has one big disadvantage when deserializing: 
The error message returned in case of failure is very unspecific and does not
explain why deserializing the different variants failed. There have been
[attempts to integrate a more verbose handling into serde](https://github.com/serde-rs/serde/pull/1544)
in the past, but so far, no consensus has been reached.

This crate offers a macro [`DeserializeUntaggedVerboseError`] which can be
applied to any macro where each variant is a tuple struct with a single field.
It behaves in the same way as a combination of the [`Deserialize`](https://serde.rs/derive.html)
with the [`untagged`](https://serde.rs/enum-representations.html#untagged) attribute.
However, in case of a deserialization failure, it collects all errors into an 
[`UntaggedEnumDeError`], providing detailed information why deserializing each 
variant failed.

> **Feedback welcome!**  
> Found a bug, missing docs, or have a feature request?  
> Please open an issue on GitHub.


The following snippet shows a side-by-side comparison with the
native [serde](https://serde.rs) error message:

```rust
use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use serde::Deserialize;
use indoc::indoc;

// Just here to provide a payload to test against - but the macro works with
// any serde-supported format
use serde_yaml;

// Random structs used as variant of the enum
#[derive(Debug, Deserialize, PartialEq)]
#[allow(dead_code)]
struct Point {
    x: f64,
    y: f64,
}
#[derive(Debug, Deserialize, PartialEq)]
#[allow(dead_code)]
struct Message {
    epochtime: usize,
    content: String,
}

// Standard Serde approach
#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
#[allow(dead_code)]
enum VarSerde {
    Message(Message),
    Point(Point),
    Value(f64),
}

// Using the macro provided by this crate
#[derive(Debug, DeserializeUntaggedVerboseError, PartialEq)]
#[allow(dead_code)]
enum VarVerboseErr {
    Message(Message),
    Point(Point),
    Value(f64),
}

let invalid_str = indoc! {"
---
name: Serde
"};

// Deserializing "invalid_str" fails, because it does not match any variant of
// VarSerde / VarVerboseErr
let err_serde = serde_yaml::from_str::<VarSerde>(invalid_str).unwrap_err();
let err_verbose = serde_yaml::from_str::<VarVerboseErr>(invalid_str).unwrap_err();

// Compare the error messages:
assert_eq!(
    err_serde.to_string(),
    "data did not match any variant of untagged enum VarSerde"
);
assert_eq!(
    err_verbose.to_string(),
    indoc! {"
    Failed to deserialize the untagged enum VarVerboseErr:
    - Could not deserialize as Message: missing field `epochtime`.
    - Could not deserialize as Point: missing field `x`.
    - Could not deserialize as Value: invalid type: map, expected f64.
    "}
);

// For valid inputs, both variants behave identical
let valid_str = indoc! {"
---
x: 1
y: 2
"};

let v1 = serde_yaml::from_str::<VarSerde>(valid_str).unwrap();
match v1 {
    VarSerde::Point(pt) => {
        assert_eq!(pt.x, 1.0);
        assert_eq!(pt.y, 2.0);
    },
    _ => panic!("Test failed")
}

let v2 = serde_yaml::from_str::<VarVerboseErr>(valid_str).unwrap();
match v2 {
    VarVerboseErr::Point(pt) => {
        assert_eq!(pt.x, 1.0);
        assert_eq!(pt.y, 2.0);
    },
    _ => panic!("Test failed")
}
```

# Implementation and limitations

For the example shown above, applying [`DeserializeUntaggedVerboseError`] to
`VarVerboseErr` generates roughly the following code:

```rust,ignore
impl<'de> serde::de::Deserialize<'de> for VarDeUnVeEr {
    fn deserialize<__D>(__deserializer: __D) -> Result<Self, __D::Error>
    where
        __D: serde::de::Deserializer<'de>,
    {
        let __content =
            <serde::__private::de::Content as serde::Deserialize>::deserialize(__deserializer)?;
        let __deserializer =
            serde::__private::de::ContentRefDeserializer::<__D::Error>::new(&__content);
        use serde::de::Error;
        let mut __errors: [::std::mem::MaybeUninit<(&'static str, __D::Error)>; 3usize] =
            [const { ::std::mem::MaybeUninit::uninit() }; 3usize];
        let mut __counter: usize = 0;
        match Message::deserialize(__deserializer) {
            Ok(__var) => return Ok(VarDeUnVeEr::Message(__var)),
            Err(__error) => {
                let __elem = &mut __errors[__counter];
                __elem.write((stringify!(Message), __error));
                __counter += 1;
            }
        }
        match Point::deserialize(__deserializer) {
            Ok(__var) => return Ok(VarDeUnVeEr::Point(__var)),
            Err(__error) => {
                let __elem = &mut __errors[__counter];
                __elem.write((stringify!(Point), __error));
                __counter += 1;
            }
        }
        match f64::deserialize(__deserializer) {
            Ok(__var) => return Ok(VarDeUnVeEr::Value(__var)),
            Err(__error) => {
                let __elem = &mut __errors[__counter];
                __elem.write((stringify!(Value), __error));
                __counter += 1;
            }
        }
        let __errors_init: [(&'static str, __D::Error); 3usize] = unsafe {
            [
                std::ptr::read(&__errors[0]).assume_init(),
                std::ptr::read(&__errors[1]).assume_init(),
                std::ptr::read(&__errors[2]).assume_init(),
            ]
        };
        return Err(__D::Error::custom(
            deserialize_untagged_verbose_error::UntaggedEnumDeError {
                enum_name: stringify!(VarDeUnVeEr),
                errors: __errors_init,
            },
        ));
    }
}
```

This has the following implications:
- The macro only works for enums where all variants have a single field.
```rust,ignore

// This example compiles
#[derive(Debug, DeserializeUntaggedVerboseError)]
enum VarVerboseErr {
    Message(Message),
    Point(Point),
    Value(f64),
}

// This one does not
#[derive(Debug, DeserializeUntaggedVerboseError)]
enum Example {
    None, // Variants without fields are not allowed
    Point(f64, f64), // Variants with more than one field are not allowed
    Value { x: i64 }, // Struct variants are not allowed
}
```
- All errors which occur when trying to deserialize the different variants
need to be collected into an array which is part of [`UntaggedEnumDeError`].
Even though this array is allocated on the stack, this still leads to slight
performance losses compared to the combination of `Deserialize` and `untagged`.

# Alternatives to this crate

[serde-untagged](https://crates.io/crates/serde-untagged) provides a much more
general solution which works for all possible enum variants (not just tuple
structs with one field). In exchange, it requires writing a lot of boilerplate
and also does not provide a verbose error where the failure for each variant
is explained.

# Documentation

The full API documentation is available at [https://docs.rs/deserialize_untagged_verbose_error/0.1.5/deserialize_untagged_verbose_error/](https://docs.rs/deserialize_untagged_verbose_error/0.1.5/deserialize_untagged_verbose_error/).