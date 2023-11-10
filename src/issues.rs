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
pub struct Issue {
    start: NaiveDate,
    end: Option<NaiveDate>,
    #[serde(deserialize_with = "parse_humantime_duration")]
    tempo: Duration,
    #[serde(deserialize_with = "parse_humantime_duration")]
    notice: Duration,
    template: String,
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
    /// Find the most recent issue number that should have already been published based on today's date
    pub fn most_recent_issue(&self, today: NaiveDate) -> u32 {
        let end = self.end.map_or(today, |end| end.min(today));

        let duration_since_start = end - self.start;
        let duration_to_due = duration_since_start + self.notice;
        let periods = duration_to_due.num_seconds() / self.tempo.num_seconds();
        periods as u32
    }

    /// The path to the template associated with this Issue
    pub const fn template(&self) -> &String {
        &self.template
    }

    /// Calculate the date the given issue number should be due.
    pub fn due_date(&self, number: u32) -> NaiveDate {
        let elapsed = Duration::milliseconds(self.tempo.num_milliseconds() * i64::from(number));
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

    #[test_case(NaiveDate::from_ymd_opt(2023, 11, 4).expect("invalid date") => 0 ; "issue '0' should be published the day before the start date")]
    #[test_case(NaiveDate::from_ymd_opt(2023, 11, 10).expect("invalid date") => 0 ; "5 days after start, issue '0' is still the latest issue that should be published")]
    #[test_case(NaiveDate::from_ymd_opt(2023, 11, 11).expect("invalid date") => 1 ; "6 days after start, issue '1' should be published")]
    fn most_recent_issue(today: NaiveDate) -> u32 {
        let issue = Issue {
            start: NaiveDate::from_ymd_opt(2023, 11, 5).expect("invalid date"),
            end: None,
            // every week
            tempo: Duration::weeks(1),
            // 1 day before due date
            notice: Duration::days(1),
            template: String::from("template.md"),
        };

        issue.most_recent_issue(today)
    }
}
