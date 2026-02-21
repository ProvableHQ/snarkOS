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

use anyhow::{Context, bail, ensure};
use std::{
    env,
    ffi::OsStr,
    fs::{self, File},
    io::Read,
    path::Path,
    process,
    str,
};
use syn::{
    ExprMacro,
    ItemUse,
    Macro,
    StmtMacro,
    UseGroup,
    UseName,
    UsePath,
    UseRename,
    UseTree,
    spanned::Spanned,
    visit::Visit,
};
use toml::Value;
use walkdir::{DirEntry, WalkDir};

// The following license text that should be present at the beginning of every source file.
const EXPECTED_LICENSE_TEXT: &[u8] = include_bytes!(".resources/license_header");

// The following directories will be excluded from the license scan.
const DIRS_TO_SKIP: [&str; 3] = ["examples", "js", "target"];

/// Determines, if a directory contains auxiliary files, not source code, and should be skipped.
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
fn check_source_files<P: AsRef<Path>>(path: P) -> anyhow::Result<()> {
    // Perform the license year check if on Linux.
    if cfg!(target_os = "linux") {
        let os_year = process::Command::new("date").arg("+%Y").output().expect("Failed to execute 'date' command");
        let current_year = str::from_utf8(&os_year.stdout).expect("Date output was not valid UTF-8").trim();
        let license_year = str::from_utf8(&EXPECTED_LICENSE_TEXT[22..][..4]).unwrap();
        assert_eq!(license_year, current_year, "The license year doesn't match the current OS year");
    }

    let mut error_formatting_violations: Vec<(String, usize, String)> = Vec::new();
    let mut locktick_violations: Vec<(String, usize, String)> = Vec::new();

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
            ensure!(
                contents == EXPECTED_LICENSE_TEXT,
                "The license in \"{}\" is either missing or it doesn't match the expected string!",
                path.display()
            );
        }

        // Read the full file once and run all AST-based checks in a single pass.
        let src = fs::read_to_string(path).unwrap();
        let ast = syn::parse_file(&src).unwrap();

        let mut checker = FileChecker::default();
        checker.visit_file(&ast);
        checker.finalize_lock_check();

        let file_str = path.display().to_string();
        locktick_violations
            .extend(checker.lock_violations.into_iter().map(|(line, code)| (file_str.clone(), line, code)));
        error_formatting_violations
            .extend(checker.error_violations.into_iter().map(|(line, code)| (file_str.clone(), line, code)));
    }

    if !locktick_violations.is_empty() {
        eprintln!("Lock imports without `locktick` counterparts found:");
        for (file, line, code) in locktick_violations {
            eprintln!("{file}:{line} -> {code}");
        }

        bail!("Build failed due to missing locktick counterparts.");
    }

    if !error_formatting_violations.is_empty() {
        eprintln!("Forbidden error formatting found! Use `{{:#}}` in log macros or chain errors via `.context()`:");
        for (file, line, code) in error_formatting_violations {
            eprintln!("{file}:{line} -> {code}");
        }

        bail!("Build failed due to forbidden error formatting.");
    }

    Ok(())
}

/// Verifies that, if the locktick feature is enabled, the build profile includes the required settings
/// (`line-tables-only` and `strip = "none"`).
fn check_locktick_profile() -> anyhow::Result<()> {
    let locktick_enabled = env::var("CARGO_FEATURE_LOCKTICK").is_ok();
    if !locktick_enabled {
        // Nohting to check.
        return Ok(());
    }

    // First check the env variables that can override the TOML values.
    let (mut valid_debug_override, mut valid_strip_override) = (false, false);

    if let Ok(val) = env::var("CARGO_PROFILE_RELEASE_DEBUG") {
        if val != "line-tables-only" {
            bail!(
                "🔴 When enabling the locktick feature, CARGO_PROFILE_RELEASE_DEBUG may only be set to `line-tables-only`."
            );
        } else {
            valid_debug_override = true;
        }
    }
    if let Ok(val) = env::var("CARGO_PROFILE_RELEASE_STRIP") {
        if val != "none" {
            bail!("🔴 When enabling the locktick feature, CARGO_PROFILE_RELEASE_STRIP may only be set to `none`.");
        } else {
            valid_strip_override = true;
        }
    }

    if valid_debug_override && valid_strip_override {
        // Both overrides are compatible with locktick, no need to check the TOML.
        return Ok(());
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
                    bail!(
                        "🔴 When enabling the locktick feature, the profile must have debug set to `line-tables-only`. Uncomment the relevant lines in Cargo.toml."
                    );
                }
            }
        } else {
            bail!(
                "🔴 When enabling the locktick feature, the profile must have `debug` set to `line-tables-only`. Uncomment the relevant lines in Cargo.toml."
            );
        }
        if let Some(debug) = profile_settings.get("strip") {
            match debug {
                Value::String(s) if s == "none" => {
                    println!("cargo:info=manifest has strip=none");
                }
                _ => {
                    bail!(
                        "🔴 When enabling the locktick feature, the profile must have `strip` set to `none`. Uncomment the relevant lines in Cargo.toml."
                    );
                }
            }
        }
    }

    Ok(())
}

