use clap::Parser;
use clap::Subcommand;
use std::ffi::OsString;

#[derive(Debug, Parser)]
#[command(name = "youre-muted-btw", bin_name = "youre-muted-btw")]
pub struct Args {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Option<Command>,
}
impl Args {
    pub fn as_args(self) -> Vec<OsString> {
        let mut rtn = Vec::new();
        if self.global.debug {
            rtn.push("--debug".into());
        }
        if let Some(command) = self.command {
            rtn.push(command.to_string().into())
        }
        rtn
    }
}

#[derive(Debug, Subcommand, strum::Display)]
#[strum(serialize_all = "kebab-case")]
pub enum Command {
    Tray,
    WelcomeGui,
}

#[derive(Debug, Parser, Clone)]
pub struct GlobalArgs {
    /// Enable debug logging
    #[arg(long, global = true, default_value = "false")]
    pub debug: bool,
}
