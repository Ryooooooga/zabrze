use std::path::Path;

use crate::{
    config::{self, Config},
    opt::InitArgs,
};

static INIT_SCRIPT: &str = include_str!("zabrze-init.zsh");
static BIND_KEYS_SCRIPT: &str = include_str!("zabrze-bindkey.zsh");

fn warn_yaml_config_deprecation() {
    if let Some(config_dir) = config::get_default_config_dir() {
        let config_paths = Config::config_file_paths(Path::new(&config_dir))
            .expect("failed to read config directory");

        let mut has_yaml = false;
        for path in &config_paths {
            if let Some(ext) = path.extension()
                && (ext == "yaml" || ext == "yml")
            {
                has_yaml = true;
                break;
            }
        }

        if has_yaml {
            eprintln!(
                "{}",
                ansi_term::Color::Yellow
                    .paint("zabrze: Warning: YAML config files are deprecated and will be removed in future versions. Please migrate to TOML format.")
            );
        }
    }
}

pub fn run(args: &InitArgs) {
    warn_yaml_config_deprecation();

    print!("{INIT_SCRIPT}");

    if args.bind_keys {
        print!("{BIND_KEYS_SCRIPT}");
    }
}
