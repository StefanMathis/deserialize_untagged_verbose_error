use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use serde::Deserialize;

#[test]
fn test_compare_to_serde_untagged() {
    #[derive(Deserialize)]
    struct Variant1 {
        field1: f64,
        field2: f64,
    }

    #[derive(Deserialize)]
    struct VariantWithOtherName {
        field: f64,
    }

    #[derive(DeserializeUntaggedVerboseError)]
    enum DeEnum {
        Variant1(Variant1),
        Variant2(VariantWithOtherName),
    }
    {
        let yaml = indoc! {"
        ---
        field1: 1.0
        field2: 2.0
        "};

        // Without serde try_from ==> Two steps
        let de_enum: DeEnum = yaml_serde::from_str(yaml).unwrap();
        match de_enum {
            DeEnum::Variant1(var1) => {
                assert_eq!(var1.field1, 1.0);
                assert_eq!(var1.field2, 2.0);
            }
            DeEnum::Variant2(_) => panic!("test failed"),
        }
    }
    {
        let yaml = indoc! {"
        ---
        field: 1.0
        "};

        // Without serde try_from ==> Two steps
        let de_enum: DeEnum = yaml_serde::from_str(yaml).unwrap();
        match de_enum {
            DeEnum::Variant1(_) => panic!("test failed"),
            DeEnum::Variant2(var) => {
                assert_eq!(var.field, 1.0);
            }
        }
    }
}

#[test]
fn test_error_message() {
    use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
    use serde::Deserialize;

    // Random structs used as variant of the enum
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Point {
        x: f64,
        y: f64,
    }
    #[derive(Debug, Deserialize)]
    #[allow(dead_code)]
    struct Message {
        epochtime: usize,
        content: String,
    }

    // Standard Serde approach
    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    #[allow(dead_code)]
    enum VarSerde {
        Message(Message),
        Point(Point),
        Value(f64),
    }

    // Using the macro provided by this crate
    #[derive(Debug, DeserializeUntaggedVerboseError)]
    #[allow(dead_code)]
    enum VarDeUnVeEr {
        Message(Message),
        Point(Point),
        Value(f64),
    }

    let test_str = indoc! {"
    ---
    name: Serde
    "};

    // Deserializing "test_str" fails, because it does not match any variant of
    // VarSerde / VarDeUnVeEr
    let err_serde = yaml_serde::from_str::<VarSerde>(test_str).unwrap_err();
    let err_deunvveer = yaml_serde::from_str::<VarDeUnVeEr>(test_str).unwrap_err();

    // Compare the error messages:
    assert_eq!(
        err_serde.to_string(),
        "data did not match any variant of untagged enum VarSerde"
    );
    assert_eq!(
        err_deunvveer.to_string(),
        indoc! {"
        Failed to deserialize the untagged enum VarDeUnVeEr:
        - Could not deserialize as Message: missing field `epochtime`.
        - Could not deserialize as Point: missing field `x`.
        - Could not deserialize as Value: invalid type: map, expected f64.
    "}
    );
}

#[test]
fn test_tuple_variants() {
    #[derive(Debug, DeserializeUntaggedVerboseError)]
    enum TupleEnum {
        Pair(i32, String),
        Triple(String, f64, bool),
    }
    {
        let yaml = indoc! {"
        ---
        - 42
        - hello
        "};

        let value: TupleEnum = yaml_serde::from_str(yaml).unwrap();

        match value {
            TupleEnum::Pair(number, text) => {
                assert_eq!(number, 42);
                assert_eq!(text, "hello");
            }
            _ => panic!("test failed"),
        }
    }

    {
        let yaml = indoc! {"
        ---
        - hello
        - 3.14
        - true
        "};

        let value: TupleEnum = yaml_serde::from_str(yaml).unwrap();

        match value {
            TupleEnum::Triple(text, number, flag) => {
                assert_eq!(text, "hello");
                assert_eq!(number, 3.14);
                assert!(flag);
            }
            _ => panic!("test failed"),
        }
    }

    {
        let yaml = indoc! {"
        ---
        "};

        let err_msg = yaml_serde::from_str::<TupleEnum>(yaml).unwrap_err();

        assert_eq!(
            err_msg.to_string(),
            indoc! {"
            Failed to deserialize the untagged enum TupleEnum:
            - Could not deserialize as Pair: invalid type: unit value, expected a tuple of size 2.
            - Could not deserialize as Triple: invalid type: unit value, expected a tuple of size 3.
        "}
        );
    }
}

#[test]
fn test_tuple_variants_with_fallback() {
    #[derive(Debug, DeserializeUntaggedVerboseError)]
    enum TupleEnumFallback {
        Pair(i32, String),
        Triple(String, f64, bool),
        Fallback,
    }
    {
        let yaml = indoc! {"
        ---
        - 42
        - hello
        "};

        let value: TupleEnumFallback = yaml_serde::from_str(yaml).unwrap();

        match value {
            TupleEnumFallback::Pair(number, text) => {
                assert_eq!(number, 42);
                assert_eq!(text, "hello");
            }
            _ => panic!("test failed"),
        }
    }

    {
        let yaml = indoc! {"
        ---
        - hello
        - 3.14
        - true
        "};

        let value: TupleEnumFallback = yaml_serde::from_str(yaml).unwrap();

        match value {
            TupleEnumFallback::Triple(text, number, flag) => {
                assert_eq!(text, "hello");
                assert_eq!(number, 3.14);
                assert!(flag);
            }
            _ => panic!("test failed"),
        }
    }

    {
        let yaml = indoc! {"
        ---
        "};

        let value: TupleEnumFallback = yaml_serde::from_str(yaml).unwrap();

        match value {
            TupleEnumFallback::Fallback => (),
            _ => panic!("test failed"),
        }
    }

    {
        let yaml = indoc! {"
        ---
        - 1
        "};

        let err_msg = yaml_serde::from_str::<TupleEnumFallback>(yaml).unwrap_err();

        assert_eq!(
            err_msg.to_string(),
            indoc! {"
            Failed to deserialize the untagged enum TupleEnumFallback:
            - Could not deserialize as Pair: invalid length 1, expected a tuple of size 2.
            - Could not deserialize as Triple: invalid length 1, expected a tuple of size 3.
            - Could not deserialize as Fallback: invalid type: sequence, expected unit.
        "}
        );
    }

    {
        let yaml = indoc! {"
        ---
        - 3.141
        "};

        let err_msg = yaml_serde::from_str::<TupleEnumFallback>(yaml).unwrap_err();

        assert_eq!(
            err_msg.to_string(),
            indoc! {"
            Failed to deserialize the untagged enum TupleEnumFallback:
            - Could not deserialize as Pair: invalid length 1, expected a tuple of size 2.
            - Could not deserialize as Triple: invalid length 1, expected a tuple of size 3.
            - Could not deserialize as Fallback: invalid type: sequence, expected unit.
        "}
        );
    }

    {
        let yaml = indoc! {"
        ---
        - 1
        - 3.141
        "};

        let err_msg = yaml_serde::from_str::<TupleEnumFallback>(yaml).unwrap_err();

        assert_eq!(
            err_msg.to_string(),
            indoc! {"
            Failed to deserialize the untagged enum TupleEnumFallback:
            - Could not deserialize as Pair: invalid type: floating point `3.141`, expected a string.
            - Could not deserialize as Triple: invalid length 2, expected a tuple of size 3.
            - Could not deserialize as Fallback: invalid type: sequence, expected unit.
        "}
        );
    }

    {
        let yaml = indoc! {"
        ---
        - 42
        - hello
        - This is too much
        "};

        let err_msg = yaml_serde::from_str::<TupleEnumFallback>(yaml).unwrap_err();

        assert_eq!(
            err_msg.to_string(),
            indoc! {"
            Failed to deserialize the untagged enum TupleEnumFallback:
            - Could not deserialize as Pair: invalid length 3, expected a tuple of size 2.
            - Could not deserialize as Triple: invalid type: integer `42`, expected a string.
            - Could not deserialize as Fallback: invalid type: sequence, expected unit.
        "}
        );
    }
}
