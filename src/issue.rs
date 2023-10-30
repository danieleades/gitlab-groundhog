use askama::Template;
use chrono::Duration;
use chrono::NaiveDate;

/// A recurring Gitlab issue
#[derive(Debug)]
pub struct Issue<T>
where
    T: Template,
{
    start: Option<NaiveDate>,
    end: Option<NaiveDate>,
    tempo: Duration,
    template: T,
}

impl<T> Issue<T>
where
    T: Template,
{
    /// Build a new issue.
    ///
    /// # Example
    ///
    /// ```
    /// use gitlab_recurring::Issue;
    /// use chrono::{Duration, NaiveDate};
    /// use askama::Template;
    ///
    /// #[derive(Template)]
    /// #[template(source = "Hello {{ name }}", ext = "txt")]
    /// struct HelloTemplate<'a> {
    ///    name: &'a str,
    /// }
    ///
    /// let issue = Issue::builder(Duration::weeks(3), HelloTemplate{name: "John Doe"})
    ///     .start(NaiveDate::from_ymd_opt(2023, 10, 30).expect("invalid date"))
    ///     .end(NaiveDate::from_ymd_opt(2024, 10, 30).expect("invalid date"))
    ///     .build();
    /// ```
    pub const fn builder(tempo: Duration, template: T) -> Builder<T> {
        let issue = Self {
            start: None,
            end: None,
            tempo,
            template,
        };
        Builder { issue }
    }

    /// Render the template as a String.
    /// 
    /// # Errors
    /// 
    /// This can fail due the underlying askama method. See [`askama::Error`]
    pub fn render(&self) -> askama::Result<String> {
        self.template.render()
    }

    /// Check whether this recurring issue is active
    pub fn expired(&self, today: &NaiveDate) -> bool {
        self.start.map_or(true, |start| &start <= today)
            && self.end.map_or(true, |end| today < &end)
    }

    /// Check whether it's time to create a new recurrence of the issue
    /// 
    /// This method will return false if the issue is 'expired'
    pub fn due(&self, last_created: NaiveDate, today: &NaiveDate) -> bool {
        if self.expired(today) {
            return false;
        }

        last_created.signed_duration_since(last_created) > self.tempo
    }
}

#[derive(Debug)]
pub struct Builder<T>
where
    T: Template,
{
    issue: Issue<T>,
}

impl<T> Builder<T>
where
    T: Template,
{
    /// Optionally set a start date for the recurring issue
    pub const fn start(mut self, start: NaiveDate) -> Self {
        self.issue.start = Some(start);
        self
    }

    /// Optionally set an end date for the recurring issue
    pub const fn end(mut self, end: NaiveDate) -> Self {
        self.issue.end = Some(end);
        self
    }

    pub fn build(self) -> Issue<T> {
        self.issue
    }
}
