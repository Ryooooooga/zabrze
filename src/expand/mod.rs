use crate::config::{default_config_path, Config};
use crate::opt::ExpandArgs;
use ansi_term::Color;
use shell_escape::escape;
use std::borrow::Cow;

pub fn run(args: &ExpandArgs) {
    let config_path = &default_config_path().expect("could not determine config file path");

    let config = Config::load_from_file(config_path).unwrap_or_else(|err| {
        let config_path = config_path.to_string_lossy();
        let error_message = format!("failed to load config `{}': {}", config_path, err);
        let error_style = Color::Red.normal();

        eprintln!("{}", error_style.paint(error_message));
        std::process::exit(1);
    });

    expand(args, &config)
}

fn expand(args: &ExpandArgs, config: &Config) {
    let lbuffer = &args.lbuffer;
    let rbuffer = &args.rbuffer;

    let buffer = lbuffer.to_string() + rbuffer;
    let cursor = lbuffer.chars().count();

    println!(r"BUFFER={};CURSOR={}", escape(Cow::from(buffer)), cursor);
}
