use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
enum SerdeNamedAttributes {
    Foo { value: i32 },
    Bar { value: String },
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[serde(deny_unknown_fields)]
#[allow(dead_code)]
enum VerboseNamedAttributes {
    Foo { value: i32 },
    Bar { value: String },
}

#[test]
fn test_named_variant_deny_unknown_fields() {
    let yaml = indoc! {"
    ---
    value: 42
    extra: hello
    "};

    let serde_result = yaml_serde::from_str::<SerdeNamedAttributes>(yaml);
    let verbose_result = yaml_serde::from_str::<VerboseNamedAttributes>(yaml);

    assert!(serde_result.is_err());
    assert!(verbose_result.is_err());

    assert_eq!(
        verbose_result.unwrap_err().to_string(),
        indoc! {"
        Failed to deserialize the untagged enum VerboseNamedAttributes:
        - Could not deserialize as Foo: unknown field `extra`, expected `value`.
        - Could not deserialize as Bar: unknown field `extra`, expected `value`.
        "}
    );
}

#[test]
fn test_named_variant_deny_unknown_fields_valid() {
    let yaml = indoc! {"
    ---
    value: 42
    "};

    let value = yaml_serde::from_str::<VerboseNamedAttributes>(yaml).unwrap();

    match value {
        VerboseNamedAttributes::Foo { value } => {
            assert_eq!(value, 42);
        }
        VerboseNamedAttributes::Bar { .. } => {
            panic!("deserialized as Bar instead of Foo");
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SerdeRenamed {
    Foo {
        #[serde(rename = "the_value")]
        value: i32,
    },
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum VerboseRenamed {
    Foo {
        #[serde(rename = "the_value")]
        value: i32,
    },
}

#[test]
fn test_named_field_rename() {
    let yaml = indoc! {"
    ---
    the_value: 42
    "};

    let serde_value: SerdeRenamed = yaml_serde::from_str(yaml).unwrap();

    let verbose_value: VerboseRenamed = yaml_serde::from_str(yaml).unwrap();

    match serde_value {
        SerdeRenamed::Foo { value } => {
            assert_eq!(value, 42);
        }
    }

    match verbose_value {
        VerboseRenamed::Foo { value } => {
            assert_eq!(value, 42);
        }
    }
}

#[test]
fn test_named_field_rename_rejects_original_name() {
    let yaml = indoc! {"
    ---
    value: 42
    "};

    let serde_result = yaml_serde::from_str::<SerdeRenamed>(yaml);

    let verbose_result = yaml_serde::from_str::<VerboseRenamed>(yaml);

    assert!(serde_result.is_err());
    assert!(verbose_result.is_err());
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SerdeAlias {
    Foo {
        #[serde(alias = "the_value")]
        value: i32,
    },
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum VerboseAlias {
    Foo {
        #[serde(alias = "the_value")]
        value: i32,
    },
}

#[test]
fn test_named_field_alias() {
    {
        let yaml = indoc! {"
        ---
        value: 42
        "};

        let value: VerboseAlias = yaml_serde::from_str(yaml).unwrap();

        match value {
            VerboseAlias::Foo { value } => {
                assert_eq!(value, 42);
            }
        }
    }

    {
        let yaml = indoc! {"
        ---
        the_value: 42
        "};

        let value: VerboseAlias = yaml_serde::from_str(yaml).unwrap();

        match value {
            VerboseAlias::Foo { value } => {
                assert_eq!(value, 42);
            }
        }
    }
}

#[test]
fn test_named_field_alias_matches_serde() {
    let yaml = indoc! {"
    ---
    the_value: 42
    "};

    let serde_value: SerdeAlias = yaml_serde::from_str(yaml).unwrap();

    let verbose_value: VerboseAlias = yaml_serde::from_str(yaml).unwrap();

    match (serde_value, verbose_value) {
        (
            SerdeAlias::Foo { value: serde_value },
            VerboseAlias::Foo {
                value: verbose_value,
            },
        ) => {
            assert_eq!(serde_value, verbose_value);
            assert_eq!(verbose_value, 42);
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SerdeDefault {
    Foo {
        value: i32,

        #[serde(default)]
        other: String,
    },
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum VerboseDefault {
    Foo {
        value: i32,

        #[serde(default)]
        other: String,
    },
}

#[test]
fn test_named_field_default() {
    let yaml = indoc! {"
    ---
    value: 42
    "};

    let serde_value: SerdeDefault = yaml_serde::from_str(yaml).unwrap();

    let verbose_value: VerboseDefault = yaml_serde::from_str(yaml).unwrap();

    match serde_value {
        SerdeDefault::Foo { value, other } => {
            assert_eq!(value, 42);
            assert_eq!(other, "");
        }
    }

    match verbose_value {
        VerboseDefault::Foo { value, other } => {
            assert_eq!(value, 42);
            assert_eq!(other, "");
        }
    }
}
