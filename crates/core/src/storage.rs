//! Persistence: Projects as JSONL, written atomically (temp file + rename).

use std::fs;
use std::io;
use std::path::Path;

use crate::Project;

/// Why a storage operation failed.
#[derive(Debug)]
pub enum StorageError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl From<io::Error> for StorageError {
    fn from(e: io::Error) -> Self {
        StorageError::Io(e)
    }
}

impl From<serde_json::Error> for StorageError {
    fn from(e: serde_json::Error) -> Self {
        StorageError::Json(e)
    }
}

/// Write all projects to `<dir>/projects.jsonl`, one JSON object per line.
/// Atomic: writes to a temp file, then renames over the real one, so a
/// crash mid-write can never leave a half-written file behind.
pub fn save_projects(dir: &Path, projects: &[Project]) -> Result<(), StorageError> {
    fs::create_dir_all(dir)?;

    let mut contents = String::new();
    for project in projects {
        let line = serde_json::to_string(project)?;
        contents.push_str(&line);
        contents.push('\n');
    }

    let tmp = dir.join("projects.jsonl.tmp");
    fs::write(&tmp, contents)?;
    fs::rename(&tmp, dir.join("projects.jsonl"))?;
    Ok(())
}

/// Read `<dir>/projects.jsonl`. A missing file means no projects yet — not an error.
pub fn load_projects(dir: &Path) -> Result<Vec<Project>, StorageError> {
    let path = dir.join("projects.jsonl");
    let contents = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(e) => return Err(e.into()),
    };

    let mut projects = Vec::new();
    for line in contents.lines() {
        if line.is_empty() {
            continue;
        }
        projects.push(serde_json::from_str(line)?);
    }
    Ok(projects)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Slug;

    #[test]
    fn save_then_load_round_trips() {
        let dir = tempfile::tempdir().unwrap();
        // build a Vec of two Projects ("alpha", "beta"),
        let projects = vec![
            Project {
                slug: Slug::parse("alpha").unwrap(),
                name: String::from("Alpha"),
            },
            Project {
                slug: Slug::parse("beta").unwrap(),
                name: String::from("Beta"),
            },
        ];
        save_projects(dir.path(), &projects).unwrap();

        // then assert load_projects(dir.path()).unwrap() == projects
        let loaded = load_projects(dir.path()).unwrap();
        assert_eq!(loaded, projects);
    }

    #[test]
    fn load_from_empty_dir_returns_no_projects() {
        // tempdir, load_projects, assert the Vec is empty
        let dir = tempfile::tempdir().unwrap();
        let loaded = load_projects(dir.path()).unwrap();
        assert!(loaded.is_empty());
    }
}
