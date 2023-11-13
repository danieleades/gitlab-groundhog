use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter};
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

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub issue_id: String,
    pub created: NaiveDate,
    pub due: Option<NaiveDate>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Ledger(HashMap<String, BTreeMap<u32, Data>>);

impl Ledger {
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

    pub fn get(&self, name: &str) -> Option<&BTreeMap<u32, Data>> {
        self.0.get(name)
    }

    pub fn save(&self, path: impl AsRef<Path>) -> anyhow::Result<()> {
        let file = BufWriter::new(File::options().create(true).write(true).open(path)?);
        serde_json::to_writer_pretty(file, &self)?;
        Ok(())
    }
}

pub fn load(path: &Path) -> Ledger {
    File::open(path)
        .map(BufReader::new)
        .map(|file| serde_json::from_reader(file).unwrap())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

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

        assert!(ledger.get("wrong/project").is_none());
        assert!(ledger.get("path/to/project").is_some());
    }
}
