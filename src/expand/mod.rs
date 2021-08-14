use crate::opt::ExpandArgs;
use shell_escape::escape;
use std::borrow::Cow;

pub fn run(args: &ExpandArgs) {
    let lbuffer = &args.lbuffer;
    let rbuffer = &args.rbuffer;

    let buffer = lbuffer.to_string() + rbuffer;
    let cursor = lbuffer.chars().count();

    println!(r"BUFFER={};CURSOR={}", escape(Cow::from(buffer)), cursor);
}
