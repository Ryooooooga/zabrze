mod init;

use std::{env, process::Command};

pub fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_zabrze"))
}

pub fn zsh() -> Command {
    Command::new("zsh")
}
