extern crate graphql_client;
extern crate failure;
extern crate reqwest;
extern crate dotenv;
#[macro_use]
extern crate prettytable;
extern crate chrono;

use dotenv::dotenv;
use std::env;
use graphql_client::{GraphQLQuery, Response};
use prettytable::Table;
use chrono::{TimeZone, Utc, Local};

// schema contains scalar Long that Rust language has no analog for so we map it to f64
type Long = f64;

// The paths are relative to the directory where your `Cargo.toml` is located.
// Both json and the GraphQL schema language are supported as sources for the schema
#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/digitransit-hsl-schema.json",
    query_path = "src/digitransit-hsl-queries.graphql",
)]
pub struct StopsQuery;

fn perform_my_query(variables: stops_query::Variables) -> Result<Response<stops_query::ResponseData>, failure::Error> {

    // this is the important line
    let request_body = StopsQuery::build_query(variables);

    let client = reqwest::Client::new();
    let mut res = client.post(
        &env::var("API_URL").unwrap_or(String::from("https://api.digitransit.fi/routing/v1/routers/hsl/index/graphql")))
        .json(&request_body).send()?;
    let response_body: Response<stops_query::ResponseData> = res.json()?;
    Ok(response_body)
}

fn main() -> Result<(), failure::Error> {
    dotenv().ok();
    let response = perform_my_query(stops_query::Variables { name: Some(env::var("STOP_NAME").unwrap()) }).unwrap();
    let mut table = Table::new();
    for stop in response.data.expect("no response data").stops.expect("no stops in response")
    {
        if let Some(stop) = stop {
            for time in stop.stoptimes_without_patterns.expect("no stop times in response") {
                if let Some(time) = time {
                    let service_day_seconds = time.service_day.expect("no service day for stop time");
                    let trip = time.trip.expect("no trip info for stop time");
                    let realtime = time.realtime.expect("no realtime flag for stop time");
                    let departure_seconds;
                    if realtime == true {
                        departure_seconds = time.realtime_departure.expect("no realtime timestamp in stop time");
                    } else {
                        departure_seconds = time.scheduled_departure.expect("no scheduled timestamp in stop time");
                    }
                    let utc_datetime = Utc.timestamp(service_day_seconds as i64 + departure_seconds, 0);
                    let departure_datetime = utc_datetime.with_timezone(&Local);
                    table.add_row(row!(
                        trip.route_short_name.expect("no route short name for trip"),
                        trip.trip_headsign.expect("no headsign for trip"),
                        realtime,
                        departure_datetime
                    ));
                }
            }
        }
    }
    table.printstd();
    Ok(())
}
// eof