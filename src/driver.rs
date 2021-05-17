use crate::dotflex::{util, common, parser, sync};
use super::dotflex::tracker::{Features, TrackedFeature};
use super::dotflex::operation::{FeatureSchema, OperationSchema, ShellInvocation, OperationEffects, OperationInstance};
use std::path::{PathBuf, Path};
use std::fs::{self, DirEntry};
use std::collections::hash_map::Entry;
use clap::ArgMatches;
use std::process::{exit, Command};

pub fn report_status() {
    println!("Target directory: {}", util::target_dir().display());
    println!("Config directory: {}", util::config_dir().display());
    println!();
    println!("Features:");
    let feats = load_features();
    if 0 == feats.expose().iter().count() {
        println!("  -- no features found.");
        return;
    }
    let longest_feature_name_len : usize = feats.expose()
        .keys()
        .max_by_key(|s| s.chars().count())
        .unwrap().chars().count();
    for (name, feat) in feats.expose().iter() {
        println!("  {:<width$}: {}",
            name,
            if feat.active() { "enabled" } else { "disabled" },
            width = longest_feature_name_len);
        if !common::output_verbose() {
            continue;
        }
        for file in feat.files().iter() {
            let path = file.local_path().unwrap_or(Path::new("<no target path>"));
            println!("    {} ({})",
                path.display(),
                util::resolve_path_target(path).display());
        }
    }
    feats.dump_local();
}

pub fn upsync(args: &ArgMatches) {
    if !util::repo_path(".git").exists() {
        eprintln!("can't upsync: no local repo");
        exit(1);
    }
    let did_sync = sync::git::upsync(None);
    if !did_sync {
        eprintln!("failed to upsync!");
        exit(1);
    }
}

pub fn downsync(args: &ArgMatches) {
    if !util::repo_path(".git").exists() {
        eprintln!("can't downsync: no local repo found at {}",
            util::repo_dir().display());
        exit(1);
    }
    let did_sync = sync::git::downsync(None);
    if !did_sync {
        eprintln!("failed to downsync");
        exit(1);
    }
}

pub fn init(args: &ArgMatches) {
    if args.is_present("git") {
        let repo = args.value_of("git");
        if repo.is_none() {
            eprintln!("no value given for --git");
            exit(1);
        }
        util::assure_path_to(util::repo_dir().parent().unwrap());
        let git_init = Command::new("git")
            .args(&["init", "-b", "master",
            util::repo_dir().to_str().unwrap()])
            .status();  
        if !git_init.is_ok() || !git_init.unwrap().success() {
            eprintln!("could not initialize git repository at {}", util::repo_dir().display());
            exit(1);
        }
        let git_remote = Command::new("git")
            .args(&["remote", "add", "upstream", repo.unwrap()])
            .current_dir(util::repo_dir().as_path())
            .status();
        if !git_remote.is_ok() || !git_remote.unwrap().success() {
            eprintln!("could not set up remote {} for git repository at {}", repo.unwrap(), util::repo_dir().display());
            exit(1);
        }
    } else {
        unreachable!();
    }
}

pub fn bind(args: &ArgMatches) {
    let feat = args.value_of("feature").expect("error: no feature name");

    let feat_dir = util::repo_path("features").join(feat);
    if !util::assure_path(&feat_dir) {
        eprintln!("error resolving feature directory {}",
            feat_dir.display());
        exit(1);
    }

    let files = args.grouped_values_of("files");
    if files.is_none() {
        return;
    }

    let mut operations: Vec<OperationSchema> = Vec::new();

    let files = files.unwrap();
    for binding in files {
        if binding.len() > 2 {
            eprintln!("too many arguments to option -f: expected 1 or 2");
            exit(1);
        }
        let binding_target = util::resolve_path_target(binding[0]);
        let binding_repo = if binding.len() == 2 {
            util::resolve_common(binding[1])
                .unwrap_or(feat_dir.join(binding[1]))
        } else {
            if !binding_target.starts_with(util::target_dir()) {
                eprintln!("error: binding out-of-target file must be fully specified: {}", binding_target.display());
                exit(1);
            }
            feat_dir.join(binding_target
                    .strip_prefix(util::target_dir()).unwrap())
        };

        let op = OperationSchema::CopyFile {
            from: binding_target,
            to: binding_repo
        };
        operations.push(op);
    }

    for op in operations.iter() {
        if !op.is_viable() {
            println!("not viable: {}", op);
            exit(1);
        }
    }

    println!("binding...");
    for op in operations.iter() {
        let inst = OperationInstance::from(op);
        print!("  {}... ", inst);
        if inst.execute() {
            println!("ok");
        } else {
            println!("failed");
            exit(1);
        }
    }

    let mut features = load_features();
    let feature = features.expose_mut().entry(feat.to_string());

    let operations = operations.iter()
        .map(|op| match op {
            OperationSchema::CopyFile { from, to }
                => OperationSchema::CopyFile {
                    from: util::unresolve_path_repo(to),
                    to: util::unresolve_path_target(from),
                },
            _ => { unreachable!(); }
        })
        .collect::<Vec<_>>();

    match feature {
        Entry::Occupied(mut e) => {
            for op in operations.iter() {
                e.get_mut()
                    .schema_mut()
                    .install_operations_mut()
                    .push(op.clone());
            }
            e.get_mut().update_files_from_schema();
        },
        Entry::Vacant(ve) => {
            println!("creating new feature {}...", feat);
            ve.insert(TrackedFeature::new(feat.to_string(), true,
                    FeatureSchema::install(operations)));
        }
    }

    parser::dump_manifest(
        &feat_dir.join("manifest.yml"),
        features.expose().get(&feat.to_string()).unwrap().schema());
    features.dump_local();
}

