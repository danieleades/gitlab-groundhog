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

pub async fn create_issue(
    url: &str,
    project_path: String,
    title: String,
    description: Option<String>,
    due: Option<NaiveDate>,
) -> reqwest::Result<String> {
    let client = reqwest::Client::new();

    let variables = create_issue::Variables {
        project_path,
        title,
        description,
        due,
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
