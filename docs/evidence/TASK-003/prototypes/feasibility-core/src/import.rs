//! Bounded, staged ZIP import of an LCL project.
//!
//! The archive library only decodes. Every safety decision is made here,
//! before a byte reaches the workspace: entry count, names, entry types,
//! declared and actual sizes, compression ratio and duplicates. Extraction
//! goes to a fresh private staging folder, the extracted project must pass LCL
//! validation, and only then is it published with one rename. A rejected
//! archive leaves nothing behind.

use crate::lcl_bridge::{Lcl, Verdict};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy)]
pub struct Limits {
    pub max_entries: usize,
    pub max_entry_bytes: u64,
    pub max_total_bytes: u64,
    /// Largest uncompressed-to-compressed size ratio accepted for one entry.
    pub max_ratio: u64,
}

impl Default for Limits {
    fn default() -> Limits {
        Limits {
            max_entries: 256,
            max_entry_bytes: 1 << 20,
            max_total_bytes: 8 << 20,
            max_ratio: 100,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Rejection {
    BadWorkspaceName(String),
    TooManyEntries(usize),
    /// Absolute, parent-relative, backslash, drive or empty-component names.
    UnsafeName(String),
    /// Symbolic links, devices and anything else that is not a file or folder.
    NotRegular(String),
    /// Exact or case-insensitive duplicate, or a file and folder of one name.
    Duplicate(String),
    TooLarge(String),
    Ratio(String),
    NoEntryDocument,
    Invalid(Verdict),
    Exists(String),
    Io(String),
    Archive(String),
}

impl Rejection {
    pub fn code(&self) -> &'static str {
        match self {
            Rejection::BadWorkspaceName(_) => "bad_workspace_name",
            Rejection::TooManyEntries(_) => "too_many_entries",
            Rejection::UnsafeName(_) => "unsafe_name",
            Rejection::NotRegular(_) => "not_regular",
            Rejection::Duplicate(_) => "duplicate",
            Rejection::TooLarge(_) => "too_large",
            Rejection::Ratio(_) => "compression_ratio",
            Rejection::NoEntryDocument => "no_entry_document",
            Rejection::Invalid(_) => "lcl_invalid",
            Rejection::Exists(_) => "exists",
            Rejection::Io(_) => "io",
            Rejection::Archive(_) => "archive",
        }
    }
}

#[derive(Debug)]
pub struct Imported {
    pub root: PathBuf,
    pub entry: PathBuf,
    pub files: usize,
    pub bytes: u64,
    pub verdict: Verdict,
}

const S_IFMT: u32 = 0o170000;
const S_IFREG: u32 = 0o100000;
const S_IFDIR: u32 = 0o040000;

/// Imports `archive` as the workspace `workspaces_root/name`.
pub fn import(
    archive: &Path,
    staging_root: &Path,
    workspaces_root: &Path,
    name: &str,
    limits: &Limits,
    lcl: &Lcl,
) -> Result<Imported, Rejection> {
    if !is_plain_name(name) {
        return Err(Rejection::BadWorkspaceName(name.to_string()));
    }
    let destination = workspaces_root.join(name);
    if destination.exists() {
        return Err(Rejection::Exists(name.to_string()));
    }
    let io = |e: std::io::Error| Rejection::Io(e.to_string());
    fs::create_dir_all(staging_root).map_err(io)?;
    let staging = staging_root.join(format!(
        "import-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    fs::create_dir(&staging).map_err(io)?;
    let outcome = extract_and_validate(archive, &staging, limits, lcl);
    match outcome {
        Ok((entry, files, bytes, verdict)) => {
            fs::create_dir_all(workspaces_root).map_err(io)?;
            if let Err(e) = fs::rename(&staging, &destination) {
                let _ = fs::remove_dir_all(&staging);
                return Err(Rejection::Io(e.to_string()));
            }
            Ok(Imported { entry: destination.join(entry), root: destination, files, bytes, verdict })
        }
        Err(rejection) => {
            let _ = fs::remove_dir_all(&staging);
            Err(rejection)
        }
    }
}

fn extract_and_validate(
    archive: &Path,
    staging: &Path,
    limits: &Limits,
    lcl: &Lcl,
) -> Result<(PathBuf, usize, u64, Verdict), Rejection> {
    let io = |e: std::io::Error| Rejection::Io(e.to_string());
    let reader = File::open(archive).map_err(io)?;
    let mut zip = zip::ZipArchive::new(reader).map_err(|e| Rejection::Archive(e.to_string()))?;
    if zip.len() > limits.max_entries {
        return Err(Rejection::TooManyEntries(zip.len()));
    }

    // Pass 1: judge every entry from the central directory before writing.
    let mut seen = HashSet::new();
    let mut folders = HashSet::new();
    let mut declared_total = 0u64;
    let mut plan = Vec::with_capacity(zip.len());
    for index in 0..zip.len() {
        let entry = zip.by_index(index).map_err(|e| Rejection::Archive(e.to_string()))?;
        let raw = entry.name().to_string();
        let path = safe_relative(&raw).ok_or_else(|| Rejection::UnsafeName(raw.clone()))?;
        let kind = entry.unix_mode().map(|mode| mode & S_IFMT);
        if entry.is_symlink() || kind.is_some_and(|k| k != S_IFREG && k != S_IFDIR) {
            return Err(Rejection::NotRegular(raw));
        }
        let folded = path.to_string_lossy().to_lowercase();
        if !seen.insert(folded.clone()) {
            return Err(Rejection::Duplicate(raw));
        }
        let is_dir = entry.is_dir();
        if !is_dir {
            if entry.size() > limits.max_entry_bytes {
                return Err(Rejection::TooLarge(raw));
            }
            declared_total += entry.size();
            if declared_total > limits.max_total_bytes {
                return Err(Rejection::TooLarge(raw));
            }
            if entry.size() / entry.compressed_size().max(1) > limits.max_ratio {
                return Err(Rejection::Ratio(raw));
            }
        }
        // Every ancestor is a folder; a file may not share a folder's name.
        for ancestor in path.ancestors().skip(1) {
            if !ancestor.as_os_str().is_empty() {
                folders.insert(ancestor.to_string_lossy().to_lowercase());
            }
        }
        if is_dir {
            folders.insert(folded);
        }
        plan.push((index, path, is_dir));
    }
    for (_, path, is_dir) in &plan {
        if !is_dir && folders.contains(&path.to_string_lossy().to_lowercase()) {
            return Err(Rejection::Duplicate(path.to_string_lossy().into_owned()));
        }
    }

    // Pass 2: extract, trusting no declared size.
    let mut files = 0usize;
    let mut total = 0u64;
    for (index, path, is_dir) in &plan {
        let target = staging.join(path);
        if *is_dir {
            fs::create_dir_all(&target).map_err(io)?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io)?;
        }
        let entry = zip.by_index(*index).map_err(|e| Rejection::Archive(e.to_string()))?;
        let declared = entry.size();
        let mut bytes = Vec::new();
        entry
            .take(limits.max_entry_bytes + 1)
            .read_to_end(&mut bytes)
            .map_err(|e| Rejection::Archive(e.to_string()))?;
        let actual = bytes.len() as u64;
        total += actual;
        if actual != declared || actual > limits.max_entry_bytes || total > limits.max_total_bytes {
            return Err(Rejection::TooLarge(path.to_string_lossy().into_owned()));
        }
        // create_new never follows or replaces an existing path.
        let mut out = OpenOptions::new().write(true).create_new(true).open(&target).map_err(io)?;
        out.write_all(&bytes).map_err(io)?;
        out.sync_all().map_err(io)?;
        files += 1;
    }

    let entry = entry_document(staging).ok_or(Rejection::NoEntryDocument)?;
    let verdict = lcl.validate(&staging.join(&entry)).map_err(Rejection::Io)?;
    if !verdict.accepted() {
        return Err(Rejection::Invalid(verdict));
    }
    Ok((entry, files, total, verdict))
}

/// `main.lcl.txt` at the root, or else the only `.lcl.txt` file at the root.
fn entry_document(root: &Path) -> Option<PathBuf> {
    if root.join("main.lcl.txt").is_file() {
        return Some(PathBuf::from("main.lcl.txt"));
    }
    let mut candidates = fs::read_dir(root)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_file() && e.file_name().to_string_lossy().ends_with(".lcl.txt"))
        .map(|e| PathBuf::from(e.file_name()));
    let first = candidates.next()?;
    candidates.next().is_none().then_some(first)
}

/// A relative path whose every component is a plain name.
fn safe_relative(name: &str) -> Option<PathBuf> {
    if name.is_empty() || name.contains('\0') || name.contains('\\') || name.starts_with('/') {
        return None;
    }
    let mut path = PathBuf::new();
    for part in name.strip_suffix('/').unwrap_or(name).split('/') {
        if part.is_empty() || part == "." || part == ".." || part.contains(':') {
            return None;
        }
        path.push(part);
    }
    Some(path)
}

fn is_plain_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 64
        && !name.starts_with('.')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
}
