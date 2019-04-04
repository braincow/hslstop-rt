extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate graphql_client;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/digitransit-hsl-schema.json",
    query_path = "src/digitransit-hsl-StopsQuery.graphql",
)]
pub struct StopsQuery;

fn main() {
    println!("Hello, world!");
}
