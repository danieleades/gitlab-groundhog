use std::collections::{BTreeMap, HashMap};
use std::io::BufReader;
use std::path::Path;
use std::{fs::File, io::BufRead};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Entry {
    pub name: String,
    #[serde(flatten)]
    pub data: Data,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub issue_number: u32,
    pub issue_id: String,
}

type IssueNumber = u32;
type IssueId = String;

#[derive(Debug, Default)]
pub struct Ledger {
    pub entries: HashMap<String, BTreeMap<IssueNumber, IssueId>>,
}

impl Ledger {
    pub fn insert(&mut self, entry: Entry) {
        let Entry { name, data } = entry;
        self.entries
            .entry(name)
            .or_default()
            .insert(data.issue_number, data.issue_id);
    }
}

impl FromIterator<Entry> for Ledger {
    fn from_iter<T: IntoIterator<Item = Entry>>(iter: T) -> Self {
        iter.into_iter().fold(Self::default(), |mut ledger, entry| {
            ledger.insert(entry);
            ledger
        })
    }
}

pub fn load(path: &Path) -> Ledger {
    File::open(path)
        .map(BufReader::new)
        .map(|file| {
            file.lines()
                .map(|result| serde_json::from_str(&result.unwrap()).unwrap())
                .collect()
        })
        .unwrap_or_default()
}
