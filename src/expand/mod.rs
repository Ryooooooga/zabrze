use crate::config::Config;
use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

pub fn run(args: &ExpandArgs) {
    expand(args, &Config::load_or_exit())
}

fn expand(args: &ExpandArgs, config: &Config) {
    let lbuffer = &args.lbuffer;
    let rbuffer = &args.rbuffer;

    let buffer = lbuffer.to_string() + rbuffer;
    let cursor = lbuffer.chars().count();

    println!(r"BUFFER={};CURSOR={}", escape(Cow::from(buffer)), cursor);
}
