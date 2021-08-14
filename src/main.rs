mod expand;
mod init;
mod opt;

use opt::{Opt, Subcommand};

fn main() {
    let opt = Opt::parse();

    match &opt.subcommand {
        Subcommand::Init(args) => init::run(args),
        Subcommand::Expand(args) => expand::run(args),
    }
}
