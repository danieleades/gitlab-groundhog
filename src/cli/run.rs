use std::{collections::HashMap, path::PathBuf};

use chrono::NaiveDate;
use clap::Parser;
use dialoguer::Confirm;
use reqwest::Url;
use tera::Tera;

use crate::{
    graphql::{self, CreateIssuePayload},
    issues::{self, Issue},
    ledger::{self, Ledger},
};

#[derive(Debug, Parser)]
pub struct Command {
    /// Path to directory containing template files.
    #[arg(short, long, env = "GROUNDHOG_GITLAB_URL")]
    url: Url,

    /// Path to directory containing template files.
    #[arg(short, long, env = "GROUNDHOG_TEMPLATES", default_value = "templates/")]
    templates: PathBuf,

    /// Path to the groundhog log file
    #[arg(short, long, env = "GROUNDHOG_LOG", default_value = "ledger.json")]
    log: PathBuf,

    /// Path to the yaml file defining the recurring issues
    #[arg(short, long, env = "GROUNDHOG_ISSUES", default_value = "issues.yml")]
    issues: PathBuf,

    /// Inject the current date.
    ///
    /// Useful for debugging.
    #[arg(short, long, default_value_t = chrono::Local::now().date_naive())]
    date: NaiveDate,

    /// If this flag is set, don't prompt user to confirm before creating new issues
    #[arg(short, long)]
    yes: bool,

    /// The gitlab API key.
    ///
    /// This should usually be set by environment variable
    #[arg(short, long, env = "GTILAB_API_KEY")]
    api: String,
}

impl Command {
    pub async fn run(&self) -> anyhow::Result<()> {
        let issues = issues::load(&self.issues)?;

        let ledger = ledger::load(&self.log);

        let to_create: Vec<_> = issues_to_create(self.date, &ledger, &issues).collect();

        if to_create.is_empty() {
            println!("no issues to create");
        } else {
            self.create_issues(ledger, to_create).await?;
        }

        Ok(())
    }

    async fn create_issues(
        &self,
        mut ledger: Ledger,
        to_create: Vec<(u32, CreateIssuePayload)>,
    ) -> anyhow::Result<()> {
        println!("to create: {to_create:#?}");
        if self.confirm() {
            let to_record = self.send_all(to_create).await;

            for res in to_record {
                match res {
                    Ok(entry) => ledger.insert(entry),
                    Err(e) => eprintln!("{e}"),
                }
            }
            ledger.save(&self.log)
        } else {
            Ok(())
        }
    }

    fn confirm(&self) -> bool {
        self.yes
            || Confirm::new()
                .with_prompt("create issues?")
                .default(true)
                .interact()
                .unwrap()
    }

    async fn send(
        &self,
        issue_number: u32,
        payload: CreateIssuePayload,
    ) -> reqwest::Result<ledger::Entry> {
        let name = payload.title.clone();
        let due = payload.due;
        let issue_id = graphql::create_issue(self.url.as_str(), &self.api, payload).await?;

        let entry = ledger::Entry {
            issue_id,
            created: self.date,
            due,
            name,
            issue_number,
        };

        Ok(entry)
    }

    async fn send_all(
        &self,
        to_create: Vec<(u32, CreateIssuePayload)>,
    ) -> Vec<reqwest::Result<ledger::Entry>> {
        futures::future::join_all(
            to_create
                .into_iter()
                .map(|(issue_number, payload)| self.send(issue_number, payload)),
        )
        .await
    }
}

fn issues_to_create<'a>(
    date: NaiveDate,
    ledger: &'a Ledger,
    issues: &'a HashMap<String, Issue>,
) -> impl Iterator<Item = (u32, CreateIssuePayload)> + 'a {
    issues
        .iter()
        .flat_map(move |(name, issue)| issues_to_create_for_name(date, ledger, (name, issue)))
}

fn issues_to_create_for_name<'a>(
    date: NaiveDate,
    ledger: &'a Ledger,
    (name, issue): (&'a String, &'a Issue),
) -> impl Iterator<Item = (u32, CreateIssuePayload)> + 'a {
    let current_issue = issue.most_recent_issue(date);

    let last_published = ledger
        .get(name)
        .and_then(|entries| entries.keys().last())
        .copied()
        .unwrap_or(0);

    current_issue
        .map(|current_issue| {
            (last_published + 1..=current_issue).map(move |issue_number| {
                let payload = CreateIssuePayload {
                    project_path: issue.project.clone(),
                    description: Some(render(issue.template())),
                    due: Some(issue.due_date(issue_number)),
                    title: name.to_string(),
                };

                (issue_number, payload)
            })
        })
        .into_iter()
        .flatten()
}

fn render(template: &str) -> String {
    let templates = Tera::new("templates/**/*").unwrap();

    templates
        .render(template, &tera::Context::default())
        .unwrap()
}
