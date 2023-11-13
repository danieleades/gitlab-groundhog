use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufReader, BufWriter};
use std::path::Path;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct Entry {
    pub name: String,
    pub issue_number: u32,
    pub issue_id: String,
    pub created: NaiveDate,
    pub due: Option<NaiveDate>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Data {
    pub issue_id: String,
    pub created: NaiveDate,
    pub due: Option<NaiveDate>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ledger(HashMap<String, BTreeMap<u32, Data>>);

impl Ledger {
    pub fn load(path: impl AsRef<Path>) -> Result<Self, LoadError> {
        let file = File::open(path).map(BufReader::new)?;

        let ledger = serde_json::from_reader(file)?;

        Ok(ledger)
    }

    pub fn insert(&mut self, entry: Entry) {
        let Entry {
            name,
            issue_number,
            issue_id,
            created,
            due,
        } = entry;

        let data = Data {
            issue_id,
            created,
            due,
        };
        self.0.entry(name).or_default().insert(issue_number, data);
    }

    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let file = BufWriter::new(File::options().create(true).write(true).open(path)?);
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }

    pub fn missing_issues(&self, name: &str, current_issue: Option<u32>) -> Vec<u32> {
        let already_issued = self
            .0
            .get(name)
            .map(|entries| entries.keys().copied().collect())
            .unwrap_or_default();

        missing_issue_numbers(&already_issued, current_issue)
    }
}

/// Find all missing numbers up to and including the current issue that have not already been issued.
fn missing_issue_numbers(already_issued: &HashSet<u32>, current_issue: Option<u32>) -> Vec<u32> {
    current_issue.map_or_else(Vec::default, |current_issue| {
        (0..=current_issue)
            .filter(|&n| !already_issued.contains(&n))
            .collect()
    })
}

#[derive(Debug, thiserror::Error)]
pub enum LoadError {
    #[error("failed to parse ledger: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Failed to read the ledger file: {0}")]
    Io(#[from] io::Error),
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use tempfile::NamedTempFile;

    use super::Entry;
    use super::Ledger;

    #[test]
    fn insert() {
        let mut ledger = Ledger::default();

        let entry = Entry {
            name: "path/to/project".to_string(),
            issue_number: 2,
            issue_id: "1234".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        };

        ledger.insert(entry);

        assert!(ledger.0.get("wrong/project").is_none());
        assert!(ledger.0.get("path/to/project").is_some());
    }

    use test_case::test_case;

    #[test_case(vec![0, 1, 3, 5], Some(5) => vec![2, 4]; "missing values")]
    #[test_case(vec![0, 1, 2, 3], Some(3) => Vec::<u32>::default(); "no missing value")]
    #[test_case(vec![0, 1, 2, 3], Some(4) => vec![4]; "missing last")]
    #[test_case(vec![1, 2, 3], Some(3) => vec![0]; "missing first")]
    #[test_case(vec![1, 2, 3], None => Vec::<u32>::default(); "none issued yet")]
    fn missing_issue_numbers(already_issued: Vec<u32>, current_issue: Option<u32>) -> Vec<u32> {
        super::missing_issue_numbers(&already_issued.into_iter().collect(), current_issue)
    }

    #[test]
    fn io() {
        let mut ledger = Ledger::default();

        ledger.insert(Entry {
            name: "path/to/project".to_string(),
            issue_number: 2,
            issue_id: "1234".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        });

        ledger.insert(Entry {
            name: "other/project".to_string(),
            issue_number: 4,
            issue_id: "5678".to_string(),
            created: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            due: Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        });

        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        ledger.save(path).unwrap();

        let loaded = Ledger::load(path).unwrap();

        assert_eq!(ledger, loaded);
    }
}
