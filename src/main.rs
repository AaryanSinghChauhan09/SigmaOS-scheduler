use std::env;
use log::{info, warn, error, debug};
use anyhow::Result;
use clap::Parser;
use scheduler::scheduler;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    auto: bool,
}

fn main() -> Result<()> { -> Result<()> {
    let cli = Cli::parse();
    let config = scheduler::Config { auto_start: cli.auto, ..Default::default() };
    let mut component = scheduler::new(config);
    
    component.initialize()?;
    component.run()?;
    
    println!("SigmaOS-scheduler operational");
    Ok(())
}

#[derive(Debug, Clone)]
struct Config {
    verbose: bool,
    dry_run: bool,
}

impl Config {
    fn from_env() -> Self {
        Config {
            verbose: env::var("VERBOSE").is_ok(),
            dry_run: env::var("DRY_RUN").is_ok(),
        }
    }
}
