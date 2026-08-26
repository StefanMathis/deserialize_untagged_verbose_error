use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use serde::Deserialize;

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum NamedEnum {
    Variant1 { field1: f64, field2: f64 },
    Variant2 { field: f64 },
}

#[test]
fn test_named_variants() {
    {
        let yaml = indoc! {"
        ---
        field1: 1.0
        field2: 2.0
        "};

        let value: NamedEnum = yaml_serde::from_str(yaml).unwrap();

        match value {
            NamedEnum::Variant1 { field1, field2 } => {
                assert_eq!(field1, 1.0);
                assert_eq!(field2, 2.0);
            }
            _ => panic!("test failed"),
        }
    }

    {
        let yaml = indoc! {"
        ---
        field: 1.0
        "};

        let value: NamedEnum = yaml_serde::from_str(yaml).unwrap();

        match value {
            NamedEnum::Variant2 { field } => {
                assert_eq!(field, 1.0);
            }
            _ => panic!("test failed"),
        }
    }
}

#[test]
fn test_named_variants_error() {
    let yaml = indoc! {"
    ---
    other: 42.0
    "};

    let err_msg = yaml_serde::from_str::<NamedEnum>(yaml).unwrap_err();

    assert_eq!(
        err_msg.to_string(),
        indoc! {"
        Failed to deserialize the untagged enum NamedEnum:
        - Could not deserialize as Variant1: missing field `field1`.
        - Could not deserialize as Variant2: missing field `field`.
        "}
    );
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum NamedEnumOverlap {
    Variant1 { field1: f64 },
    Variant2 { field1: f64, field2: f64 },
}

#[test]
fn test_named_variants_overlap() {
    let yaml = indoc! {"
    ---
    field1: 1.0
    "};

    let value: NamedEnumOverlap = yaml_serde::from_str(yaml).unwrap();

    match value {
        NamedEnumOverlap::Variant1 { field1 } => {
            assert_eq!(field1, 1.0);
        }
        NamedEnumOverlap::Variant2 { .. } => panic!("test failed"),
    }
}

#[test]
fn test_named_variant_with_extra_fields() {
    let yaml = indoc! {"
    ---
    field: 1.0
    extra: 42.0
    "};

    let value: NamedEnum = yaml_serde::from_str(yaml).unwrap();

    match value {
        NamedEnum::Variant2 { field } => {
            assert_eq!(field, 1.0);
        }
        _ => panic!("test failed"),
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum SerdeNamedEnum {
    Variant1 { field1: f64, field2: f64 },
    Variant2 { field: f64 },
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum VerboseNamedEnum {
    Variant1 { field1: f64, field2: f64 },
    Variant2 { field: f64 },
}

#[test]
fn test_named_variants_match_serde() {
    {
        let yaml = indoc! {"
        ---
        field1: 1.0
        field2: 2.0
        "};

        let serde_value: SerdeNamedEnum = yaml_serde::from_str(yaml).unwrap();

        let verbose_value: VerboseNamedEnum = yaml_serde::from_str(yaml).unwrap();

        match serde_value {
            SerdeNamedEnum::Variant1 { field1, field2 } => {
                assert_eq!(field1, 1.0);
                assert_eq!(field2, 2.0);
            }
            SerdeNamedEnum::Variant2 { .. } => panic!("Serde selected Variant2"),
        }

        match verbose_value {
            VerboseNamedEnum::Variant1 { field1, field2 } => {
                assert_eq!(field1, 1.0);
                assert_eq!(field2, 2.0);
            }
            VerboseNamedEnum::Variant2 { .. } => {
                panic!("verbose implementation selected Variant2")
            }
        }
    }

    {
        let yaml = indoc! {"
        ---
        field: 1.0
        "};

        let serde_value: SerdeNamedEnum = yaml_serde::from_str(yaml).unwrap();

        let verbose_value: VerboseNamedEnum = yaml_serde::from_str(yaml).unwrap();

        match serde_value {
            SerdeNamedEnum::Variant1 { .. } => {
                panic!("Serde selected Variant1")
            }
            SerdeNamedEnum::Variant2 { field } => {
                assert_eq!(field, 1.0);
            }
        }

        match verbose_value {
            VerboseNamedEnum::Variant1 { .. } => {
                panic!("verbose implementation selected Variant1")
            }
            VerboseNamedEnum::Variant2 { field } => {
                assert_eq!(field, 1.0);
            }
        }
    }
}

#[test]
fn test_named_variants_with_extra_fields_match_serde() {
    let yaml = indoc! {"
    ---
    field: 1.0
    extra: 42.0
    "};

    let serde_value: SerdeNamedEnum = yaml_serde::from_str(yaml).unwrap();

    let verbose_value: VerboseNamedEnum = yaml_serde::from_str(yaml).unwrap();

    match serde_value {
        SerdeNamedEnum::Variant2 { field } => {
            assert_eq!(field, 1.0);
        }
        SerdeNamedEnum::Variant1 { .. } => {
            panic!("Serde selected Variant1")
        }
    }

    match verbose_value {
        VerboseNamedEnum::Variant2 { field } => {
            assert_eq!(field, 1.0);
        }
        VerboseNamedEnum::Variant1 { .. } => {
            panic!("verbose implementation selected Variant1")
        }
    }
}

#[test]
fn test_named_variants_failure_matches_serde() {
    let yaml = indoc! {"
    ---
    other: hello
    "};

    let serde_result = yaml_serde::from_str::<SerdeNamedEnum>(yaml);

    let verbose_result = yaml_serde::from_str::<VerboseNamedEnum>(yaml);

    assert!(serde_result.is_err());
    assert!(verbose_result.is_err());

    assert_eq!(
        verbose_result.unwrap_err().to_string(),
        indoc! {"
        Failed to deserialize the untagged enum VerboseNamedEnum:
        - Could not deserialize as Variant1: missing field `field1`.
        - Could not deserialize as Variant2: missing field `field`.
        "}
    );
}
