#![allow(dead_code)]
#![allow(unused_variables)]

use std::convert::From;
use std::ffi::{OsStr,OsString};
use std::fmt;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::iter::Iterator;

use serde::{Serialize, Deserialize};

use super::common::output_verbose;
use super::util;

#[derive(Serialize, Deserialize, Clone)]
pub struct OperationEffects {
    generates: Vec<PathBuf>,
    clobbers: Vec<PathBuf>,
    deletes: Vec<PathBuf>,
}

impl OperationEffects {
    pub fn new() -> OperationEffects {
        OperationEffects {
            generates: Vec::new(),
            clobbers: Vec::new(),
            deletes: Vec::new(),
        }
    }
    pub fn from(generates_: &[&dyn AsRef<Path>],
        clobbers_: &[&dyn AsRef<Path>],
        deletes_: &[&dyn AsRef<Path>]) -> OperationEffects {
        OperationEffects {
            generates: generates_.iter()
                .map(|p| PathBuf::from(p.as_ref()))
                .collect(),
            clobbers: clobbers_.iter()
                .map(|p| PathBuf::from(p.as_ref()))
                .collect(),
            deletes: deletes_.iter()
                .map(|p| PathBuf::from(p.as_ref()))
                .collect(),
        }
    }

    pub fn generates<T: AsRef<Path>>(&mut self, path: T) {
        self.generates.push(PathBuf::from(path.as_ref()));
    }
    pub fn clobbers<T: AsRef<Path>>(&mut self, path: T) {
        self.clobbers.push(PathBuf::from(path.as_ref()));
    }
    pub fn deletes<T: AsRef<Path>>(&mut self, path: T) {
        self.deletes.push(PathBuf::from(path.as_ref()));
    }

    pub fn generates_list(&self) -> Vec<&Path> {
        self.generates.iter().map(|pb| pb.as_path()).collect()
    }
    pub fn clobbers_list(&self) -> Vec<&Path> {
        self.clobbers.iter().map(|pb| pb.as_path()).collect()
    }
    pub fn deletes_list(&self) -> Vec<&Path> {
        self.deletes.iter().map(|pb| pb.as_path()).collect()
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ShellInvocation {
    file: PathBuf,
    args: Vec<String>,
}

impl ShellInvocation {
    pub fn from(path: &Path, args: &[&str]) -> ShellInvocation {
        ShellInvocation {
            file: PathBuf::from(path),
            args: args.iter().map(|s| String::from(*s))
                .collect()
        }
    }
    pub fn args<'a>(&'a self) -> &'a Vec<String> {
        &self.args
    }
    pub fn file(&'_ self) -> &'_ Path {
        self.file.as_path()
    }
}   

impl fmt::Display for ShellInvocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!( f, "{} {}",
            self.file.as_path().display(),
            self.args.iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<&str>>()
                    .join(" "))
    }
}

#[derive(Serialize, Deserialize)]
pub struct FeatureSchema {
    install: Vec<OperationSchema>,
    uninstall: Vec<OperationSchema>,
}

impl FeatureSchema {
    pub fn new () -> FeatureSchema {
        FeatureSchema {
            install : Vec::new(),
            uninstall : Vec::new()
        }
    }
    pub fn install (install_: Vec<OperationSchema>) -> FeatureSchema {
        FeatureSchema {
            install : Vec::from(install_),
            uninstall: Vec::new()
        }
    }

    pub fn install_operations(&self) -> &Vec<OperationSchema> {
        &self.install
    }
    pub fn install_operations_mut(&mut self) -> &mut Vec<OperationSchema> {
        &mut self.install
    }

