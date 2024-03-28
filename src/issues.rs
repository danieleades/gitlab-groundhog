use chrono::Duration;
use chrono::NaiveDate;
use serde::Deserialize;
use serde::Deserializer;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::path::Path;
use std::str::FromStr;

/// A recurring Gitlab issue
#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Issue {
    pub project: String,
    start: NaiveDate,
    end: Option<NaiveDate>,
    #[serde(deserialize_with = "parse_humantime_duration")]
    tempo: Duration,
    #[serde(deserialize_with = "parse_humantime_duration")]
    notice: Duration,
    template: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(default)]
    pub template_args: serde_json::Map<String, serde_json::Value>,
}

fn parse_humantime_duration<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(d)?;
    let humantime_duration = humantime::Duration::from_str(&s).map_err(serde::de::Error::custom)?;
    chrono::Duration::from_std(humantime_duration.into()).map_err(serde::de::Error::custom)
}

impl Issue {
    /// Find the most recent issue number that should have already been published based on today's date.
    ///
    /// If no issue should be published yet, returns [`None`].
    pub fn most_recent_issue(&self, today: NaiveDate) -> Option<u32> {
        let end = self.end.map_or(today, |end| end.min(today));

        let duration_since_start = end - self.start;
        let duration_to_due = duration_since_start + self.notice;
        let periods = duration_to_due.num_seconds() / self.tempo.num_seconds();

        u32::try_from(periods).ok()
    }

    /// The path to the template associated with this Issue
    pub const fn template(&self) -> &String {
        &self.template
    }

    /// Calculate the date the given issue number should be due.
    pub fn due_date(&self, number: u32) -> NaiveDate {
        let elapsed =
            Duration::try_milliseconds(self.tempo.num_milliseconds() * i64::from(number)).unwrap();
        self.start + elapsed
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to load the issues specification from disk")]
    Io(#[from] io::Error),
    #[error("failed to parse the issues specification file")]
    Yaml(#[from] serde_yaml::Error),
}

pub fn load(path: impl AsRef<Path>) -> Result<HashMap<String, Issue>, Error> {
    let file = File::open(path)?;
    let issues = serde_yaml::from_reader(file)?;
    Ok(issues)
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use chrono::NaiveDate;
    use test_case::test_case;

    use super::Issue;

    #[test_case(NaiveDate::from_ymd_opt(2023, 11, 4).expect("invalid date") => Some(0) ; "issue '0' should be published the day before the start date")]
    #[test_case(NaiveDate::from_ymd_opt(2023, 11, 10).expect("invalid date") => Some(0) ; "5 days after start, issue '0' is still the latest issue that should be published")]
    #[test_case(NaiveDate::from_ymd_opt(2023, 11, 11).expect("invalid date") => Some(1) ; "6 days after start, issue '1' should be published")]
    #[test_case(NaiveDate::from_ymd_opt(2022, 11, 11).expect("invalid date") => None ; "date before start date")]
    fn most_recent_issue(today: NaiveDate) -> Option<u32> {
        let issue = Issue {
            project: "path/to/project".to_string(),
            start: NaiveDate::from_ymd_opt(2023, 11, 5).expect("invalid date"),
            end: None,
            tempo: Duration::try_weeks(1).unwrap(),
            notice: Duration::try_days(1).unwrap(),
            template: String::from("template.md"),
            labels: Vec::default(),
            template_args: serde_json::Map::default(),
        };

        issue.most_recent_issue(today)
    }

    #[test]
    fn can_deserialise() {
        super::load("src/cli/issues.yml").unwrap();
    }
}
