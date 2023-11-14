use clap::Parser;

mod init;
mod run;

#[derive(Debug, Parser)]
#[command(author, version)]
pub enum Command {
    /// Run GroundHog and generate new Gitlab issue recurrences
    Run(run::Command),

    /// Set up a new GroundHog directory
    Init(init::Command),
}

impl Command {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Run(command) => command.run().await?,
            Self::Init(command) => command.run()?,
        };

        Ok(())
    }
}
