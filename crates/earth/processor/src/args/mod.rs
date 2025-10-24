use clap::Parser;

pub mod command;
mod resolution;

pub use command::EarthCommand;
pub use resolution::*;

#[derive(Parser)]
#[clap(version, about)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: EarthCommand,
}

pub fn default_resolutions() -> Vec<ImageResolution> {
    vec![
        ImageResolution {
            width: 2700,
            height: 1350,
        },
        ImageResolution {
            width: 5400,
            height: 2700,
        },
        ImageResolution {
            width: 8100,
            height: 4050,
        },
        ImageResolution {
            width: 10800,
            height: 5400,
        },
        ImageResolution {
            width: 21600,
            height: 10800,
        },
    ]
}
