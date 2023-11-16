use std::{collections::HashSet, path::PathBuf};

use clap::Parser;
use tera::Tera;

#[derive(Debug, Parser)]
pub struct Command {
    /// The path to the template file, relative to the templates/ directory
    template: PathBuf,

    /// Path to directory containing template files.
    #[arg(short, long, env = "GROUNDHOG_TEMPLATES", default_value = "templates/")]
    templates: PathBuf,

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
            let rendered = templates.render(template_name, &tera::Context::default())?;
            println!("{rendered}");
        } else {
            let options = names.into_iter().fold(String::new(), |mut a, b| {
                a.reserve(b.len() + 3);
                a.push_str("- ");
                a.push_str(b);
                a.push('\n');
                a
            });
            eprintln!("template '{}' not found! choose from the following options:\n{options}", self.template.display());
        }

        Ok(())
    }
}
