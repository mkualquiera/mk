use std::{collections::HashMap, error::Error, time::SystemTime};

use log::info;
use serde::{Deserialize, Serialize};

use crate::mkfile::{ConcreteTarget, MkFile, Target};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UpdateState {
    last_update: HashMap<ConcreteTarget, SystemTime>,
}

/// Returns the update time of the target. If it's a folder, it recursively
/// finds the latest update time of all files in the folder.
pub fn update_time(path: &ConcreteTarget) -> Result<SystemTime, Box<dyn Error>> {
    let metadata = path.pathbuf().metadata()?;
    if metadata.is_dir() {
        let mut latest = metadata.modified()?;
        if let ConcreteTarget::Deep(path) = path {
            for entry in std::fs::read_dir(path)? {
                let entry = entry?;
                let internal_target = ConcreteTarget::Deep(entry.path());
                let entry_time = update_time(&internal_target)?;
                if entry_time > latest {
                    latest = entry_time;
                }
            }
        }
        Ok(latest)
    } else {
        Ok(metadata.modified()?)
    }
}

impl UpdateState {
    /// Determines if the given path is up to date.
    pub fn is_up_to_date(&self, path: &ConcreteTarget) -> Result<bool, Box<dyn Error>> {
        let last_update = self.last_update.get(path);
        if let Some(last_update) = last_update {
            let current_update = update_time(path)?;
            Ok(current_update <= *last_update)
        } else {
            Ok(false)
        }
    }

    /// Updates the state of the given path.
    pub fn update_state(&mut self, path: &ConcreteTarget) -> Result<(), Box<dyn Error>> {
        let current_update = update_time(path)?;
        self.last_update.insert(path.clone(), current_update);
        Ok(())
    }
}

/// Returns true if the target was updated. Might be an error if there is no
/// rule to make the target.
pub fn make(
    file: &MkFile,
    target: &Target,
    update_state: &mut UpdateState,
) -> Result<bool, Box<dyn std::error::Error>> {
    info!("Making target '{:?}'", target);

    if !file.has_target(target) {
        match target {
            Target::Virtual(name) => {
                return Err(format!("No rule to make virtual target '{name}'").into());
            }
            Target::Concrete(path) => {
                if !update_state.is_up_to_date(path)? {
                    update_state.update_state(path)?;
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
        }
    }

    let dependency_make_results = file
        .dependencies(target)
        .iter()
        .map(|t| make(file, t, update_state))
        .collect::<Result<Vec<_>, _>>()?;

    let mut needs_making = dependency_make_results.iter().any(|b| *b);

    // if it's concrete and doesn't exist, it needs making
    if let Target::Concrete(path) = target {
        if !path.exists() {
            needs_making = true;
        }
    }

    // If it's virtual and has no dependencies, it always needs making
    if let Target::Virtual(_) = target {
        if dependency_make_results.is_empty() {
            needs_making = true;
        }
    }

    if needs_making {
        let commands = file.commands(target);
        for command in commands {
            info!("Executing command '{}'", command);
            let status = std::process::Command::new("sh")
                .arg("-c")
                .arg(command)
                .status()?;

            if !status.success() {
                return Err(format!("Failed to execute command '{}'", command).into());
            }
        }
        if let Target::Concrete(path) = target {
            // See if the file does exist
            if path.exists() {
                update_state.update_state(path)?;
            } else {
                return Err(format!("Target '{path:?}' was not created").into());
            }
        }
    } else {
        // If it's concrete, update the state
        if let Target::Concrete(path) = target {
            update_state.update_state(path)?;
        }
    }

    Ok(needs_making)
}
