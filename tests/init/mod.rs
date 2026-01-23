use crate::{cli, zsh};

fn run_test(testname: &str, args: &[&str]) {
    let output = cli().arg("init").args(args).output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr, "");
    assert_ne!(stdout, "");
    assert!(output.status.success());

    let result = zsh()
        .arg("-c")
        .arg(stdout.as_ref())
        .status()
        .expect("Failed to execute zsh with init output");

    assert!(result.success());

    insta::assert_snapshot!(testname, stdout);
}

#[test]
fn test_init() {
    run_test("init", &[]);
    run_test("init --bind-keys", &["--bind-keys"]);
}
