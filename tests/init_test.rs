mod helpers;

use crate::helpers::{cli, run_command, zsh};

fn run_test(testname: &str, args: &[&str]) {
    let stdout = run_command(cli().args(args));
    assert_ne!(stdout, "");

    assert_eq!(run_command(zsh().args(&["-c", &stdout])), "");
    insta::assert_snapshot!(testname, stdout);
}

#[test]
fn test_init() {
    run_test("init", &["init"]);
    run_test("init --bind-keys", &["init", "--bind-keys"]);
}
