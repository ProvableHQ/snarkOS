// Copyright (c) 2019-2025 Provable Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:

// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io::Read,
    path::Path,
    process,
};
use toml::Value;
use walkdir::WalkDir;

// The following license text that should be present at the beginning of every source file.
const EXPECTED_LICENSE_TEXT: &[u8] = include_bytes!(".resources/license_header");

// The following directories will be excluded from the license scan.
const DIRS_TO_SKIP: [&str; 8] = [".cargo", ".circleci", ".git", ".github", ".resources", "examples", "js", "target"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ImportOfInterest {
    Locktick,
    ParkingLot,
    Tokio,
}

fn check_locktick_imports<P: AsRef<Path>>(path: P) {
    let mut iter = WalkDir::new(path).into_iter();
    while let Some(entry) = iter.next() {
        let entry = entry.unwrap();
        let entry_type = entry.file_type();

        // Skip the specified directories.
        if entry_type.is_dir() && DIRS_TO_SKIP.contains(&entry.file_name().to_str().unwrap_or("")) {
            iter.skip_current_dir();

            continue;
        }

        let path = entry.path();

        // Ignore non-rs
        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        }

        // Read the entire file.
        let file = fs::read_to_string(path).unwrap();

        // Prepare a filtered line iterator.
        let lines = file
            .lines()
            .filter(|l| !l.is_empty()) // Ignore empty lines.
            .skip_while(|l| !l.starts_with("use")) // Skip the license etc.
            .take_while(|l| { // Process the section containing import statements.
                l.starts_with("use")
                    || l.starts_with("#[cfg")
                    || l.starts_with("//")
                    || *l == "};"
                    || l.starts_with(|c: char| c.is_ascii_whitespace())
            });

        // The currently processed import of interest.
        let mut import_of_interest: Option<ImportOfInterest> = None;
        // This value not being zero at the end of the imports suggests a missing locktick import.
        let mut lock_balance: i8 = 0;

        // Process the filtered lines.
        for line in lines {
            // Check if this is a lock-related import.
            if import_of_interest.is_none() {
                if line.starts_with("use locktick::") {
                    import_of_interest = Some(ImportOfInterest::Locktick);
                } else if line.starts_with("use parking_lot::") {
                    import_of_interest = Some(ImportOfInterest::ParkingLot);
                } else if line.starts_with("use tokio::") {
                    import_of_interest = Some(ImportOfInterest::Tokio);
                }
            }

            // Skip irrelevant imports.
            let Some(ioi) = import_of_interest else {
                continue;
            };

            // Modify the lock balance based on the type of the relevant import.
            if [ImportOfInterest::ParkingLot, ImportOfInterest::Tokio].contains(&ioi) {
                if line.contains("Mutex") {
                    lock_balance += 1;
                }
                if line.contains("RwLock") {
                    lock_balance += 1;
                }
            } else if ioi == ImportOfInterest::Locktick {
                // Use `matches` instead of just `contains` here, as more than a single
                // lock type entry is possible in a locktick import.
                for _hit in line.matches("Mutex") {
                    lock_balance -= 1;
                }
                for _hit in line.matches("RwLock") {
                    lock_balance -= 1;
                }
                // A correction in case of the `use tokio::Mutex as TMutex` convention.
                if line.contains("TMutex") {
                    lock_balance += 1;
                }
            }

            // Register the end of an import statement.
            if line.ends_with(";") {
                import_of_interest = None;
            }
        }

        // If the file has a lock import "imbalance", print it out and increment the counter.
        assert!(
            lock_balance == 0,
            "The locks in \"{}\" don't seem to have `locktick` counterparts!",
            entry.path().display()
        );
    }
}

