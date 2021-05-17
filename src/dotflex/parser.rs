use std::path::Path;
use std::fs::OpenOptions;
use super::operation::{
    FeatureSchema,
    ShellInvocation,
    OperationSchema
};
use super::util;
use std::io::{BufReader, BufWriter};

pub fn parse_manifest<T: AsRef<Path>>(path: &T) -> FeatureSchema {
    if !path.as_ref().exists() {
        println!("manifest file {} does not exist",
            path.as_ref().display());
        std::process::exit(1);
    }
    let file = OpenOptions::new()
        .create(false)
        .read(true)
        .open(path);
    if let Err(e) = file {
        println!("couldn't open manifest file {} for reading: {}",
            path.as_ref().display(),
            e);
        std::process::exit(1);
    }
    let reader = BufReader::new(file.unwrap());
    let feature = serde_yaml::from_reader(reader);
    if let Err(e) = feature {
        println!("couldn't parse manifest file {}: {}",
            path.as_ref().display(),
            e);
        std::process::exit(1);
    }
    feature.unwrap()
}

pub fn dump_manifest<T: AsRef<Path>> (path: &T, feature: &FeatureSchema) {
    super::util::assure_path_to(path);
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path);
    if let Err(e) = file {
        println!("couldn't open manifest file {} for writing: {}",
            path.as_ref().display(),
            e);
        std::process::exit(1);
    }
    let writer = BufWriter::new(file.unwrap());
    let write = serde_yaml::to_writer(writer, feature);
    if let Err(e) = write {
        println!("couldn't write to manifest file {}: {}",
            path.as_ref().display(),
            e);
        std::process::exit(1);
    }
}

