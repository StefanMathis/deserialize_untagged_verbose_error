/*!
[`DeserializeUntaggedVerboseError`]: crate::DeserializeUntaggedVerboseError
[`UntaggedEnumDeError`]: crate::UntaggedEnumDeError

A library for creating verbose error messages when deserializing untagged enums.

 */
#![doc = include_str!("../docs/main.md")]
#![deny(missing_docs)]

pub use deserialize_untagged_verbose_error_macro::DeserializeUntaggedVerboseError;
pub use serde as __serde;
pub use serde_value as __serde_value;

/**
The error returned by [`DeserializeUntaggedVerboseError`].
 */
#[derive(Debug, Clone)]
pub struct UntaggedEnumDeError<const VARIANTS: usize, D: std::error::Error> {
    /**
    Name of the untagged enum we attempted to deserialize using [`DeserializeUntaggedVerboseError`].
     */
    pub enum_name: &'static str,
    /**
    Array which holds all errors which resulted from the failed deserialization
    attempts of the variants.

    For example, if deserializing the following enum failed:
    ```ignore
    #[derive(Debug, DeserializeUntaggedVerboseError)]
    enum DifferentTypes {
        Message(String),
        Index(usize),
        Value(f64),
    }
    ```
    - `errors[0]` explaions why deserializing into a `String` failed,
    - `errors[1]` explaions why deserializing into an `usize` failed,
    - `errors[2]` explaions why deserializing into a `f64` failed.
     */
    pub errors: [(&'static str, D); VARIANTS],
}

impl<const VARIANTS: usize, D: std::error::Error> std::fmt::Display
    for UntaggedEnumDeError<VARIANTS, D>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "Failed to deserialize the untagged enum {}:",
            self.enum_name
        )?;
        for err in self.errors.iter() {
            writeln!(f, "- Could not deserialize as {}: {}.", err.0, err.1)?;
        }
        return Ok(());
    }
}

impl<const VARIANTS: usize, D: std::error::Error> std::error::Error
    for UntaggedEnumDeError<VARIANTS, D>
{
}