fn check_file_licenses<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();

    let mut iter = WalkDir::new(path).into_iter();
    while let Some(entry) = iter.next() {
        let entry = entry.unwrap();
        let entry_type = entry.file_type();

        // Skip the specified directories.
        if entry_type.is_dir() && DIRS_TO_SKIP.contains(&entry.file_name().to_str().unwrap_or("")) {
            iter.skip_current_dir();

            continue;
        }

        // Check all files with the ".rs" extension.
        if entry_type.is_file() && entry.file_name().to_str().unwrap_or("").ends_with(".rs") {
            let file = File::open(entry.path()).unwrap();
            let mut contents = Vec::with_capacity(EXPECTED_LICENSE_TEXT.len());
            file.take(EXPECTED_LICENSE_TEXT.len() as u64).read_to_end(&mut contents).unwrap();

            assert!(
                contents == EXPECTED_LICENSE_TEXT,
                "The license in \"{}\" is either missing or it doesn't match the expected string!",
                entry.path().display()
            );
        }
    }
}

fn check_locktick_profile() {
    let locktick_enabled = env::var("CARGO_FEATURE_LOCKTICK").is_ok();
    if locktick_enabled {
        // First check the env variables that can override the TOML values.
        let (mut valid_debug_override, mut valid_strip_override) = (false, false);

        if let Ok(val) = env::var("CARGO_PROFILE_RELEASE_DEBUG") {
            if val != "line-tables-only" {
                eprintln!(
                    "🔴 When enabling the locktick feature, CARGO_PROFILE_RELEASE_DEBUG may only be set to `line-tables-only`."
                );
                process::exit(1);
            } else {
                valid_debug_override = true;
            }
        }
        if let Ok(val) = env::var("CARGO_PROFILE_RELEASE_STRIP") {
            if val != "none" {
                eprintln!(
                    "🔴 When enabling the locktick feature, CARGO_PROFILE_RELEASE_STRIP may only be set to `none`."
                );
                process::exit(1);
            } else {
                valid_strip_override = true;
            }
        }

        if valid_debug_override && valid_strip_override {
            // Both overrides are compatible with locktick, no need to check the TOML.
            return;
        }

        // If the relevant overrides were either invalid or not present, check the TOML.
        let profile = env::var("PROFILE").unwrap_or_else(|_| "".to_string());
        let manifest = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("Cargo.toml");
        let contents = fs::read_to_string(&manifest).expect("failed to read Cargo.toml");
        let doc: Value = toml::from_str(&contents).expect("invalid TOML in Cargo.toml");

        let profile_table = doc.get("profile").and_then(|p| p.get(profile));
        if let Some(Value::Table(profile_settings)) = profile_table {
            if let Some(debug) = profile_settings.get("debug") {
                match debug {
                    Value::String(s) if s == "line-tables-only" => {
                        println!("cargo:info=manifest has debuginfo=line-tables-only");
                    }
                    _ => {
                        eprintln!(
                            "🔴 When enabling the locktick feature, the profile must have debug set to `line-tables-only`. Uncomment the relevant lines in Cargo.toml."
                        );
                        process::exit(1);
                    }
                }
            } else {
                eprintln!(
                    "🔴 When enabling the locktick feature, the profile must have `debug` set to `line-tables-only`. Uncomment the relevant lines in Cargo.toml."
                );
                process::exit(1);
            }
            if let Some(debug) = profile_settings.get("strip") {
                match debug {
                    Value::String(s) if s == "none" => {
                        println!("cargo:info=manifest has strip=none");
                    }
                    _ => {
                        eprintln!(
                            "🔴 When enabling the locktick feature, the profile must have `strip` set to `none`. Uncomment the relevant lines in Cargo.toml."
                        );
                        process::exit(1);
                    }
                }
            }
        }
    }
}

// The build script; it currently only checks the licenses.
fn main() {
    // Check licenses in the current folder.
    check_file_licenses(".");
    // Ensure that lock imports have locktick counterparts.
    check_locktick_imports(".");
    // Check if locktick feature is correctly enabled.
    check_locktick_profile();

    // Register build-time information.
    built::write_built_file().expect("Failed to acquire build-time information");
}
