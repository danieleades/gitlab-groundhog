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
}

pub async fn create_issue(
    url: &str,
    api_key: &str,
    payload: CreateIssuePayload,
) -> reqwest::Result<String> {
    let client = reqwest::Client::builder()
        .default_headers(
            std::iter::once((
                reqwest::header::AUTHORIZATION,
                reqwest::header::HeaderValue::from_str(&format!("Bearer {api_key}")).unwrap(),
            ))
            .collect(),
        )
        .build()
        .unwrap();

    let variables = create_issue::Variables {
        project_path: payload.project_path,
        title: payload.title,
        description: payload.description,
        due: payload.due,
    };

    let response =
        graphql_client::reqwest::post_graphql::<CreateIssue, _>(&client, url, variables).await?;

    Ok(response
        .data
        .unwrap()
        .create_issue
        .unwrap()
        .issue
        .unwrap()
        .iid)
}
