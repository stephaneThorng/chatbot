use std::fs;
use std::path::{Path, PathBuf};

fn collect_rust_files(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let entries = fs::read_dir(root)
        .unwrap_or_else(|error| panic!("Failed to read directory {}: {error}", root.display()));

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!(
                "Failed to read directory entry under {}: {error}",
                root.display()
            )
        });
        let path = entry.path();
        if path.is_dir() {
            files.extend(collect_rust_files(&path));
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            files.push(path);
        }
    }

    files
}

fn relative_core_path(path: &Path) -> String {
    path.strip_prefix("src/core")
        .unwrap_or_else(|error| {
            panic!(
                "File {} is not under src/core: {error}",
                path.to_string_lossy()
            )
        })
        .to_string_lossy()
        .replace('\\', "/")
}

fn path_segments(path: &Path) -> Vec<String> {
    path.iter()
        .map(|segment| segment.to_string_lossy().into_owned())
        .collect()
}

fn core_features(root: &Path) -> Vec<String> {
    let entries = fs::read_dir(root).unwrap_or_else(|error| {
        panic!("Failed to read core directory {}: {error}", root.display())
    });

    let mut features = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .filter_map(|path| {
            path.file_name()
                .map(|name| name.to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>();

    features.sort();
    features
}

fn feature_and_layer(path: &Path) -> Option<(String, String)> {
    let segments = path_segments(path);
    let core_index = segments.iter().position(|segment| segment == "core")?;
    let feature = segments.get(core_index + 1)?.clone();
    let layer = segments.get(core_index + 2)?.clone();
    Some((feature, layer))
}

fn has_layer(feature_root: &Path, layer: &str) -> bool {
    feature_root.join(layer).is_dir()
}

fn read_source(path: &Path) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("Failed to read source file {}: {error}", path.display()))
}

#[test]
fn application_does_not_import_other_feature_application_or_any_adapter() {
    let features = core_features(Path::new("src/core"));

    for path in collect_rust_files(Path::new("src/core")) {
        let Some((feature, layer)) = feature_and_layer(&path) else {
            continue;
        };
        if layer != "application" {
            continue;
        }

        let source = read_source(&path);

        for other_feature in &features {
            if other_feature != &feature {
                if !has_layer(&Path::new("src/core").join(other_feature), "application") {
                    continue;
                }
                let forbidden = format!("use crate::core::{other_feature}::application::");
                assert!(
                    !source.contains(&forbidden),
                    "Application layer file {} imports another feature application layer via `{forbidden}`",
                    relative_core_path(&path)
                );
            }
        }

        for other_feature in &features {
            if !has_layer(&Path::new("src/core").join(other_feature), "adapter") {
                continue;
            }
            let forbidden = format!("use crate::core::{other_feature}::adapter::");
            assert!(
                !source.contains(&forbidden),
                "Application layer file {} imports an adapter module via `{forbidden}`",
                relative_core_path(&path)
            );
        }
    }
}

#[test]
fn domain_does_not_import_application_or_adapter_layers() {
    let features = core_features(Path::new("src/core"));

    for path in collect_rust_files(Path::new("src/core")) {
        let Some((_, layer)) = feature_and_layer(&path) else {
            continue;
        };
        if layer != "domain" {
            continue;
        }

        let source = read_source(&path);
        for feature in &features {
            let feature_root = Path::new("src/core").join(feature);

            if has_layer(&feature_root, "application") {
                let forbidden = format!("use crate::core::{feature}::application::");
                assert!(
                    !source.contains(&forbidden),
                    "Domain file {} imports application code via `{forbidden}`",
                    relative_core_path(&path)
                );
            }

            if has_layer(&feature_root, "adapter") {
                let forbidden = format!("use crate::core::{feature}::adapter::");
                assert!(
                    !source.contains(&forbidden),
                    "Domain file {} imports adapter code via `{forbidden}`",
                    relative_core_path(&path)
                );
            }
        }
    }
}

#[test]
fn application_does_not_mutate_workflow_in_depth() {
    for path in collect_rust_files(Path::new("src/core")) {
        let Some((_, layer)) = feature_and_layer(&path) else {
            continue;
        };
        if layer != "application" {
            continue;
        }

        let source = read_source(&path);
        assert!(
            !source.contains("active_workflow_mut("),
            "Application layer file {} mutates workflow internals via `active_workflow_mut(`",
            relative_core_path(&path)
        );
    }
}

#[test]
fn restaurant_is_not_a_standalone_core_feature() {
    assert!(
        !Path::new("src/core/restaurant").exists(),
        "Restaurant must not return as a standalone hexagon; chatbot restaurant behavior belongs under conversation"
    );

    for path in collect_rust_files(Path::new("src")) {
        let source = read_source(&path);
        assert!(
            !source.contains("crate::core::restaurant")
                && !source.contains("corebot_backend::core::restaurant"),
            "File {} imports the removed restaurant hexagon",
            path.to_string_lossy()
        );
    }
}
