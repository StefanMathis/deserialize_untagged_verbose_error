deserialize_untagged_verbose_error
==================================

<!-- This file has ben generated with build.rs by concatenating docs/links.md,
docs/main.md and (if available docs/end.md). Do not modify this file, instead
modify the components. -->

[`DeserializeUntaggedVerboseError`]: https://docs.rs/deserialize_untagged_verbose_error/0.1.6/deserialize_untagged_verbose_error/derive.DeserializeUntaggedVerboseError.html
[`UntaggedEnumDeError`]: https://docs.rs/deserialize_untagged_verbose_error/0.1.6/deserialize_untagged_verbose_error/struct.UntaggedEnumDeError.html

[![Documentation](https://docs.rs/deserialize_untagged_verbose_error/badge.svg)](https://docs.rs/deserialize_untagged_verbose_error)

A library for creating verbose error messages when deserializing untagged enums.

The full API documentation is available at https://docs.rs/deserialize_untagged_verbose_error/0.1.6/deserialize_untagged_verbose_error.

> **Feedback welcome!**
> Found a bug, missing docs, or have a feature request?
> Please open an issue on [GitHub](https://github.com/StefanMathis/deserialize_untagged_verbose_error/issues).

[`DeserializeUntaggedVerboseError`] is a drop-in replacement for Serde's
`#[serde(untagged)]` enum deserialization that provides detailed errors when
none of the variants match.

It supports unit, tuple, and struct variants, generic types, and the usual Serde
attributes, while retaining the same variant-selection semantics as Serde's
untagged representation. When deserialization fails, instead of simply reporting
that no variant matched, it tells you why each variant failed.

## Basic usage

The following example compares Serde's standard error with the error
produced by this crate.

```rust
use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use serde::Deserialize;
use yaml_serde;

// Standard Serde approach.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
enum ValueSerde {
    Point {
        x: f64,
        y: f64,
    },
    Message {
        #[serde(rename = "time", alias = "epochtime")]
        epochtime: usize,
        content: String,
        #[serde(default)]
        source: String,
    },
    Coordinates(f64, f64),
}

// Using DeserializeUntaggedVerboseError.
#[derive(Debug, DeserializeUntaggedVerboseError, PartialEq)]
#[serde(deny_unknown_fields)]
enum ValueVerbose {
    Point {
        x: f64,
        y: f64,
    },
    Message {
        #[serde(rename = "time", alias = "epochtime")]
        epochtime: usize,
        content: String,
        #[serde(default)]
        source: String,
    },
    Coordinates(f64, f64),
}

let invalid = indoc! {"
    ---
    epochtime: not a timestamp
    content: 42
"};

let serde_error =
    yaml_serde::from_str::<ValueSerde>(invalid).unwrap_err();

let verbose_error =
    yaml_serde::from_str::<ValueVerbose>(invalid).unwrap_err();

// Serde only tells us that none of the variants matched.
assert_eq!(
    serde_error.to_string(),
    "data did not match any variant of untagged enum ValueSerde"
);

// The verbose error explains why every variant failed.
assert_eq!(
    verbose_error.to_string(),
    indoc! {"
    Failed to deserialize the untagged enum ValueVerbose:
    - Could not deserialize as Point: unknown field `content`, expected `x` or `y`.
    - Could not deserialize as Message: invalid type: integer `42`, expected a string.
    - Could not deserialize as Coordinates: invalid type: map, expected a tuple of size 2.
    "}
);

// For valid input, both approaches produce the same result.
let valid = indoc! {"
    ---
    epochtime: 123456789
    content: Hello
"};

let serde_value =
    yaml_serde::from_str::<ValueSerde>(valid).unwrap();

let verbose_value =
    yaml_serde::from_str::<ValueVerbose>(valid).unwrap();

assert_eq!(
    serde_value,
    ValueSerde::Message {
        epochtime: 123456789,
        content: "Hello".to_string(),
        source: String::new(),
    }
);

assert_eq!(
    verbose_value,
    ValueVerbose::Message {
        epochtime: 123456789,
        content: "Hello".to_string(),
        source: String::new(),
    }
);
```

The macro works with any Serde-supported deserialization format. The example
uses YAML simply because it makes the examples easy to read.

# Supported enum variants

[`DeserializeUntaggedVerboseError`] supports all three kinds of Rust enum variants.

## Unit variants

Unit variants work just like they do with Serde's untagged representation.

```rust
use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use yaml_serde;

#[derive(Debug, DeserializeUntaggedVerboseError, PartialEq)]
enum Example {
    Nothing,
    Value(i32),
}

// A YAML null value can deserialize into the unit variant:
let input = "---\n";
let value: Example = yaml_serde::from_str(input).unwrap();
assert_eq!(value, Example::Nothing);

// Unit variants can also participate in the detailed error reporting
let input = indoc! {"
    ---
    hello
"};
let error = yaml_serde::from_str::<Example>(input).unwrap_err();
assert_eq!(
    error.to_string(),
    indoc! {"
        Failed to deserialize the untagged enum Example:
        - Could not deserialize as Nothing: invalid type: string \"hello\", expected unit.
        - Could not deserialize as Value: invalid type: string \"hello\", expected i32.
    "}
);
```

## Tuple variants

Tuple variants may contain one or multiple fields. The tuple length is checked
before attempting to deserialize the individual fields. This ensures that a
tuple variant is only considered a match when the input sequence has exactly the
expected number of elements. This means that a sequence with the wrong number of
elements is reported as such rather than accidentally matching a shorter tuple
variant.

```rust
use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use yaml_serde;

#[derive(Debug, DeserializeUntaggedVerboseError, PartialEq)]
enum Example {
    Pair(i32, String),
    Triple(String, f64, bool),
}

// A two-element sequence selects Pair:
let input = indoc! {"
    ---
    - 42
    - hello
"};
let value: Example = yaml_serde::from_str(input).unwrap();
assert_eq!(
    value,
    Example::Pair(42, "hello".to_string())
);

// A three-element sequence selects Triple:
let input = indoc! {"
    ---
    - hello
    - 3.14
    - true
"};
let value: Example = yaml_serde::from_str(input).unwrap();
assert_eq!(
    value,
    Example::Triple("hello".to_string(), 3.14, true)
);

// Wrong sequence length
let input = indoc! {"
    ---
    - 42
    - hello
    - too much
"};
let error = yaml_serde::from_str::<Example>(input).unwrap_err();
assert_eq!(
    error.to_string(),
    indoc! {"
        Failed to deserialize the untagged enum Example:
        - Could not deserialize as Pair: invalid length 3, expected a tuple of size 2.
        - Could not deserialize as Triple: invalid type: integer `42`, expected a string.
    "}
);
```

## Named variants

```rust
use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use yaml_serde;

#[derive(Debug, DeserializeUntaggedVerboseError, PartialEq)]
enum Example {
    Point {
        x: f64,
        y: f64,
    },
    Message {
        content: String,
    },
}

let input = indoc! {"
    ---
    x: 1
    y: 2
"};
let value: Example = yaml_serde::from_str(input).unwrap();
assert_eq!(
    value,
    Example::Point { x: 1.0, y: 2.0 }
);
```

# Serde attributes

The macro is designed to work with Serde's attributes rather than providing
a separate attribute system.

Container attributes can be placed directly on the enum:

```rust,ignore
#[derive(Debug, DeserializeUntaggedVerboseError)]
#[serde(deny_unknown_fields)]
enum Example {
    Integer {
        value: i32,
    },
    Text {
        value: String,
    },
}
```

Field attributes are supported as well:
```rust,ignore
#[derive(Debug, DeserializeUntaggedVerboseError)]
enum Example {
    Foo {
        #[serde(rename = "the_value")]
        value: i32,

        #[serde(default)]
        other: String,
    },
}
```

Consequently, Serde's standard attributes such as `rename`, `alias`, `default`,
and `deserialize_with` can be used directly on the corresponding fields and
variants.

For example, `rename` changes the serialized/deserialized field name:

```rust
use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use yaml_serde;

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum Example {
    Foo {
        #[serde(rename = "the_value")]
        value: i32,
    },
}

let input = indoc! {"
    ---
    the_value: 42
"};

let value: Example = yaml_serde::from_str(input).unwrap();

match value {
    Example::Foo { value } => assert_eq!(value, 42),
}
```

Likewise, `alias` allows multiple names for the same field:

```rust
use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use yaml_serde;

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum Example {
    Foo {
        #[serde(alias = "the_value")]
        value: i32,
    },
}

let input = indoc! {"
    ---
    the_value: 42
"};

let value: Example = yaml_serde::from_str(input).unwrap();

match value {
    Example::Foo { value } => assert_eq!(value, 42),
}
```

The macro also supports attributes on tuple fields:

```rust
use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use serde::de::Deserialize; // Bring trait into scope for String::deserialize
use yaml_serde;

fn deserialize_special_value<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse().map_err(serde::de::Error::custom)
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum Example {
    Value(
        #[serde(deserialize_with = "deserialize_special_value")]
        i32,
    ),
}

 // String representation of an integer
let input = indoc! {"
    ---
    \"42\"
"};
let value: Example = yaml_serde::from_str(input).unwrap();
match value {
    Example::Value(value) => assert_eq!(value, 42),
}
```

# Generics

Generic enums are supported as well, provided their contained types satisfy
Serde's usual `Deserialize` requirements.

```rust
use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use yaml_serde;

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum Example<T> {
    Foo {
        value: T,
    },
}

let input = indoc! {"
    ---
    value: 42
"};
let value: Example<i32> = yaml_serde::from_str(input).unwrap();
match value {
    Example::Foo { value } => assert_eq!(value, 42),
}

```

Multiple generic parameters are supported too:

```rust,ignore
#[derive(Debug, DeserializeUntaggedVerboseError)]
enum Example<F, T> {
    Point {
        x: F,
        y: F,
    },
    Value {
        value: T,
    },
}
```

# Error handling

When deserialization succeeds, [`DeserializeUntaggedVerboseError`] behaves like
Serde's `untagged` representation: variants are attempted in declaration
order and the first successful variant is returned.

When all variants fail, the macro collects the individual errors into an
[`UntaggedEnumDeError`]. The resulting error contains the name of the enum and
the error produced while attempting each variant.

This makes errors from deeply nested untagged enums substantially easier to
diagnose. Instead of simply receiving

`data did not match any variant of untagged enum MyEnum`

you can see which variants were attempted and why each one failed.

# Implementation notes

Serde's `untagged` representation needs to attempt deserialization of the
same input more than once. `DeserializeUntaggedVerboseError` therefore first
deserializes the input into an intermediate representation and then replays
that representation for each variant.

The intermediate representation is used only during deserialization and is
an implementation detail of the macro.

Because all variant errors need to be retained when deserialization fails,
using `DeserializeUntaggedVerboseError` has some additional overhead compared
to Serde's native `untagged` implementation. In particular, the input needs
to be buffered and each candidate variant is attempted independently.

For applications where the additional diagnostic information is valuable,
this trade-off can make failures considerably easier to understand and debug.

# Alternatives

[serde-untagged](https://crates.io/crates/serde-untagged) provides a more
general mechanism for manually attempting several deserialization strategies.
It is useful when the deserialization logic itself needs to be customized.

`DeserializeUntaggedVerboseError`, on the other hand, is intended to provide a
drop-in derive-style alternative to Serde's `#[serde(untagged)]` representation
while retaining detailed information about why every candidate variant failed.
