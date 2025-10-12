use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use serde::Deserialize;

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

#[test]
fn test_deserialize() {
    {
        let yaml = indoc! {"
        ---
        field1: 1.0
        field2: 2.0
        "};

        // Without serde try_from ==> Two steps
        let de_enum: DeEnum = serde_yaml::from_str(yaml).unwrap();
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
        let de_enum: DeEnum = serde_yaml::from_str(yaml).unwrap();
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

    // Just here to provide a payload to test against - but the macro works with
    // any serde-supported format
    use serde_yaml;

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
    let err_serde = serde_yaml::from_str::<VarSerde>(test_str).unwrap_err();
    let err_deunvveer = serde_yaml::from_str::<VarDeUnVeEr>(test_str).unwrap_err();

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
