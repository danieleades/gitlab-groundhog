use std::{io::Write, path::PathBuf};

use chrono::NaiveDate;
use clap::Parser;
use serde::Serialize;
use tera::Tera;

use crate::{
    issues,
    ledger::{self, Data, Entry},
};

#[derive(Debug, Parser)]
pub struct Command {
    /// Path to directory containing template files.
    #[arg(short, long, env = "GROUNDHOG_TEMPLATES", default_value = "templates/")]
    templates: PathBuf,

    /// Path to the groundhog log file
    #[arg(short, long, env = "GROUNDHOG_LOG", default_value = "log.jsonl")]
    log: PathBuf,

    /// Path to the yaml file defining the recurring issues
    #[arg(short, long, env = "GROUNDHOG_ISSUES", default_value = "issues.yml")]
    issues: PathBuf,

    /// Inject the current date.
    ///
    /// Useful for debugging.
    #[arg(short, long, default_value_t = chrono::Local::now().date_naive())]
    date: NaiveDate,
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let issues = issues::load(&self.issues)?;
        println!("issues: {:?}", &issues);
        let ledger = ledger::load(&self.log);

        let mut file = std::fs::File::options()
            .create(true)
            .append(true)
            .open("log.jsonl")?;

        for (name, issue) in &issues {
            let current_issue = issue.most_recent_issue(self.date);

            dbg!(&current_issue);

            let last_published = ledger
                .entries
                .get(name)
                .and_then(|entries| entries.keys().last())
                .unwrap_or(&0);
            for n in *last_published + 1..=current_issue {
                println!("sending issue number: {n}!");
                let due = issue.due_date(n);
                let rendered = render(issue.template(), due);
                print!("{rendered}");
                let issue_id = send_issue();
                let entry = Entry {
                    name: name.to_string(),
                    data: Data {
                        issue_number: n,
                        issue_id,
                    },
                };

                let mut entry_string = serde_json::to_string(&entry).unwrap();
                entry_string.push('\n');
                file.write_all(entry_string.as_bytes())?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Serialize)]
struct Context {
    due: NaiveDate,
}

fn render(template: &str, due: NaiveDate) -> String {
    let templates = Tera::new("templates/**/*").unwrap();

    let context = Context { due };

    templates
        .render(template, &tera::Context::from_serialize(context).unwrap())
        .unwrap()
}

fn send_issue() -> String {
    "1234".to_string()
}
