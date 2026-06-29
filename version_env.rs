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
    fs,
    path::{Path, PathBuf},
};

pub fn emit_version_env() {
    let release_version = find_release_version_path().unwrap_or_else(|| {
        panic!(
            "Failed to locate '.cargo/release-version' starting from '{}'",
            env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| "<unknown manifest dir>".to_string())
        )
    });

    println!("cargo:rerun-if-changed={}", release_version.display());

    let version = fs::read_to_string(&release_version)
        .unwrap_or_else(|err| panic!("Failed to read '{}': {err}", release_version.display()));
    let version = version.trim();
    let version = version.strip_prefix('v').unwrap_or(version);
    assert!(!version.is_empty(), "'{}' did not contain a version", release_version.display());

    println!("cargo:rustc-env=SNARKOS_VERSION={version}");
}

fn find_release_version_path() -> Option<PathBuf> {
    let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")?;

    Path::new(&manifest_dir)
        .ancestors()
        .map(|dir| dir.join(".cargo").join("release-version"))
        .find(|path| path.is_file())
}
