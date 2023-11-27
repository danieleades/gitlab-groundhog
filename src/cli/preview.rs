use std::{collections::HashSet, path::PathBuf};

use anyhow::bail;
use clap::Parser;
use tera::Tera;

use crate::issues;

#[derive(Debug, Parser)]
pub struct Command {
    /// The path to the template file, relative to the templates/ directory
    template: PathBuf,

    /// Path to directory containing template files.
    #[arg(short, long, env = "GROUNDHOG_TEMPLATES", default_value = "templates/")]
    templates: PathBuf,

    /// Path to the yaml file defining the recurring issues
    #[arg(long, env = "GROUNDHOG_ISSUES", default_value = "issues.yml")]
    issues: PathBuf,

    /// The name of the issue.
    ///
    /// This is only required if the issue passes arguments to the template that are required when rendering.
    #[arg(short, long)]
    issue: Option<String>,
}

impl Command {
    pub fn run(&self) -> anyhow::Result<()> {
        let glob = format!("{}/**/*", self.templates.display());
        let templates = Tera::new(&glob)?;

        let names: HashSet<&str> = templates.get_template_names().collect();

        let template_name: &str = self.template.to_str().unwrap();

        if names.contains(template_name) {
            let context = if let Some(issue_name) = &self.issue {
                let mut issues = issues::load(&self.issues)?;
                if let Some(issue) = issues.remove(issue_name) {
                    dbg!(&issue);
                    tera::Context::from_value(issue.template_args.into()).unwrap()
                } else {
                    bail!("could not find specified issue")
                }
            } else {
                tera::Context::default()
            };

            dbg!(&context);

            let rendered = templates.render(template_name, &context)?;
            println!("{rendered}");
        } else {
            let options = names.into_iter().fold(String::new(), |mut a, b| {
                a.reserve(b.len() + 3);
                a.push_str("- ");
                a.push_str(b);
                a.push('\n');
                a
            });
            eprintln!(
                "template '{}' not found! choose from the following options:\n{options}",
                self.template.display()
            );
        }

        Ok(())
    }
}
