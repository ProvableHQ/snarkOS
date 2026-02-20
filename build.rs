// Copyright (c) 2019-2026 Provable Inc.
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
    str,
};
use syn::{ExprMacro, Macro, StmtMacro, spanned::Spanned, visit::Visit};
use toml::Value;
use walkdir::{DirEntry, WalkDir};

// The following license text that should be present at the beginning of every source file.
const EXPECTED_LICENSE_TEXT: &[u8] = include_bytes!(".resources/license_header");

// The following directories will be excluded from the license scan.
const DIRS_TO_SKIP: [&str; 3] = ["examples", "js", "target"];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ImportOfInterest {
    Locktick,
    ParkingLot,
    Tokio,
}

fn should_skip_dir(entry: &DirEntry) -> bool {
    let entry_type = entry.file_type();
    if !entry_type.is_dir() {
        return false;
    }
    // Skip root-level dot folders (e.g. .git, .github, .cargo, .ci).
    if entry.depth() == 1 && entry.file_name().to_str().is_some_and(|n| n.starts_with('.')) {
        return true;
    }
    // Skip the specified directories at any depth.
    DIRS_TO_SKIP.contains(&entry.file_name().to_str().unwrap_or(""))
}

/// Checks license headers, locktick import balance, and forbidden error formatting in a single
/// directory walk to avoid reading every source file more than once.
fn check_source_files<P: AsRef<Path>>(path: P) {
    // Perform the license year check if on Linux.
    if cfg!(target_os = "linux") {
        let os_year = process::Command::new("date").arg("+%Y").output().expect("Failed to execute 'date' command");
        let current_year = str::from_utf8(&os_year.stdout).expect("Date output was not valid UTF-8").trim();
        let license_year = str::from_utf8(&EXPECTED_LICENSE_TEXT[22..][..4]).unwrap();
        assert_eq!(license_year, current_year, "The license year doesn't match the current OS year");
    }

    let mut error_formatting_violations: Vec<(String, usize, String)> = Vec::new();

    let mut iter = WalkDir::new(path).into_iter();
    while let Some(entry) = iter.next() {
        let entry = entry.unwrap();

        if should_skip_dir(&entry) {
            iter.skip_current_dir();
            continue;
        }

        // Only process .rs files.
        if !entry.file_type().is_file() || entry.path().extension() != Some(OsStr::new("rs")) {
            continue;
        }

        let path = entry.path();

        // --- License check (reads only the header bytes) ---
        {
            let file = File::open(path).unwrap();
            let mut contents = Vec::with_capacity(EXPECTED_LICENSE_TEXT.len());
            file.take(EXPECTED_LICENSE_TEXT.len() as u64).read_to_end(&mut contents).unwrap();
            assert!(
                contents == EXPECTED_LICENSE_TEXT,
                "The license in \"{}\" is either missing or it doesn't match the expected string!",
                path.display()
            );
        }

        // Read the full file once for the remaining checks.
        let src = fs::read_to_string(path).unwrap();

        // --- Locktick import balance check ---
        {
            let lines = src.lines().filter(|l| !l.is_empty()).skip_while(|l| !l.starts_with("use")).take_while(|l| {
                l.starts_with("use")
                    || l.starts_with("#[cfg")
                    || l.starts_with("//")
                    || *l == "};"
                    || l.starts_with(|c: char| c.is_ascii_whitespace())
            });

            let mut import_of_interest: Option<ImportOfInterest> = None;
            let mut lock_balance: i8 = 0;

            for line in lines {
                if import_of_interest.is_none() {
                    if line.starts_with("use locktick::") {
                        import_of_interest = Some(ImportOfInterest::Locktick);
                    } else if line.starts_with("use parking_lot::") {
                        import_of_interest = Some(ImportOfInterest::ParkingLot);
                    } else if line.starts_with("use tokio::") {
                        import_of_interest = Some(ImportOfInterest::Tokio);
                    }
                }

                let Some(ioi) = import_of_interest else {
                    continue;
                };

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

                if line.ends_with(";") {
                    import_of_interest = None;
                }
            }

            assert!(
                lock_balance == 0,
                "The locks in \"{}\" don't seem to have `locktick` counterparts!",
                path.display()
            );
        }

        // --- Error formatting check ---
        {
            let ast = syn::parse_file(&src).unwrap();
            let mut checker = ErrorChecker { violations: Vec::new() };
            checker.visit_file(&ast);
            error_formatting_violations
                .extend(checker.violations.into_iter().map(|(line, code)| (path.display().to_string(), line, code)));
        }
    }

    if !error_formatting_violations.is_empty() {
        eprintln!("Forbidden error formatting found! Use `{{:#}}` or helper like `full_chain()`:");
        for (file, line, code) in error_formatting_violations {
            eprintln!("{file}:{line} -> {code}");
        }
        panic!("Build failed due to forbidden error formatting.");
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

fn is_clippy() -> bool {
    env::var("RUSTC_WORKSPACE_WRAPPER").is_ok_and(|var| var.contains("clippy"))
}

fn check_tokio_console_flags() {
    // Don't run this check under clippy, otherwise it will cause issues with --all-features.
    if is_clippy() {
        return;
    }

    // Skip if the feature is not used.
    let feature_enabled = env::var("CARGO_FEATURE_TOKIO_CONSOLE").is_ok();
    if !feature_enabled {
        return;
    }

    // Check for the presence of RUSTFLAGS.
    let Ok(rustflags) = env::var("CARGO_ENCODED_RUSTFLAGS") else {
        eprintln!(
            "🔴 When enabling the tokio_console feature, you must run with `RUSTFLAGS=\"--cfg tokio_unstable\"`."
        );
        process::exit(1);
    };

    // Check for the presence of `tokio_unstable` within RUSTFLAGS.
    if !rustflags.contains("tokio_unstable") {
        eprintln!(
            "🔴 When enabling the tokio_console feature, you must run with `RUSTFLAGS=\"--cfg tokio_unstable\"`."
        );
        process::exit(1);
    }
}

/// List of allowed wrapper function names
const ALLOWED_WRAPPERS: &[&str] = &["flatten_error"];
/// Common variable names used for errors (used to detect captured-identifier format syntax).
const ERROR_VAR_NAMES: &[&str] = &["error", "err", "e"];

struct ErrorChecker {
    violations: Vec<(usize, String)>,
}

impl ErrorChecker {
    fn check_macro(&mut self, mac: &Macro) {
        let mac_name = mac.path.segments.last().unwrap().ident.to_string();
        if !["println", "format", "error", "warn", "info", "debug", "trace"].contains(&mac_name.as_str()) {
            return;
        }

        let tokens = mac.tokens.to_string();

        // Heuristic: detect raw error formatting via `"{}"` with an error variable,
        // or via the captured-identifier syntax `"{error}"` / `"{err}"` / `"{e}"`.
        //
        // For the plain-placeholder case, check that the variable name appears as a
        // standalone identifier (not as a substring of a longer word like "current_round").
        let has_plain_placeholder = tokens.contains("\"{}\"")
            && ERROR_VAR_NAMES
                .iter()
                .any(|var| tokens.split(|c: char| !c.is_alphanumeric() && c != '_').any(|word| word == *var));
        let has_captured_error = ERROR_VAR_NAMES.iter().any(|var| tokens.contains(&format!("\"{{{var}}}\"")));
        if (has_plain_placeholder || has_captured_error) && !ALLOWED_WRAPPERS.iter().any(|f| tokens.contains(f)) {
            self.violations.push((mac.span().start().line, tokens));
        }
    }
}

impl<'ast> Visit<'ast> for ErrorChecker {
    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        self.check_macro(&node.mac);
        syn::visit::visit_expr_macro(self, node);
    }

    fn visit_stmt_macro(&mut self, node: &'ast StmtMacro) {
        self.check_macro(&node.mac);
        syn::visit::visit_stmt_macro(self, node);
    }
}

// The build script.
fn main() {
    // Single walk: check licenses, locktick imports, and error formatting for all source files.
    check_source_files(".");
    // Check if locktick feature is correctly enabled.
    check_locktick_profile();
    // Check if the tokio_console feature is correctly enabled.
    check_tokio_console_flags();

    // Register build-time information.
    built::write_built_file().expect("Failed to acquire build-time information");
}
