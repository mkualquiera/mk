use clap::Parser;
use log::{error, info};
use making::make;
use simple_logger::SimpleLogger;

mod making;
mod mkfile;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about=None)]
#[command(propagate_version = true)]
struct Cli {
    /// Path to the mkfile to use.
    #[arg(short, long, default_value = "mkfile")]
    mkfile: String,
    /// Path to the update state file to use.
    #[arg(short, long, default_value = ".mkstate.sexpr")]
    state: String,
    /// The target to make
    #[arg(default_value = "all")]
    target: String,
}

fn main() {
    SimpleLogger::new().init().unwrap();
    let cli = Cli::parse();

    // Parse the mkfile
    let text = std::fs::read_to_string(cli.mkfile).expect("Failed to read mkfile");
    let mkfile = mkfile::MkFile::parse(&text);

    // Load the state
    let mut state = match std::fs::read_to_string(&cli.state) {
        Ok(text) => serde_sexpr::from_str(&text).expect("Failed to parse state"),
        Err(_) => making::UpdateState::default(),
    };

    // Make the target
    let mut target = mkfile::Target::parse(&cli.target);
    if !mkfile.has_target(&target) {
        target = mkfile::Target::Virtual(cli.target);
    }

    let made = make(&mkfile, &target, &mut state);

    // Save the state
    let text = serde_sexpr::to_string(&state).expect("Failed to serialize state");
    std::fs::write(&cli.state, text).expect("Failed to write state");

    match made {
        Ok(made) => {
            if made {
                info!("Made target '{:?}'", target);
            } else {
                info!("Target '{:?}' is up to date", target);
            }
        }
        Err(err) => {
            error!("Failed to make target '{:?}': {}", target, err);
            std::process::exit(1);
        }
    }
}
