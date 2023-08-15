use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// What mode to run the program in
    #[arg(value_enum)]
    pub mode: Role,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Role {
    /// Run as a driver.
    Driver,
    /// Run as a worker (executor).
    Worker,
}
