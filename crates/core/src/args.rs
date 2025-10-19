use bevy_log::DEFAULT_FILTER;
use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct CliArgs {
    #[arg(
        long,
        default_value_t = bevy_log::Level::INFO,
    )]
    pub log_level: bevy_log::Level,
    #[arg(long, short, default_value_t = DEFAULT_FILTER.into())]
    pub log_filter: String,
}