use chrono::NaiveDate;
use graphql_client::GraphQLQuery;

type ISO8601Date = NaiveDate;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "gitlab-schema.json",
    query_path = "queries.gql",
    response_derives = "Debug"
)]
pub struct CreateIssue;

#[derive(Debug, Clone)]
pub struct CreateIssuePayload {
    pub project_path: String,
    pub title: String,
    pub description: Option<String>,
    pub due: Option<NaiveDate>,
    pub labels: Vec<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to make http request: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("Gitlab rejected the request")]
    Gitlab,
}

pub async fn create_issue(
    url: &str,
    api_key: &str,
    payload: CreateIssuePayload,
) -> Result<String, Error> {
    let auth_header = (
        reqwest::header::AUTHORIZATION,
        reqwest::header::HeaderValue::from_str(&format!("Bearer {api_key}")).unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(std::iter::once(auth_header).collect())
        .build()?;

    let variables = create_issue::Variables {
        project_path: payload.project_path,
        title: payload.title,
        description: payload.description,
        due: payload.due,
        labels: Some(payload.labels),
    };

    let response =
        graphql_client::reqwest::post_graphql::<CreateIssue, _>(&client, url, variables).await?;

    // TODO: this error handling is a bit shit.
    if response.errors.is_some() {
        return Err(Error::Gitlab);
    }
    Ok(response
        .data
        .unwrap()
        .create_issue
        .unwrap()
        .issue
        .unwrap()
        .id)
}
