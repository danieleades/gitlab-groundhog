use clap::Parser;

mod run;

#[derive(Debug, Parser)]
#[command(author, version)]
pub enum Command {
    Run(run::Command),
    Init,
}

impl Command {
    pub async fn run(&self) -> anyhow::Result<()> {
        match self {
            Self::Run(command) => command.run().await,
            Self::Init => todo!(),
        }
    }
}
