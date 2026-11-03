//! SigmaOS-scheduler - SigmaOS Component
//!
//! Development and system tools for SigmaOS.

use anyhow::Result;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub enabled: bool,
    pub auto_start: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self { enabled: true, auto_start: true }
    }
}

pub struct scheduler {
    config: Config,
}

impl scheduler {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    pub fn initialize(&mut self) -> Result<()> {
        info!("Initializing SigmaOS-scheduler");
        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        info!("Running SigmaOS-scheduler");
        Ok(())
    }
}
