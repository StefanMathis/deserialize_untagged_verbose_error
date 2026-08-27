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

struct VariantFoo {
    x: String,
    y: i32,
    z: bool,
}

fn deserialize_struct_variant<'de, D>(deserializer: D) -> Result<VariantFoo, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct DeHelper {
        x: String,
        y: i32,
    }

    let helper = DeHelper::deserialize(deserializer)?;

    Ok(VariantFoo {
        x: helper.x,
        y: helper.y,
        z: true,
    })
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum VariantWithAttribute {
    #[serde(deserialize_with = "deserialize_struct_variant")]
    Foo { x: String, y: i32, z: bool },
}

#[test]
fn test_attribute_with_struct_variant() {
    {
        let yaml = indoc! {"
            ---
            x: hello
            y: 42
        "};

        let result = yaml_serde::from_str::<VariantWithAttribute>(yaml);

        assert!(matches!(
            result,
            Ok(VariantWithAttribute::Foo {
                x,
                y,
                z: true,
            }) if x == "hello" && y == 42
        ));
    }
    {
        // false is ignored due to the custom deserializer
        let yaml = indoc! {"
            ---
            x: hello
            y: 42
            z: false
        "};

        let result = yaml_serde::from_str::<VariantWithAttribute>(yaml);

        assert!(matches!(
            result,
            Ok(VariantWithAttribute::Foo {
                x,
                y,
                z: true,
            }) if x == "hello" && y == 42
        ));
    }
    {
        // false is ignored due to the custom deserializer
        let yaml = indoc! {"
            ---
            x: hello
            y: 42
            z: 84
        "};

        let result = yaml_serde::from_str::<VariantWithAttribute>(yaml);

        assert!(matches!(
            result,
            Ok(VariantWithAttribute::Foo {
                x,
                y,
                z: true,
            }) if x == "hello" && y == 42
        ));
    }
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum StructVariantWithAttributes {
    #[serde(skip_deserializing)]
    Skipped {
        x: String,
        y: i32,
    },
    Selected {
        x: String,
        y: i32,
    },
}

#[test]
fn test_skip_deserializing_with_struct_variant() {
    let yaml = indoc! {"
        ---
        x: hello
        y: 42
    "};

    let result = yaml_serde::from_str::<StructVariantWithAttributes>(yaml);

    assert!(matches!(
        result,
        Ok(StructVariantWithAttributes::Selected { x, y })
            if x == "hello" && y == 42
    ));
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum StructVariantSkipAndDeserializeWith {
    #[serde(
        skip_deserializing,
        deserialize_with = "deserialize_struct_should_never_be_called"
    )]
    Skipped {
        x: String,
        y: i32,
    },

    Selected {
        x: String,
        y: i32,
    },
}

#[allow(dead_code)]
struct StructVariantFoo {
    x: String,
    y: i32,
}

#[allow(dead_code)]
fn deserialize_struct_should_never_be_called<'de, D>(
    _deserializer: D,
) -> Result<StructVariantFoo, D::Error>
where
    D: serde::Deserializer<'de>,
{
    panic!("deserialize_with was called for a skipped variant");
}

#[test]
fn test_skip_deserializing_overrides_deserialize_with_struct_variant() {
    let yaml = indoc! {"
        ---
        x: hello
        y: 42
    "};

    let result = yaml_serde::from_str::<StructVariantSkipAndDeserializeWith>(yaml);

    assert!(matches!(
        result,
        Ok(StructVariantSkipAndDeserializeWith::Selected { x, y })
            if x == "hello" && y == 42
    ));
}

fn deserialize_struct_variant_from_map<'de, D>(deserializer: D) -> Result<VariantFoo, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct DeHelper {
        x: String,
        y: i32,
    }

    let helper = DeHelper::deserialize(deserializer)?;

    Ok(VariantFoo {
        x: helper.x,
        y: helper.y,
        z: true,
    })
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum LaterDeserializeWithStruct {
    First(i32),
    #[serde(deserialize_with = "deserialize_struct_variant_from_map")]
    Second {
        x: String,
        y: i32,
        z: bool,
    },
}

#[test]
fn test_deserialize_with_later_struct_variant() {
    let yaml = indoc! {"
        ---
        x: hello
        y: 42
        z: 84
    "};

    let result = yaml_serde::from_str::<LaterDeserializeWithStruct>(yaml);

    assert!(matches!(
        result,
        Ok(LaterDeserializeWithStruct::Second {
            x,
            y,
            z: true,
        }) if x == "hello" && y == 42
    ));
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum VariantDoesNotAppearInErrorReporting {
    #[serde(skip_deserializing)]
    Skipped {
        value: i32,
    },
    Selected {
        value: String,
    },
}

#[test]
fn test_skip_deserializing_omitted_from_errors() {
    let yaml = indoc! {"
        ---
        value: true
    "};

    let result = yaml_serde::from_str::<VariantDoesNotAppearInErrorReporting>(yaml);

    let error = result.expect_err("deserialization should fail");
    let error = error.to_string();

    assert!(error.contains("Selected"));
    assert!(!error.contains("Skipped"));
}
