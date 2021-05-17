// -*- rust -*-
// mod dotflex::util
// changelog
//  8.5   MC added path helpers

use dirs_next;
use std::env;
use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};
use std::fs;

pub fn assure_path<T: AsRef<Path>> (path: T) -> bool {
    fs::create_dir_all(path).is_ok()
}

pub fn assure_path_to<T: AsRef<Path>>(path: T) -> bool {
    let parent_path_maybe = path.as_ref().parent();
    if let Some(path) = parent_path_maybe {
        assure_path(path)
    } else {
        true
    }
}

/// Resolve a path to an absolute path
pub fn resolve_common<T: AsRef<Path>>(p: T) -> Option<PathBuf> {
    let p = p.as_ref();
    if p.is_absolute() {
        Some(p.to_path_buf())
    } else if p.starts_with("@l") {
        Some(self::local_dir()
            .join(p.strip_prefix("@l").unwrap()))
    } else if p.starts_with("@r") {
        Some(self::repo_dir()
            .join(p.strip_prefix("@r").unwrap()))
    } else if p.starts_with("@t") {
        Some(self::target_dir()
            .join(p.strip_prefix("@t").unwrap()))
    } else {
        None
        // // normalize_path from https://github.com/rust-lang/cargo/blob/fede83ccf973457de319ba6fa0e36ead454d2e20/src/cargo/util/paths.rs
        // let mut components = p.components().peekable();
        // let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        //     components.next();
        //     PathBuf::from(c.as_os_str())
        // } else {
        //     PathBuf::new()
        // };

        // for component in components {
        //     match component {
        //         Component::Prefix(..) => unreachable!(),
        //         Component::RootDir => {
        //             ret.push(component.as_os_str());
        //         }
        //         Component::CurDir => {}
        //         Component::ParentDir => {
        //             ret.pop();
        //         }
        //         Component::Normal(c) => {
        //             ret.push(c);
        //         }
        //     }
        // }
        // Some(ret)
    }
}
pub fn resolve_path_repo<T: AsRef<Path>>(p: T) -> PathBuf {
    resolve_common(&p)
        .unwrap_or(self::repo_dir().join(&p))
}
pub fn resolve_path_target<T: AsRef<Path>>(p: T) -> PathBuf {
    resolve_common(&p)
        .unwrap_or(self::target_dir().join(&p))
}
pub fn resolve_path_local<T: AsRef<Path>>(p: T) -> PathBuf {
    resolve_common(&p)
        .unwrap_or(self::local_dir().join(&p))
}

pub fn unresolve_path<T: AsRef<Path>>(p: T) -> Option<PathBuf> {
    let p = p.as_ref();
    if p.is_absolute() {
        if p.starts_with(self::repo_dir()) {
            Some(PathBuf::from(OsStr::new("@r"))
                .join(p.strip_prefix(self::repo_dir()).unwrap()))
        } else if p.starts_with(self::local_dir()) {
            Some(PathBuf::from(OsStr::new("@l"))
                .join(p.strip_prefix(self::local_dir()).unwrap()))
        } else if p.starts_with(self::target_dir()) {
            Some(PathBuf::from(OsStr::new("@t"))
                .join(p.strip_prefix(self::target_dir()).unwrap()))
        } else {
            Some(p.to_path_buf())
        }
    } else {
        None
    }
}
pub fn unresolve_path_repo<T: AsRef<Path>>(p: T) -> PathBuf {
    unresolve_path(&p)
        .map(|pb| if pb.starts_with("@r") {
            pb.strip_prefix("@r").unwrap().to_path_buf()
        } else { pb })
        .unwrap_or(p.as_ref().to_path_buf())
}
pub fn unresolve_path_local<T: AsRef<Path>>(p: T) -> PathBuf {
    unresolve_path(&p)
        .map(|pb| if pb.starts_with("@l") {
            pb.strip_prefix("@l").unwrap().to_path_buf()
        } else { pb })
        .unwrap_or(p.as_ref().to_path_buf())
}
pub fn unresolve_path_target<T: AsRef<Path>>(p: T) -> PathBuf {
    unresolve_path(&p)
        .map(|pb| if pb.starts_with("@t") {
            pb.strip_prefix("@t").unwrap().to_path_buf()
        } else { pb })
        .unwrap_or(p.as_ref().to_path_buf())
}

fn assure_var<K: AsRef<OsStr>, F>(
    var: &'static mut Option<PathBuf>,
    env_name: K,
    default: F,
) -> &'static Path
where
    F: Fn() -> PathBuf,
{
    match var {
        Some(pb) => pb.as_path(),
        None => {
            let p = match env::var(env_name.as_ref()) {
                Ok(val) => PathBuf::from(val),
                Err(e) => {
                    if let env::VarError::NotUnicode(_) = e {
                        panic!(
                            "environmental variable {:?} could not be parsed: {}",
                            env_name.as_ref(),
                            e
                        );
                    }
                    default()
                }
            };
            if !p.exists() {
                if let Err(e) = fs::create_dir_all(&p) {
                    panic!("directory {} does not exist and cannot be created: {}", p.display(), e);
                }
            }
            let canonical = fs::canonicalize(&p);
            if let Err(e) = canonical {
                panic!("could not canonicalize path {}: {}", p.display(), e);
            }
            var.insert(canonical.unwrap());
            var.as_ref().unwrap()
        }
    }
}

static mut TARGET_PATH: Option<PathBuf> = None;
static mut CONFIG_PATH: Option<PathBuf> = None;

fn assure_config() -> &'static Option<PathBuf> {
    unsafe {
        assure_var(&mut CONFIG_PATH, "DOTFLEX_CONFIG_PATH", || {
            dirs_next::home_dir()
                .expect("No home directory found")
                .to_path_buf()
                .join(".dotflex")
        });
        &CONFIG_PATH
    }
}
fn assure_target() -> &'static Option<PathBuf> {
    unsafe {
        assure_var(&mut TARGET_PATH, "DOTFLEX_TARGET_PATH", || {
            dirs_next::home_dir()
                .expect("No home directory found")
                .to_path_buf()
        });
        &TARGET_PATH
    }
}

#[allow(dead_code)]
pub fn config_dir() -> &'static Path
{
    assure_config().as_ref().unwrap().as_path()
}
#[allow(dead_code)]
pub fn config_path<T: AsRef<Path>>(path: T) -> PathBuf {
    assure_config().as_ref().unwrap().join(path)
}
pub fn local_dir() -> PathBuf {
    config_path("LOCAL")
}
pub fn repo_dir() -> PathBuf {
    config_path("REPO")
}
#[allow(dead_code)]
pub fn local_path<T: AsRef<Path>>(path: T) -> PathBuf {
    config_path("LOCAL").join(path)
}
#[allow(dead_code)]
pub fn repo_path<T: AsRef<Path>>(path: T) -> PathBuf {
    config_path("REPO").join(path)
}
#[allow(dead_code)]
pub fn target_dir() -> &'static Path
{
    assure_target().as_ref().unwrap().as_path()
}
#[allow(dead_code)]
pub fn target_path<T: AsRef<Path>>(path: T) -> PathBuf {
    assure_target().as_ref().unwrap().join(path)
}
