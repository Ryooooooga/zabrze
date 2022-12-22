use crate::opt::InitArgs;

static INIT_SCRIPT: &str = include_str!("zabrze-init.zsh");
static BIND_KEYS_SCRIPT: &str = include_str!("zabrze-bindkey.zsh");

pub fn run(args: &InitArgs) {
    print!("{INIT_SCRIPT}");

    if args.bind_keys {
        print!("{BIND_KEYS_SCRIPT}");
    }
}
