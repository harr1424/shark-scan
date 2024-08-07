use log::{LevelFilter};
use clap::{CommandFactory, FromArgMatches};
use shark_scan::{parser::Args, scanner::scan};

#[tokio::main]
async fn main() {
    let command = Args::command().arg_required_else_help(true);
    let matches = command.get_matches();
    let args = Args::from_arg_matches(&matches).expect("Failed to parse arguments");

    match args.verbosity.as_str() {
        "none" => env_logger::builder().filter_level(LevelFilter::Error).init(),
        "low" => env_logger::builder().filter_level(LevelFilter::Info).init(),
        "high" => env_logger::builder().filter_level(LevelFilter::Trace).init(),
        _ => env_logger::builder().filter_level(LevelFilter::Error).init(),
    }

    scan(args).await;
}
