use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;
use serde::Deserialize;

#[derive(Deserialize)]
struct Variant1<F> {
    field1: F,
    field2: F,
}

#[derive(Deserialize)]
struct VariantWithOtherName<T> {
    field: T,
}

#[derive(DeserializeUntaggedVerboseError)]
enum DeEnum<F, T> {
    Variant1(Variant1<F>),
    Variant2(VariantWithOtherName<T>),
}

#[test]
fn test_deserialize() {
    {
        let yaml = indoc! {"
        ---
        field1: 1.0
        field2: 2.0
        "};

        let de_enum: DeEnum<f64, String> = yaml_serde::from_str(yaml).unwrap();
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
        field: Test
        "};

        let de_enum: DeEnum<f64, String> = yaml_serde::from_str(yaml).unwrap();
        match de_enum {
            DeEnum::Variant1(_) => panic!("test failed"),
            DeEnum::Variant2(var) => {
                assert_eq!(var.field, "Test".to_string());
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct Foo<T> {
    value: T,
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum GenericNamed<T> {
    Foo { value: T },
}

#[test]
fn test_generic_named_variant() {
    {
        let yaml = indoc! {"
        ---
        value: 42
        "};

        let value: GenericNamed<i32> = yaml_serde::from_str(yaml).unwrap();

        match value {
            GenericNamed::Foo { value } => {
                assert_eq!(value, 42);
            }
        }
    }

    {
        let yaml = indoc! {"
        ---
        value: hello
        "};

        let value: GenericNamed<String> = yaml_serde::from_str(yaml).unwrap();

        match value {
            GenericNamed::Foo { value } => {
                assert_eq!(value, "hello");
            }
        }
    }

    {
        let yaml = indoc! {"
        ---
        value: 3.141
        "};

        let value: GenericNamed<f64> = yaml_serde::from_str(yaml).unwrap();

        match value {
            GenericNamed::Foo { value } => {
                assert_eq!(value, 3.141);
            }
        }
    }
}

#[test]
fn test_generic_named_variant_matches_struct() {
    let yaml = indoc! {"
    ---
    value: 42
    "};

    let foo: Foo<i32> = yaml_serde::from_str(yaml).unwrap();

    let generic_named: GenericNamed<i32> = yaml_serde::from_str(yaml).unwrap();

    assert_eq!(foo.value, 42);

    match generic_named {
        GenericNamed::Foo { value } => {
            assert_eq!(value, foo.value);
        }
    }
}

#[test]
fn test_generic_named_variant_type_mismatch() {
    let yaml = indoc! {"
    ---
    value: hello
    "};

    let err_msg = yaml_serde::from_str::<GenericNamed<i32>>(yaml).unwrap_err();

    assert_eq!(
        err_msg.to_string(),
        indoc! {"
        Failed to deserialize the untagged enum GenericNamed:
        - Could not deserialize as Foo: invalid type: string \"hello\", expected i32.
        "}
    );
}

#[test]
fn test_generic_named_variant_with_extra_field() {
    let yaml = indoc! {"
    ---
    value: 42
    extra: ignored
    "};

    let value: GenericNamed<i32> = yaml_serde::from_str(yaml).unwrap();

    match value {
        GenericNamed::Foo { value } => {
            assert_eq!(value, 42);
        }
    }
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
enum GenericNamedMultiple<T, U> {
    Foo { first: T, second: U },
}

#[test]
fn test_generic_named_multiple_fields() {
    let yaml = indoc! {"
    ---
    first: 42
    second: hello
    "};

    let value: GenericNamedMultiple<i32, String> = yaml_serde::from_str(yaml).unwrap();

    match value {
        GenericNamedMultiple::Foo { first, second } => {
            assert_eq!(first, 42);
            assert_eq!(second, "hello");
        }
    }
}
