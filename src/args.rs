use clap::Parser;
use std::path::PathBuf;

/// Program for testing your orthography knowledge
#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    /// List of tort scripts to run
    #[arg(required=true, num_args(1..))]
    pub files: Vec<PathBuf>,

    /// Check all tests for correctness (it ignores `-n`)
    #[arg(short, long)]
    pub check: bool,

    /// Shuffle all test statements
    #[arg(short, long)]
    pub random: bool,

    /// how many tests you want to pass (0 means every test)
    #[arg(short, long)]
    pub number_of_tests: Option<usize>
}
