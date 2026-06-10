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

use std::io;

use tokio::time::{Duration, MissedTickBehavior, interval};
use tracing::*;

/// Node-scale fd use.
#[derive(Debug, Clone, Copy)]
pub struct FdUsage {
    /// File descriptors currently open.
    pub open: u64,
    /// Current soft limit (RLIMIT_NOFILE). `None` == unlimited.
    pub soft_limit: Option<u64>,
}

impl FdUsage {
    /// Fraction of the soft limit in use (0.0..=1.0). 0.0 when unlimited.
    pub fn ratio(&self) -> f64 {
        match self.soft_limit {
            Some(limit) if limit > 0 => self.open as f64 / limit as f64,
            _ => 0.0,
        }
    }

    /// True once usage reaches `threshold` of the soft limit (e.g. 0.8 == 80%).
    pub fn approaching_limit(&self, threshold: f64) -> bool {
        self.soft_limit.is_some() && self.ratio() >= threshold
    }
}

/// Probe the live system: current soft limit + count of open descriptors.
pub fn fd_usage() -> io::Result<FdUsage> {
    let soft_limit = soft_nofile_limit()?;
    let open = count_open_fds(soft_limit)?;
    Ok(FdUsage { open, soft_limit })
}

fn soft_nofile_limit() -> io::Result<Option<u64>> {
    let (soft, _hard) = rlimit::Resource::NOFILE.get()?;
    Ok(if soft == rlimit::INFINITY { None } else { Some(soft) })
}

#[cfg(target_os = "linux")]
fn count_open_fds(_limit: Option<u64>) -> io::Result<u64> {
    // Each open descriptor is an entry in /proc/self/fd. The directory
    // handle itself holds one fd while we iterate, so subtract it back out.
    let mut n: u64 = 0;
    for entry in std::fs::read_dir("/proc/self/fd")? {
        entry?;
        n += 1;
    }
    Ok(n.saturating_sub(1))
}

#[cfg(all(unix, not(target_os = "linux")))]
fn count_open_fds(_limit: Option<u64>) -> io::Result<u64> {
    // macOS and most BSDs expose open fds via /dev/fd (same idea as Linux's /proc/self/fd).
    // The directory handle itself holds one fd while we iterate, so subtract it back out.
    let mut n: u64 = 0;
    for entry in std::fs::read_dir("/dev/fd")? {
        entry?;
        n += 1;
    }
    Ok(n.saturating_sub(1))
}

/// System-wide (whole machine) fd use.
#[derive(Debug, Clone, Copy)]
pub struct SystemFd {
    pub allocated: u64,
    pub max: u64,
}

impl SystemFd {
    pub fn ratio(&self) -> f64 {
        if self.max > 0 { self.allocated as f64 / self.max as f64 } else { 0.0 }
    }
}

#[cfg(target_os = "linux")]
pub fn system_fd_usage() -> std::io::Result<SystemFd> {
    // /proc/sys/fs/file-nr => "<allocated>\t<free, always 0>\t<max>"
    let s = std::fs::read_to_string("/proc/sys/fs/file-nr")?;
    let mut f = s.split_whitespace();
    let bad = || std::io::Error::new(std::io::ErrorKind::InvalidData, "unexpected file-nr format");
    let allocated = f.next().and_then(|v| v.parse().ok()).ok_or_else(bad)?;
    let _free = f.next(); // always 0 on modern kernels
    let max = f.next().and_then(|v| v.parse().ok()).ok_or_else(bad)?;
    Ok(SystemFd { allocated, max })
}

#[cfg(all(unix, not(target_os = "linux")))]
pub fn system_fd_usage() -> std::io::Result<SystemFd> {
    // OID names differ by flavor; values are plain integers.
    #[cfg(target_os = "freebsd")]
    let (cur_oid, max_oid) = ("kern.openfiles", "kern.maxfiles");
    #[cfg(target_os = "macos")]
    let (cur_oid, max_oid) = ("kern.num_files", "kern.maxfiles");
    #[cfg(any(target_os = "openbsd", target_os = "netbsd"))]
    let (cur_oid, max_oid) = ("kern.nfiles", "kern.maxfiles");
    #[cfg(not(any(target_os = "freebsd", target_os = "macos", target_os = "openbsd", target_os = "netbsd")))]
    return Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "system fd probe unsupported on this OS"));

    fn read(oid: &str) -> std::io::Result<u64> {
        let out = std::process::Command::new("sysctl").arg("-n").arg(oid).output()?;
        if !out.status.success() {
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("sysctl {oid} unavailable")));
        }
        String::from_utf8_lossy(&out.stdout)
            .trim()
            .parse()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("bad value for {oid}")))
    }

    Ok(SystemFd { allocated: read(cur_oid)?, max: read(max_oid)? })
}

pub fn spawn_fd_monitor() {
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(30));
        tick.set_missed_tick_behavior(MissedTickBehavior::Skip);

        loop {
            tick.tick().await;

            // (1) the node's own fds
            match fd_usage() {
                Ok(u) => {
                    if let Some(limit) = u.soft_limit {
                        let (pct, left) = (u.ratio() * 100.0, limit.saturating_sub(u.open));
                        if u.ratio() >= 0.95 {
                            error!(
                                scope = "process",
                                open = u.open,
                                limit,
                                left,
                                pct = format!("{pct:.1}%"),
                                "node fd usage critical"
                            );
                        } else if u.ratio() >= 0.80 {
                            warn!(
                                scope = "process",
                                open = u.open,
                                limit,
                                left,
                                pct = format!("{pct:.1}%"),
                                "node fd usage elevated"
                            );
                        }
                    }
                }
                Err(e) => error!(error = %e, "process fd probe failed"),
            }

            // (2) whole-machine fds are allowed 5 percentage points more leeway.
            match system_fd_usage() {
                Ok(s) => {
                    let (pct, left) = (s.ratio() * 100.0, s.max.saturating_sub(s.allocated));
                    if s.ratio() >= 0.90 {
                        error!(
                            scope = "system",
                            allocated = s.allocated,
                            max = s.max,
                            left,
                            pct = format!("{pct:.1}%"),
                            "system-wide fd usage critical"
                        );
                    } else if s.ratio() >= 0.75 {
                        warn!(
                            scope = "system",
                            allocated = s.allocated,
                            max = s.max,
                            left,
                            pct = format!("{pct:.1}%"),
                            "system-wide fd usage elevated"
                        );
                    }
                }
                Err(e) => error!(error = %e, "system fd probe failed"),
            }
        }
    });
}
