use deserialize_untagged_verbose_error::DeserializeUntaggedVerboseError;
use indoc::indoc;

#[derive(Debug, DeserializeUntaggedVerboseError)]
#[allow(dead_code)]
enum HasHelperStruct {
    Foo { x: String, y: i32 },
    Bar(i32),
    Baz { x: String, z: i32 },
}

#[test]
fn test_substitute_helper_struct_name() {
    let yaml = indoc! {"
        ---
        hello
    "};

    let result = yaml_serde::from_str::<HasHelperStruct>(yaml).unwrap_err();
    let msg = result.to_string();

    assert!(msg.contains("expected struct Foo"));
    assert!(!msg.contains("__DeserializeUntaggedVerboseError_HasHelperStruct_Foo"));

    assert!(msg.contains("expected struct Baz"));
    assert!(!msg.contains("__DeserializeUntaggedVerboseError_HasHelperStruct_Baz"));
}