pub fn rebind(args: &ArgMatches) {
    let feat = args.value_of("feature").expect("error: no feature name");

    let mut features = load_features();
    let feature = features
        .expose_mut()
        .get_mut(&feat.to_string());

    if feature.is_none() {
        eprintln!("no such feature: {}", feat);
        exit(1);
    }
    let feature = feature.unwrap();

    let feat_dir = util::repo_path("features").join(feat);
    if !feat_dir.exists() {
        eprintln!("could not find feature directory: {}", feat_dir.display());
        exit(1);
    }

    let files = args.values_of("files");
    if files.is_none() {
        return;
    }

    println!("rebinding...");

    let files = files.unwrap();
    for file in files {
        let binding_target = util::resolve_path_target(file);
        let binding_target = util::unresolve_path_target(binding_target);
        let mut did_rebind = false;

        for op in feature.schema().install_operations() {
            match op {
                OperationSchema::CopyFile { from, to } => {
                    if binding_target.as_path() == to.as_path() {
                        let schema = OperationSchema::CopyFile {
                                from: util::resolve_path_target(to),
                                to: util::resolve_path_repo(from)
                            };
                        let inst = OperationInstance::from(&schema);
                        print!("  {}... ", inst);
                        println!("{}", if inst.execute() {
                            "ok"
                        } else {
                            "failed"
                        });
                        did_rebind = true;
                    }
                },
                _ => ()
            }
        }
        if !did_rebind {
            println!("-- couldn't rebind: {}", file);
        }
    }
}

pub fn enable(args: &ArgMatches) {
    let features = load_features();
    let enabled_features = args
        .values_of("enable")
        .unwrap_or(clap::Values::default())
        .collect::<Vec<_>>();
    enabled_features.iter()
        .filter_map(
        |feat| {
            let tracked_feat = features.expose()
                .get(&feat.to_string())?;
            if tracked_feat.active() {
                None
            } else {
                Some(tracked_feat)
            }
        })
        .for_each(|tracked_feat| {
            println!("Enabling feature {}:", tracked_feat.name());
            let ops = tracked_feat.schema().install_operations()
                .iter()
                .map(|op| match op {
                    OperationSchema::CopyFile { from, to } =>
                        OperationSchema::CopyFile {
                            from: util::resolve_path_repo(from),
                            to: util::resolve_path_target(to)
                        },
                    OperationSchema::AppendToFile { from, to } =>
                        OperationSchema::AppendToFile {
                            from: util::resolve_path_repo(from),
                            to: util::resolve_path_target(to)
                        },
                    OperationSchema::ShellFile { cmd, effects } =>
                        OperationSchema::ShellFile {
                            cmd: ShellInvocation::from(
                                     &util::resolve_path_repo(cmd.file()), cmd.args().iter().map(|s| s.as_str()).collect::<Vec<&str>>().as_slice()),
                            effects: effects.clone()
                        },
                    OperationSchema::ShellString { cmd, effects } =>
                        OperationSchema::ShellString {
                            cmd: cmd.clone(),
                            effects: effects.clone()
                        }
                });
            for op in ops {
                let inst = OperationInstance::from(&op);
                print!("  {}... ", inst);
                println!("{}", if inst.execute() {
                    "ok"
                } else {
                    "failed"
                });
            }
        });
}

fn load_features() -> Features {
    let mut feats = Features::load_local();

    let features_path = util::repo_path("features");
    if features_path.exists() && features_path.is_dir() {
        let features_dir = fs::read_dir(&features_path);
        if let Err(e) = features_dir {
            eprintln!("couldn't read features directory at {}: {}",
                features_path.display(),
                e);
            std::process::exit(1);
        }
        let features_from_dir = features_dir.unwrap()
            .filter_map(|res|
                res.map_or(None,
                    |ent| Some((ent.path(), ent.path().strip_prefix(&features_path).unwrap().to_path_buf()))
                ))
            .collect::<Vec<_>>();
        for feature in features_from_dir.iter() {
            let mut manifest = feature.0.to_path_buf();
            manifest.push("manifest.yml");
            if !manifest.exists() {
                continue;
            }
            let feature_name = feature.1.to_string_lossy().into_owned();
            let entry = feats.expose_mut().entry(feature_name.clone());
            match entry {
                Entry::Occupied(mut e) => {
                    e.insert(TrackedFeature::new(
                            e.get().name().clone(),
                            e.get().active(),
                            parser::parse_manifest(&manifest)));
                },
                Entry::Vacant(ve) => {
                    println!("found unrecorded feature: {}!", &feature_name);

                    let feat = TrackedFeature::new(feature_name.clone(), false, parser::parse_manifest(&manifest));

                    parser::dump_manifest(&manifest, feat.schema());

                    ve.insert(feat);
                }
            }
        }
    }

    feats
}
