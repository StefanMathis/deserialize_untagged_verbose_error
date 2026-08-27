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

fn deserialize_generic_tuple<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    let value = T::deserialize(deserializer)?;
    Ok(value)
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum GenericTuple<T> {
    #[serde(deserialize_with = "deserialize_generic_tuple")]
    Value(T),
}

#[test]
fn test_deserialize_with_generic_tuple_variant() {
    let yaml = indoc! {"
        ---
        42
    "};

    let result = yaml_serde::from_str::<GenericTuple<i32>>(yaml);

    assert!(matches!(result, Ok(GenericTuple::Value(42))));
}

fn deserialize_generic_struct<'de, D, T>(deserializer: D) -> Result<GenericStructValue<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    #[derive(Deserialize)]
    struct DeHelper<T> {
        value: T,
    }

    let helper = DeHelper::deserialize(deserializer)?;

    Ok(GenericStructValue {
        value: helper.value,
    })
}

#[derive(Debug)]
#[allow(dead_code)]
struct GenericStructValue<T> {
    value: T,
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum GenericStruct<T> {
    #[serde(deserialize_with = "deserialize_generic_struct")]
    Value { value: T },
}

#[test]
fn test_deserialize_with_generic_struct_variant() {
    let yaml = indoc! {"
        ---
        value: 42
    "};

    let result = yaml_serde::from_str::<GenericStruct<i32>>(yaml);

    assert!(matches!(result, Ok(GenericStruct::Value { value: 42 })));
}

struct NotDeserialize;

impl<'de> serde::Deserialize<'de> for NotDeserialize {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _: String = serde::Deserialize::deserialize(deserializer)?;
        Ok(Self)
    }
}

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum GenericBoundedTuple<T> {
    #[serde(bound(deserialize = "T: serde::Deserialize<'de>"))]
    Value(T),
}

#[test]
fn test_bound_with_generic_tuple_variant() {
    let yaml = indoc! {"
        ---
        hello
    "};

    let result = yaml_serde::from_str::<GenericBoundedTuple<NotDeserialize>>(yaml);

    assert!(matches!(result, Ok(GenericBoundedTuple::Value(_))));
}

// TODO: This currently doesn't work, maybe future work if someone really needs it?
// trait CustomDeserialize<'de>: Sized {
//     fn deserialize_custom<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>;
// }

// struct NotDeserializable {
//     value: String,
// }

// impl<'de> CustomDeserialize<'de> for NotDeserializable {
//     fn deserialize_custom<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let value = String::deserialize(deserializer)?;
//         Ok(Self { value })
//     }
// }

// fn deserialize_custom<'de, D, T>(deserializer: D) -> Result<T, D::Error>
// where
//     D: serde::Deserializer<'de>,
//     T: CustomDeserialize<'de>,
// {
//     T::deserialize_custom(deserializer)
// }

// #[derive(Debug, DeserializeUntaggedVerboseError)]
// #[allow(dead_code)]
// enum GenericBoundedWithCustomDeserializer<T> {
//     #[serde(
//         deserialize_with = "deserialize_custom",
//         bound(deserialize = "T: CustomDeserialize<'de>")
//     )]
//     Value(T),
// }

// #[test]
// fn test_bound_with_deserialize_with() {
//     let yaml = indoc! {"
//         ---
//         hello
//     "};

//     let result =
//         yaml_serde::from_str::<GenericBoundedWithCustomDeserializer<NotDeserializable>>(yaml);

//     assert!(matches!(
//         result,
//         Ok(GenericBoundedWithCustomDeserializer::Value(value))
//             if value.value == "hello"
//     ));
// }
