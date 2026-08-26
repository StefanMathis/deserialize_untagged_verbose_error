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
