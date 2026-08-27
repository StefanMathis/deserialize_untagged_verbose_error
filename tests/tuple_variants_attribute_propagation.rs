use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use serde::Deserialize;

fn deserialize_as_string<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    value.parse::<i32>().map_err(serde::de::Error::custom)
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SerdeUnnamedAttributes {
    Foo(
        #[serde(deserialize_with = "deserialize_as_string")] i32,
        String,
    ),
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum VerboseUnnamedAttributes {
    Foo(
        #[serde(deserialize_with = "deserialize_as_string")] i32,
        String,
    ),
}

#[test]
fn test_unnamed_field_deserialize_with() {
    let yaml = indoc! {"
    ---
    - \"42\"
    - hello
    "};

    let serde_value = yaml_serde::from_str::<SerdeUnnamedAttributes>(yaml).unwrap();

    let verbose_value = yaml_serde::from_str::<VerboseUnnamedAttributes>(yaml).unwrap();

    match serde_value {
        SerdeUnnamedAttributes::Foo(number, text) => {
            assert_eq!(number, 42);
            assert_eq!(text, "hello");
        }
    }

    match verbose_value {
        VerboseUnnamedAttributes::Foo(number, text) => {
            assert_eq!(number, 42);
            assert_eq!(text, "hello");
        }
    }
}

#[test]
fn test_unnamed_field_deserialize_with_is_actually_applied() {
    let yaml = indoc! {"
    ---
    - \"not a number\"
    - hello
    "};

    let result = yaml_serde::from_str::<VerboseUnnamedAttributes>(yaml);

    assert!(result.is_err());
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum UnnamedThree {
    Foo(
        String,
        #[serde(deserialize_with = "deserialize_as_string")] i32,
        bool,
    ),
}

#[test]
fn test_unnamed_multiple_fields_with_attribute() {
    let yaml = indoc! {"
    ---
    - hello
    - \"42\"
    - true
    "};

    let value = yaml_serde::from_str::<UnnamedThree>(yaml).unwrap();

    match value {
        UnnamedThree::Foo(text, number, flag) => {
            assert_eq!(text, "hello");
            assert_eq!(number, 42);
            assert!(flag);
        }
    }
}

#[test]
fn test_unnamed_attribute_with_wrong_length() {
    let yaml = indoc! {"
    ---
    - hello
    - \"42\"
    "};

    let result = yaml_serde::from_str::<UnnamedThree>(yaml);

    assert!(result.is_err());

    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("expected a tuple of size 3")
    );
}

fn deserialize_tuple_variant<'de, D>(deserializer: D) -> Result<(String, i32, bool), D::Error>
where
    D: serde::Deserializer<'de>,
{
    let (string, number) = <(String, i32)>::deserialize(deserializer)?;
    return Ok((string, number, true));
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum VariantWithAttribute {
    #[serde(deserialize_with = "deserialize_tuple_variant")]
    Foo(String, i32, bool),
}

#[test]
fn test_attribute_with_variant() {
    {
        let yaml = indoc! {"
        ---
        - hello
        - 42
        "};

        let result = yaml_serde::from_str::<VariantWithAttribute>(yaml);

        assert!(matches!(
            result,
            Ok(VariantWithAttribute::Foo(x, y, true)) if x == "hello" && y == 42
        ));
    }
    {
        // false is ignored due to the custom deserializer
        let yaml = indoc! {"
        ---
        - hello
        - 42
        - false
        "};

        let result = yaml_serde::from_str::<VariantWithAttribute>(yaml);

        assert!(matches!(
            result,
            Ok(VariantWithAttribute::Foo(x, y, true)) if x == "hello" && y == 42
        ));
    }
    {
        // false is ignored due to the custom deserializer
        let yaml = indoc! {"
        ---
        - hello
        - 42
        - 84
        "};

        let result = yaml_serde::from_str::<VariantWithAttribute>(yaml);

        assert!(matches!(
            result,
            Ok(VariantWithAttribute::Foo(x, y, true)) if x == "hello" && y == 42
        ));
    }
}

fn deserialize_tuple_variant_from_map<'de, D>(
    deserializer: D,
) -> Result<(String, i32, bool), D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct DeHelper {
        x: String,
        y: i32,
    }

    let helper = DeHelper::deserialize(deserializer)?;

    Ok((helper.x, helper.y, true))
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum TupleVariantWithDeserializeWith {
    #[serde(deserialize_with = "deserialize_tuple_variant_from_map")]
    Foo(String, i32, bool),
}

#[test]
fn test_deserialize_with_tuple_variant() {
    let yaml = indoc! {"
        ---
        x: hello
        y: 42
        z: 84
    "};

    let result = yaml_serde::from_str::<TupleVariantWithDeserializeWith>(yaml);

    assert!(matches!(
        result,
        Ok(TupleVariantWithDeserializeWith::Foo(x, y, true))
            if x == "hello" && y == 42
    ));
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum LaterDeserializeWithTuple {
    First(i32),
    #[serde(deserialize_with = "deserialize_tuple_variant_from_map")]
    Second(String, i32, bool),
}

#[test]
fn test_deserialize_with_later_tuple_variant() {
    let yaml = indoc! {"
        ---
        x: hello
        y: 42
        z: 84
    "};

    let result = yaml_serde::from_str::<LaterDeserializeWithTuple>(yaml);

    assert!(matches!(
        result,
        Ok(LaterDeserializeWithTuple::Second(x, y, true))
            if x == "hello" && y == 42
    ));
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum TupleVariantWithAttributes {
    #[serde(skip_deserializing)]
    Skipped(String),
    Selected(i32),
}

#[test]
fn test_skip_deserializing_with_tuple_variant() {
    let yaml = indoc! {"
        ---
        42
    "};

    let result = yaml_serde::from_str::<TupleVariantWithAttributes>(yaml);

    assert!(matches!(
        result,
        Ok(TupleVariantWithAttributes::Selected(42))
    ));
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum TupleVariantWithSkippedVariant {
    Selected(i32),

    #[serde(skip_deserializing)]
    Skipped(String),
}

#[test]
fn test_skip_deserializing_with_tuple_variant_after_match() {
    let yaml = indoc! {"
        ---
        42
    "};

    let result = yaml_serde::from_str::<TupleVariantWithSkippedVariant>(yaml);

    assert!(matches!(
        result,
        Ok(TupleVariantWithSkippedVariant::Selected(42))
    ));
}

#[allow(dead_code)]
fn deserialize_should_never_be_called<'de, D>(_deserializer: D) -> Result<(String, i32), D::Error>
where
    D: serde::Deserializer<'de>,
{
    panic!("deserialize_with was called for a skipped variant");
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum TupleVariantSkipAndDeserializeWith {
    #[serde(
        skip_deserializing,
        deserialize_with = "deserialize_should_never_be_called"
    )]
    Skipped(String, i32),

    Selected(String, i32),
}

#[test]
fn test_skip_deserializing_overrides_deserialize_with_tuple_variant() {
    let yaml = indoc! {"
        ---
        - hello
        - 42
    "};

    let result = yaml_serde::from_str::<TupleVariantSkipAndDeserializeWith>(yaml);

    assert!(matches!(
        result,
        Ok(TupleVariantSkipAndDeserializeWith::Selected(x, y))
            if x == "hello" && y == 42
    ));
}

fn deserialize_tuple_variant_fails<'de, D>(deserializer: D) -> Result<(String, i32, bool), D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    struct DeHelper {
        _value: String,
    }

    let _ = DeHelper::deserialize(deserializer)?;

    Err(serde::de::Error::custom("intentional failure"))
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum FailedDeserializeWithTuple {
    First(i32),
    #[serde(deserialize_with = "deserialize_tuple_variant_fails")]
    Second(String, i32, bool),
    Third {
        x: String,
        y: i32,
    },
}

#[test]
fn test_failed_deserialize_with_followed_by_tuple_variant() {
    let yaml = indoc! {"
        ---
        x: hello
        y: 42
    "};

    let result = yaml_serde::from_str::<FailedDeserializeWithTuple>(yaml);

    assert!(matches!(
        result,
        Ok(FailedDeserializeWithTuple::Third { x, y })
            if x == "hello" && y == 42
    ));
}