fn is_clippy() -> bool {
    env::var("RUSTC_WORKSPACE_WRAPPER").is_ok_and(|var| var.contains("clippy"))
}

fn check_tokio_console_flags() -> anyhow::Result<()> {
    // Don't run this check under clippy, otherwise it will cause issues with --all-features.
    if is_clippy() {
        return Ok(());
    }

    // Skip if the feature is not used.
    let feature_enabled = env::var("CARGO_FEATURE_TOKIO_CONSOLE").is_ok();
    if !feature_enabled {
        return Ok(());
    }

    // Check for the presence of RUSTFLAGS.
    let Ok(rustflags) = env::var("CARGO_ENCODED_RUSTFLAGS") else {
        bail!("🔴 When enabling the tokio_console feature, you must run with `RUSTFLAGS=\"--cfg tokio_unstable\"`.");
    };

    // Check for the presence of `tokio_unstable` within RUSTFLAGS.
    ensure!(
        rustflags.contains("tokio_unstable"),
        "🔴 When enabling the tokio_console feature, you must run with `RUSTFLAGS=\"--cfg tokio_unstable\"`."
    );

    Ok(())
}

const ALLOWED_WRAPPERS: &[&str] = &["flatten_error"];
const ERROR_VAR_NAMES: &[&str] = &["error", "err", "e"];
const LOCK_TYPES: &[&str] = &["Mutex", "RwLock"];

/// Visits a single source file, checking both locktick import balance and forbidden error
/// formatting in one AST pass.
#[derive(Default)]
struct FileChecker {
    /// Lock types imported from `parking_lot` or `tokio`: (line, type_name).
    non_locktick_locks: Vec<(usize, String)>,
    /// Lock types imported from `locktick`: (line, type_name).
    locktick_locks: Vec<(usize, String)>,
    lock_violations: Vec<(usize, String)>,
    error_violations: Vec<(usize, String)>,
    /// Depth counter for `#[cfg(test)]`-gated modules; imports inside are ignored.
    test_module_depth: usize,
}

impl FileChecker {
    /// After visiting the file, compare the two lock sets and populate `lock_violations`
    /// with any type that appears in one side but not the other.
    fn finalize_lock_check(&mut self) {
        let non_locktick_types: std::collections::HashSet<&str> =
            self.non_locktick_locks.iter().map(|(_, t)| t.as_str()).collect();
        let locktick_types: std::collections::HashSet<&str> =
            self.locktick_locks.iter().map(|(_, t)| t.as_str()).collect();

        // parking_lot/tokio imports with no locktick counterpart.
        for (line, ty) in &self.non_locktick_locks {
            if !locktick_types.contains(ty.as_str()) {
                self.lock_violations.push((*line, format!("{ty} imported without a locktick counterpart")));
            }
        }
        // locktick imports with no parking_lot/tokio counterpart.
        for (line, ty) in &self.locktick_locks {
            if !non_locktick_types.contains(ty.as_str()) {
                self.lock_violations
                    .push((*line, format!("{ty} imported from locktick without a non-locktick counterpart")));
            }
        }
    }
}

