use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser, Clone)]
pub struct Args {
    #[clap(short, long)]
    pub config: Option<PathBuf>,
    #[clap(long)]
    pub dry_run: bool,
}