    pub fn install_feature(&self) -> bool {
        for schema in self.install.iter() {
            if output_verbose() {
                println!("Executing: {}", schema);
            }
            if !OperationInstance::from(schema).execute() {
                return false
            }
        }
        true
    }
    pub fn uninstall_feature(&self) -> bool {
        unimplemented!();
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub enum OperationSchema {
    #[serde(rename = "copy_file")]
    CopyFile {
        from: PathBuf,
        to: PathBuf, },
    #[serde(rename = "append_file")]
    AppendToFile {
        from: PathBuf,
        to: PathBuf, },
    #[serde(rename = "shell")]
    ShellString {
        cmd: String,
        effects: Option<OperationEffects>, },
    #[serde(rename = "script")]
    ShellFile {
        cmd: ShellInvocation,
        effects: Option<OperationEffects>, },
}

impl OperationSchema {
    pub fn is_viable(&self) -> bool {
        match self {
            OperationSchema::CopyFile { from, to } => from.exists(),
            OperationSchema::AppendToFile {from, to} => from.exists(),
            OperationSchema::ShellString {cmd, effects:_} => true,
            OperationSchema::ShellFile {cmd, effects:_} => cmd.file.exists(),
        }
    }
}

impl fmt::Display for OperationSchema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OperationSchema::CopyFile { from, to } =>
                write!(f, "copying {} to {}",
                    util::unresolve_path_repo(from)
                    .as_path().display(),
                    util::unresolve_path_target(to)
                    .as_path().display()),
            OperationSchema::AppendToFile { from, to } =>
                write!(f, "append {} to {}", 
                    util::unresolve_path_repo(from)
                    .as_path().display(),
                    util::unresolve_path_target(to)
                    .as_path().display()),
            OperationSchema::ShellString { cmd, effects } =>
                write!(f, "shell: [[{:?}]]", cmd),
            OperationSchema::ShellFile { cmd, effects } =>
                write!(f, "shell: {}", cmd),
        }
    }
}

pub struct OperationInstance<'a> {
    schema: &'a OperationSchema,
}

impl<'a> From<&'a OperationSchema> for OperationInstance<'a> {
    fn from(schema: &'a OperationSchema) -> Self {
        Self { schema: schema }
    }
}

impl<'a> fmt::Display for OperationInstance<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.schema {
            OperationSchema::CopyFile { from, to } =>
                write!(f, "copying {} to {}",
                    util::unresolve_path_repo(from)
                    .as_path().display(),
                    util::unresolve_path_target(to)
                    .as_path().display()),
            OperationSchema::AppendToFile { from, to } =>
                write!(f, "appending {} to {}", 
                    util::unresolve_path_repo(from)
                    .as_path().display(),
                    util::unresolve_path_target(to)
                    .as_path().display()),
            OperationSchema::ShellString { cmd, effects } =>
                write!(f, "executing shell command"),
            OperationSchema::ShellFile { cmd, effects } =>
                write!(f, "executing file: {}", cmd),
        }
    }
}

impl<'a> OperationInstance<'a> {
    pub fn execute(&self) -> bool {
        if self.schema.is_viable() {
            match self.schema {
                OperationSchema::CopyFile { from, to } => {
                    let parent_path_maybe = to.parent();
                    if let Some(path) = parent_path_maybe {
                        fs::create_dir_all(path).expect(format!(
                            "Could not create directories necessary for path: {}",
                            to.display()
                        ).as_str());
                    }
                    if from.is_dir() {
                        let res = fs::copy(from, to);
                        if res.is_err() {
                            // eprintln!("\nfailed: {}", res.unwrap_err());
                            false
                        } else {
                            res.is_ok()
                        }
                    } else {
                        let out = Command::new("cp").arg("-R").arg(from).arg(to).output();
                        match out {
                            Ok(output) => output.status.success(),
                            Err(_) => false,
                        }
                    }
                }
                OperationSchema::ShellString { cmd, effects: _ } => {
                    let out = Command::new("sh").arg("-c").arg(cmd).output();
                    match out {
                        Ok(output) => output.status.success(),
                        Err(_) => false,
                    }
                }
                OperationSchema::ShellFile { cmd, effects: _ } => {
                    let out = Command::new(&cmd.file).args(&cmd.args).output();
                    match out {
                        Ok(output) => output.status.success(),
                        Err(_) => false,
                    }
                }
                OperationSchema::AppendToFile { from, to } => {
                    let mut to_file = fs::OpenOptions::new()
                        .create(true)
                        .write(true)
                        .append(true)
                        .open(to)
                        .or_else(|e| -> io::Result<File> { panic!("could not open or create file {}: {}", to.as_path().display(), e) })
                        .unwrap();
                    to_file.write_all(
                        fs::read(from)
                            .or_else(|e| -> io::Result<Vec<u8>> { panic!("could not open file {}: {}", from.as_path().display(), e) })
                            .unwrap()
                            .as_slice())
                        .is_ok()
                }
            }
        } else {
            // println!("not viable");
            false
        }
    }
}
