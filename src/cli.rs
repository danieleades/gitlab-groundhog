use clap::Parser;

mod init;
mod preview;
mod run;

#[derive(Debug, Parser)]
#[command(author, version)]
pub enum Command {
    /// Run GroundHog and generate new Gitlab issue recurrences
    Run(run::Command),

    /// Set up a new GroundHog directory
    Init(init::Command),

    /// Preview a rendered template
    Preview(preview::Command),
}

impl Command {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Run(command) => command.run().await?,
            Self::Init(command) => command.run()?,
            Self::Preview(command) => command.run()?,
        };

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use test_case::test_case;
    use super::Command;
    use clap::Parser;

    #[test_case(&["groundhog", "run"])]
    #[test_case(&["groundhog", "preview", "path/to/template.md"])]
    #[test_case(&["groundhog", "init", "."])]
    fn parse_input(input: &[&str]) {
        Command::try_parse_from(input).expect("invalid input");
    }
}