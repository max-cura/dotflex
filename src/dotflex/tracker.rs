use std::collections::{hash_map::Entry, HashMap};
use std::path::{Path, PathBuf};
use super::operation::{FeatureSchema, OperationSchema};
use std::fs::{self, OpenOptions, DirEntry};
use std::io::{BufReader, BufWriter};
use serde::{Serialize, Deserialize};
use super::util;

#[derive(Serialize, Deserialize)]
pub struct InstalledFile {
    repo_path: PathBuf,

    // Some: installed directly via OperationSchema::Copy
    // None: installed through some other OperationSchema
    local_path: Option<PathBuf>,
}

impl InstalledFile {
    pub fn repo_path(&self) -> &Path {
        self.repo_path.as_path()
    }
    pub fn local_path(&self) -> Option<&Path> {
        match &self.local_path {
            Some(pb) => Some(pb.as_path()),
            None => None
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TrackedFeature {
    name: String,
    active: bool,
    #[serde(skip)]
    files: Vec<InstalledFile>,
    #[serde(skip, default = "FeatureSchema::new")]
    schema: FeatureSchema,
}

impl TrackedFeature {
    pub fn new(name_: String, active_: bool, schema_: FeatureSchema) -> TrackedFeature {
        // let tracked_files = schema_
        //     .install_operations().iter()
        //     .filter_map(|op| {
        //         match op {
        //             OperationSchema::CopyFile { from, to } 
        //                 => Some(InstalledFile {
        //                     repo_path: PathBuf::from(from),
        //                     local_path: Some(PathBuf::from(to))
        //                 }),
        //             _ => None
        //         }
        //     })
        //     .collect::<Vec<_>>();
        TrackedFeature {
            name: name_,
            active: active_,
            files: Self::files_from_schema(&schema_),
            schema: schema_
        }
    }

    pub fn files_from_schema(schema: &FeatureSchema) -> Vec<InstalledFile> {
        schema.install_operations().iter()
            .filter_map(|op| {
                match op {
                    OperationSchema::CopyFile { from, to }
                        => Some(InstalledFile {
                            repo_path: PathBuf::from(from),
                            local_path: Some(PathBuf::from(to)),
                        }),
                    _ => None
                }
            })
        .collect::<Vec<_>>()
    }
    pub fn update_files_from_schema(&mut self) {
        self.files = Self::files_from_schema(&self.schema);
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn active(&self) -> bool {
        self.active
    }

    pub fn mark_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn files(&self) -> &Vec<InstalledFile> {
        &self.files
    }
    pub fn files_mut(&mut self) -> &mut Vec<InstalledFile> {
        &mut self.files
    }

    pub fn schema(&self) -> &FeatureSchema {
        &self.schema
    }
    pub fn schema_mut(&mut self) -> &mut FeatureSchema {
        &mut self.schema
    }
    pub fn insert_schema(&mut self, schema: FeatureSchema) {
        self.schema = schema;
    }
}

#[derive(Serialize, Deserialize)]
pub struct Features {
    features: HashMap<String, TrackedFeature>,
}

impl Features {
    pub fn expose_mut(&mut self) -> &mut HashMap<String, TrackedFeature> {
        &mut self.features
    }
    pub fn expose(&self) -> &HashMap<String, TrackedFeature> {
        &self.features
    }

    pub fn is_active(&self, name: &str) -> bool {
        match self.features.get(&name.to_string()) {
            Some(f) => f.active(),
            None => false
        }
    }

    pub fn mark_active(&mut self, name: &str) -> bool {
        self._mark_active(name, true)
    }

    pub fn mark_inactive(&mut self, name: &str) -> bool {
        self._mark_active(name, false)
    }

    pub fn load_local() -> Features {
        let path = util::local_path("features.yml");
        let mut features = Features {
            features : HashMap::new()
        };
        if path.exists() {
            let file = OpenOptions::new()
                .create(false)
                .read(true)
                .open(&path);

            if let Err(e) = file {
                eprintln!("couldn't open features manifest {} for reading: {}",
                    &path.display(),
                    e);
                std::process::exit(1);
            }

            let reader = BufReader::new(file.unwrap());
            let features_manifest = serde_yaml::from_reader(reader);

            if let Err(e) = features_manifest {
                eprintln!("couldn't parse features manifest {}: {}",
                    &path.display(),
                    e);
                std::process::exit(1);
            }
            features = features_manifest.unwrap();
        }

        features
    }

    pub fn dump_local(&self) {
        let path = util::local_path("features.yml");
        super::util::assure_path_to(&path);
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&path);
        if let Err(e) = file {
            eprintln!("couldn't open active features manifest {} for writing: {}",
                &path.display(),
                e);
            std::process::exit(1);
        }
        let writer = BufWriter::new(file.unwrap());
        let write = serde_yaml::to_writer(writer, self);
        if let Err(e) = write {
            eprintln!("couldn't write to active features manifest {}: {}",
                &path.display(),
                e);
            std::process::exit(1);
        }
    }

    pub fn install_all(&self) -> bool {
        for (feature_name, feature) in &self.features {
            if !feature.active() {
                if !feature.schema.install_feature() {
                    return false;
                }
            }
        }

        true
    }

    fn _mark_active(&mut self, name: &str, val: bool) -> bool {
        match self.features.entry(name.to_string()) {
            Entry::Occupied(mut e) => {
                e.get_mut().mark_active(val);
                true
            },
            Entry::Vacant(_) => false,
        }
    }
}
