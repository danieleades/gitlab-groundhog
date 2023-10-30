use askama::Template;

#[derive(Template)]
#[template(path = "dummy.md")]
struct TestExecutionPlan {}
