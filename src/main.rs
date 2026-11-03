use anyhow::Result;
use clap::Parser;
use scheduler::scheduler;

#[derive(Parser)]
struct Cli {
    #[arg(short, long)]
    auto: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = scheduler::Config { auto_start: cli.auto, ..Default::default() };
    let mut component = scheduler::new(config);
    
    component.initialize()?;
    component.run()?;
    
    println!("SigmaOS-scheduler operational");
    Ok(())
}
