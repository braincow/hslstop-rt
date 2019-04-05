extern crate graphql_client;
extern crate failure;
extern crate reqwest;
extern crate dotenv;

use dotenv::dotenv;
use std::env;
use graphql_client::{GraphQLQuery, Response};

// schema contains scalar Long that Rust language has no analog for so we map it to f64
type Long = f64;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/digitransit-hsl-schema.json",
    query_path = "src/digitransit-hsl-queries.graphql",
    response_derives = "Debug",
)]
pub struct StopsQuery;

fn perform_my_query(variables: stops_query::Variables) -> Result<(), failure::Error> {

    // this is the important line
    let request_body = StopsQuery::build_query(variables);

    let client = reqwest::Client::new();
    let mut res = client.post(
        &env::var("API_URL").unwrap_or(String::from("https://api.digitransit.fi/routing/v1/routers/hsl/index/graphql")))
        .json(&request_body).send()?;
    let response_body: Response<stops_query::ResponseData> = res.json()?;
    println!("{:#?}", response_body);
    Ok(())
}

fn main() -> Result<(), failure::Error> {
    dotenv().ok();
    perform_my_query(stops_query::Variables { name: Some(env::var("STOP_NAME").unwrap()) }).unwrap();
    Ok(())
}
// eof