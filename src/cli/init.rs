use std::{io, path::PathBuf};

use clap::Parser;

const ISSUES: &str = include_str!("issues.yml");
const ISSUES_SCHEMA: &str = include_str!("issues.schema.json");

#[derive(Debug, Parser)]
pub struct Command {
    /// Path to the directory that should contain the files used by GroundHog.
    ///
    /// The directory will be created if it doesn't exist.
    path: PathBuf,
}

impl Command {
    pub fn run(&self) -> io::Result<()> {
        std::fs::create_dir_all(self.path.join("templates"))?;
        std::fs::write(self.path.join("templates/my-template.md"), "body text\n")?;

        std::fs::write(self.path.join("ledger.json"), "{}")?;
        std::fs::write(self.path.join("issues.yml"), ISSUES)?;
        std::fs::write(self.path.join("issues.schema.json"), ISSUES_SCHEMA)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{ISSUES, ISSUES_SCHEMA};
    use jsonschema::JSONSchema;
    use std::fmt::Write;

    #[test]
    fn issues_schema() {
        let schema = JSONSchema::compile(&serde_json::from_str(ISSUES_SCHEMA).unwrap()).unwrap();
        let issues: serde_json::Value = serde_yaml::from_str(ISSUES).unwrap();
        let result = schema.validate(&issues);
        if let Err(errors) = result {
            let msg: String = errors.fold(String::new(), |mut s, e| {
                let _ = writeln!(s, "{e}");
                s
            });
            panic!("{msg}");
        }
    }
}
