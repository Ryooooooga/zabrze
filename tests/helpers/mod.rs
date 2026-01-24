use std::{env, process::Command};

pub fn cli() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_zabrze"));
    cmd.env_clear();
    cmd
}

pub fn zsh() -> Command {
    let mut cmd = Command::new("zsh");
    cmd.env_clear();
    cmd
}

pub fn run_command(cmd: &mut Command) -> String {
    let (stdout, stderr) = run_command_outputs(cmd);
    assert_eq!(stderr, "");
    stdout
}

pub fn run_command_outputs(cmd: &mut Command) -> (String, String) {
    let output = cmd.output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        panic!("command {cmd:?} failed with stderr: {stderr}",);
    }

    (stdout.to_string(), stderr.to_string())
}
