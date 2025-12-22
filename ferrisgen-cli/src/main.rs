use anyhow::Result;
use clap::Parser;
use ferrisgen::{generate, Options};

/// Simple password generator CLI
#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Password length (6-25)
    #[arg(long, default_value_t = 12)]
    length: usize,

    /// Include lowercase characters
    #[arg(short = 'l', long = "lowercase", action = clap::ArgAction::SetTrue)]
    lowercase: bool,

    /// Include uppercase characters
    #[arg(short = 'u', long = "uppercase", action = clap::ArgAction::SetTrue)]
    uppercase: bool,

    /// Include numbers
    #[arg(short = 'n', long = "numbers", action = clap::ArgAction::SetTrue)]
    numbers: bool,

    /// Include special characters
    #[arg(short = 's', long = "special", action = clap::ArgAction::SetTrue)]
    special: bool,

    /// Exclude ambiguous characters like O and 0, I and l
    #[arg(short = 'a', long = "no-ambiguous", action = clap::ArgAction::SetTrue)]
    no_ambiguous: bool,

    /// Copy generated password to clipboard
    #[arg(long, action = clap::ArgAction::SetTrue)]
    copy: bool,
}

fn main() -> Result<()> {
    // If no arguments supplied, print an error and exit
    if std::env::args_os().nth(1).is_none() {
        eprintln!("Error: no arguments supplied. Use --help to see available options.");
        std::process::exit(1);
    }

    let args = Args::parse();

    // If no inclusion flags are provided, default to all on (lowercase, uppercase, numbers)
    let any_flag = args.lowercase || args.uppercase || args.numbers || args.special;

    let opts = Options {
        length: args.length,
        lowercase: if any_flag { args.lowercase } else { true },
        uppercase: if any_flag { args.uppercase } else { true },
        numbers: if any_flag { args.numbers } else { true },
        special: if any_flag { args.special } else { false },
        no_ambiguous: args.no_ambiguous,
    };

    let password = generate(&opts)?;

    println!("{}", password);

    if args.copy {
        if let Err(e) = copy_to_clipboard(&password) {
            eprintln!("Warning: failed to copy to clipboard: {}", e);
        }
    }

    Ok(())
}

fn copy_to_clipboard(s: &str) -> Result<()> {
    let mut ctx = arboard::Clipboard::new()?;
    ctx.set_text(s.to_owned())?;
    Ok(())
}