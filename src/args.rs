use std::path::PathBuf;
use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    #[clap(short, long)]
    pub config: Option<PathBuf>,
    #[clap(short, long)]
    pub storage_dir: Option<PathBuf>
}