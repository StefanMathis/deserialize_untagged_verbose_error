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

        let de_enum: DeEnum<f64, String> = serde_yaml::from_str(yaml).unwrap();
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

        let de_enum: DeEnum<f64, String> = serde_yaml::from_str(yaml).unwrap();
        match de_enum {
            DeEnum::Variant1(_) => panic!("test failed"),
            DeEnum::Variant2(var) => {
                assert_eq!(var.field, "Test".to_string());
            }
        }
    }
}
