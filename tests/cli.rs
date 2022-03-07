use assert_cmd::Command;

#[test]
fn verify_behavior() {
    behavior_for("simple");
    behavior_for("nested");
    behavior_for("attributes");
    behavior_for("markdown");
    behavior_for("if_conditional");
    behavior_for("if_else_conditional");
    behavior_for("interpolation");
    behavior_for("attribute_interpolation");
    behavior_for("for_loops");
}

fn behavior_for(test: &str) {
    let mut cmd = Command::cargo_bin("socket").unwrap();

    cmd.arg("--context");
    cmd.arg("tests/regression/context.json");

    let (input, output) = load_data(test);
    cmd.write_stdin(input);

    cmd.assert().success().stdout(output);
}

fn load_data(test: &str) -> (String, String) {
    let input = std::fs::read_to_string(format!("tests/regression/{}.skt", test)).unwrap();
    let output = std::fs::read_to_string(format!("tests/regression/{}.html", test)).unwrap();

    (input, output)
}
