mod backends;
mod cli;
mod collect;
mod error;
mod format;
mod ops;
mod util;

use std::process::ExitCode;

use clap::Parser;

use cli::{Cli, Command};
use error::Result;

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(e.exit_code() as u8)
        }
    }
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Compress {
            output,
            inputs,
            format,
            level,
            verbose,
        } => {
            ops::compress::run(&output, &inputs, &format, level, verbose)?;
            println!("created {}", output.display());
        }
        Command::Extract {
            archive,
            output,
            format,
            verbose,
        } => {
            ops::extract::run(&archive, &output, &format, verbose)?;
            println!("extracted {} -> {}", archive.display(), output.display());
        }
        Command::List { archive, format } => {
            let entries = ops::list::run(&archive, &format)?;
            for e in &entries {
                println!("{:>12}  {}", e.size, e.name);
            }
            println!(
                "{} entr{}",
                entries.len(),
                if entries.len() == 1 { "y" } else { "ies" }
            );
        }
    }
    Ok(())
}