impl FileChecker {
    /// Collects lock-type names (`Mutex`, `RwLock`) found within a `UseTree` into `out`.
    fn collect_lock_types_in_tree(module: Option<&str>, tree: &UseTree, line: usize, out: &mut Vec<(usize, String)>) {
        match tree {
            UseTree::Name(UseName { ident, .. }) | UseTree::Rename(UseRename { ident, .. }) => {
                let name = ident.to_string();
                if LOCK_TYPES.contains(&name.as_str()) {
                    // At this point we should know if it is `tokio` or `parking_lot`.
                    let module = module.expect("module name is missing");
                    out.push((line, format!("{module}::{name}")));
                }
            }
            UseTree::Group(UseGroup { items, .. }) => {
                for item in items {
                    Self::collect_lock_types_in_tree(module, item, line, out);
                }
            }
            UseTree::Path(UsePath { tree, ident, .. }) => {
                let module = if let Some(module) = module { module } else { &ident.to_string() };
                Self::collect_lock_types_in_tree(Some(module), tree, line, out);
            }
            UseTree::Glob(_) => {}
        }
    }

    fn check_macro(&mut self, mac: &Macro) {
        let mac_name = mac.path.segments.last().unwrap().ident.to_string();
        let tokens = mac.tokens.to_string();
        let line = mac.span().start().line;

        // Check logging macros for raw error display — should use `{:#}` or a helper.
        if ["println", "format", "error", "warn", "info", "debug", "trace"].contains(&mac_name.as_str()) {
            // Detect `"{}"` with a standalone error variable, or captured-identifier syntax
            // `"{error}"` / `"{err}"` / `"{e}"`. Use word-boundary splitting for the plain
            // case to avoid matching substrings like "current_round" when checking for "err".
            let has_plain_placeholder = tokens.contains("\"{}\"")
                && ERROR_VAR_NAMES
                    .iter()
                    .any(|var| tokens.split(|c: char| !c.is_alphanumeric() && c != '_').any(|word| word == *var));
            let has_captured_error = ERROR_VAR_NAMES.iter().any(|var| tokens.contains(&format!("\"{{{var}}}\"")));
            if (has_plain_placeholder || has_captured_error) && !ALLOWED_WRAPPERS.iter().any(|f| tokens.contains(f)) {
                self.error_violations.push((line, format!("{mac_name}!({tokens})")));
            }
        }

        // Check error-construction macros (anyhow!, bail!, format_err!) for embedded error
        // variables — errors should be chained via `.context()`/`.with_context()` instead.
        // Use `anyhow_concat!` or `bail_concat!` to explicitly opt in to concatenation.
        if ["anyhow", "bail", "format_err"].contains(&mac_name.as_str()) {
            let has_captured_error = ERROR_VAR_NAMES.iter().any(|var| tokens.contains(&format!("\"{{{var}}}\"")));
            let has_embedded_error = !has_captured_error
                && ERROR_VAR_NAMES
                    .iter()
                    .any(|var| tokens.contains(&format!("{{{var}}}")) || tokens.contains(&format!("{{{var}:")));
            if has_captured_error || has_embedded_error {
                self.error_violations.push((line, format!("{mac_name}!({tokens})")));
            }
        }
    }
}

impl<'ast> Visit<'ast> for FileChecker {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        let is_test_mod = node.attrs.iter().any(|attr| {
            attr.path().is_ident("cfg")
                && attr.meta.require_list().ok().is_some_and(|l| l.tokens.to_string().contains("test"))
        });
        if is_test_mod {
            self.test_module_depth += 1;
        }
        syn::visit::visit_item_mod(self, node);
        if is_test_mod {
            self.test_module_depth -= 1;
        }
    }

    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        if self.test_module_depth > 0 {
            // Ignore test code.
            return;
        }

        let line = node.span().start().line;
        if let UseTree::Path(UsePath { ident, tree, .. }) = &node.tree {
            match ident.to_string().as_str() {
                "parking_lot" => {
                    Self::collect_lock_types_in_tree(Some("parking_lot"), tree, line, &mut self.non_locktick_locks)
                }
                "tokio" => Self::collect_lock_types_in_tree(Some("tokio"), tree, line, &mut self.non_locktick_locks),
                "locktick" => Self::collect_lock_types_in_tree(None, tree, line, &mut self.locktick_locks),
                _ => {}
            }
        }
    }

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
fn main() -> anyhow::Result<()> {
    // Single walk: check licenses, locktick imports, and error formatting for all source files.
    check_source_files(".")?;
    // Check if locktick feature is correctly enabled.
    check_locktick_profile()?;
    // Check if the tokio_console feature is correctly enabled.
    check_tokio_console_flags()?;

    // Register build-time information.
    built::write_built_file().with_context(|| "Failed to acquire build-time information")?;

    Ok(())
}
