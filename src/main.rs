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
            let actual = ops::compress::run(&output, &inputs, &format, level, verbose)?;
            println!("created {}", actual.display());
        }
        Command::Extract {
            archive,
            output,
            format,
            verbose,
        } => {
            let dest = output.unwrap_or_else(|| {
                let name = archive
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "extracted".to_string());
                std::path::PathBuf::from(format::strip_archive_suffix(&name))
            });
            ops::extract::run(&archive, Some(&dest), &format, verbose)?;
            println!("extracted {} -> {}", archive.display(), dest.display());
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
        Command::Squeeze {
            inputs,
            output,
            format,
            quality,
        } => {
            ops::squeeze::run(&inputs, &output, &format, quality)?;
        }
        Command::Update { check } => {
            run_update(check)?;
        }
        Command::Uninstall { purge, force } => {
            run_uninstall(purge, force)?;
        }
    }
    Ok(())
}

fn run_update(check_only: bool) -> Result<()> {
    use std::io::{self, Write as _};
    let current = env!("CARGO_PKG_VERSION");

    let url = "https://api.github.com/repos/podsni/epax/releases/latest";
    let output = std::process::Command::new("curl")
        .args(["-sSf", "-H", "User-Agent: epax", url])
        .output();

    let latest = match output {
        Ok(o) if o.status.success() => {
            let body = String::from_utf8_lossy(&o.stdout);
            body.split('"')
                .skip_while(|s| *s != "tag_name")
                .nth(2)
                .map(|s| s.trim_start_matches('v').to_string())
                .unwrap_or_default()
        }
        _ => {
            eprintln!("warning: could not reach GitHub API — check your internet connection");
            return Ok(());
        }
    };

    if latest.is_empty() {
        eprintln!("warning: could not parse latest version from GitHub API response");
        return Ok(());
    }

    if latest == current {
        println!("epax {current} is already the latest version.");
        return Ok(());
    }

    println!("epax {current} → {latest} available");

    if check_only {
        println!("Run `epax update` (without --check) to install.");
        return Ok(());
    }

    println!("Updating to v{latest}…");
    print!("Proceed with update? [y/N] ");
    io::stdout().flush().ok();
    let mut ans = String::new();
    io::stdin().read_line(&mut ans).ok();
    if !ans.trim().eq_ignore_ascii_case("y") {
        println!("Aborted. To update manually:");
        #[cfg(unix)]
        println!(
            "  curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/install.sh | bash"
        );
        #[cfg(windows)]
        println!(
            "  iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/install.ps1 | iex"
        );
        return Ok(());
    }

    #[cfg(unix)]
    {
        let cmd = format!(
            "VERSION=v{latest} curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/install.sh | bash"
        );
        let status = std::process::Command::new("sh").args(["-c", &cmd]).status();
        match status {
            Ok(s) if s.success() => println!("Updated to epax {latest}."),
            _ => eprintln!(
                "Update failed. Install manually:\n  curl -sSL https://raw.githubusercontent.com/podsni/epax/main/scripts/install.sh | bash"
            ),
        }
    }
    #[cfg(windows)]
    {
        let script = format!(
            "iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/install.ps1 -OutFile $env:TEMP\\epax_install.ps1; & $env:TEMP\\epax_install.ps1 -Version v{latest}"
        );
        let status = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &script])
            .status();
        match status {
            Ok(s) if s.success() => println!("Updated to epax {latest}."),
            _ => eprintln!(
                "Update failed. Install manually:\n  iwr -useb https://raw.githubusercontent.com/podsni/epax/main/scripts/install.ps1 | iex"
            ),
        }
    }

    Ok(())
}

fn run_uninstall(purge: bool, force: bool) -> Result<()> {
    use std::io::{self, Write as _};

    let bin_path = std::env::current_exe().map_err(crate::error::EpaxError::Io)?;

    if !force {
        print!("Uninstall epax at {}? [y/N] ", bin_path.display());
        io::stdout().flush().ok();
        let mut ans = String::new();
        io::stdin().read_line(&mut ans).ok();
        if !ans.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    #[cfg(unix)]
    {
        std::fs::remove_file(&bin_path).map_err(crate::error::EpaxError::Io)?;
        println!("Removed {}", bin_path.display());
    }
    #[cfg(windows)]
    {
        let bat = std::env::temp_dir().join("epax_uninstall.bat");
        let bin_str = bin_path.to_string_lossy();
        let script = format!(
            "@echo off\r\nping 127.0.0.1 -n 3 >nul\r\ndel /f /q \"{bin_str}\"\r\necho epax uninstalled.\r\n"
        );
        std::fs::write(&bat, script).map_err(crate::error::EpaxError::Io)?;
        std::process::Command::new("cmd")
            .args(["/c", "start", "/min", bat.to_str().unwrap_or("")])
            .spawn()
            .map_err(crate::error::EpaxError::Io)?;
        println!("Uninstall scheduled (binary will be removed shortly).");
    }

    if purge {
        for d in [dirs_config(), dirs_data()].into_iter().flatten() {
            if d.exists() {
                let _ = std::fs::remove_dir_all(&d);
                println!("Removed {}", d.display());
            }
        }
    }

    println!("epax uninstalled.");
    Ok(())
}

fn dirs_config() -> Option<std::path::PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config/epax"))
    }
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(|a| std::path::PathBuf::from(a).join("epax"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

fn dirs_data() -> Option<std::path::PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".local/share/epax"))
    }
    #[cfg(windows)]
    {
        std::env::var_os("LOCALAPPDATA").map(|a| std::path::PathBuf::from(a).join("epax"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}
