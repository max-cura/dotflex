use std::process::Command;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use crate::dotflex::util;
use crate::dotflex::tracker::{TrackedFeature};
use crate::dotflex::operation::{OperationInstance};
use std::error::Error;

pub fn upsync(features: Option<&Vec<TrackedFeature>>) -> bool {
    // TODO: copy config dir files back to the repo
    // may need to revise the schema to do this

    let repo = util::repo_dir();

    let git_add = Command::new("git")
        .args(&["add", "-A"])
        .current_dir(repo.as_path())
        .output();

    if git_add.is_err() {
        return false
    }

    if let Err(e) = io::stdout().write_all(&git_add.unwrap().stdout) {
        panic!("error forwarding command output: {}", e);
    }

    // leaving this as an expect for now
    // since this should spawn vim or another editor
    // EDIT: manual message
    let git_commit = Command::new("git")
        .args(&["commit", "-m", "upsync"])
        .current_dir(repo.as_path())
        .status()
        .expect("Commit error");

    let git_push = Command::new("git")
        .args(&["push", "-u", "upstream", "master"])
        .current_dir(repo.as_path())
        .output();

    if git_push.is_err() {
        return false
    }

    if let Err(e) = io::stdout().write_all(&git_push.unwrap().stdout) {
        panic!("error forwarding command output: {}", e);
    }

    true
}

pub fn downsync(features: Option<&Vec<TrackedFeature>>) -> bool {
    let repo = util::repo_path("");

    let git_pull = Command::new("git")
        .args(&["pull", "upstream", "master"])
        .current_dir(&repo)
        .output();

    if git_pull.is_err() {
        return false;
    }

    if let Err(e) = io::stdout().write_all(&git_pull.unwrap().stdout) {
        panic!("error forwarding command output: {}", e);
    }

    if features.is_some() {
        let features = features.unwrap();
        for feature in features.iter() {
            if !feature.schema().install_feature() {
                return false;
            }
        }
    }
    true
}
