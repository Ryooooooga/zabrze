use crate::opt::InitArgs;

pub fn run(_args: &InitArgs) {
    let init_script = include_str!("zabrze.zsh");

    print!("{}", init_script);
}
