#[derive(Debug, clap::Parser)]
#[command(version, disable_version_flag = true, author, about)]
pub struct Opt {
    #[arg(short, long, help = "Print version information", action=clap::ArgAction::Version)]
    pub version: Option<bool>,

    #[command(subcommand)]
    pub subcommand: Subcommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    #[command(about = "Initialize the plugin")]
    Init(InitArgs),

    #[command(about = "List abbreviations")]
    List(ListArgs),

    #[command(about = "Expand abbreviation")]
    Expand(ExpandArgs),
}

#[derive(Debug, clap::Args)]
pub struct InitArgs {
    #[arg(help = "Enable default key bindings", long)]
    pub bind_keys: bool,
}

#[derive(Debug, clap::Args)]
pub struct ListArgs {}

#[derive(Debug, clap::Args)]
pub struct ExpandArgs {
    #[arg(help = "$LBUFFER", long, short = 'l')]
    pub lbuffer: String,

    #[arg(help = "$RBUFFER", long, short = 'r')]
    pub rbuffer: String,
}
