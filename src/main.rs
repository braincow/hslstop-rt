extern crate graphql_client;
extern crate failure;
extern crate reqwest;
extern crate dotenv;
#[macro_use]
extern crate prettytable;
extern crate chrono;

use std::env;
use std::cmp::Ordering;
use dotenv::dotenv;
use graphql_client::{GraphQLQuery, Response};
use prettytable::Table;
use chrono::{TimeZone, Utc, Local, Duration};

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
    let stop_name = env::var("STOP_NAME").expect("STOP_NAME environment variable is missing. Consult README");
    let response = perform_my_query(stops_query::Variables { name: Some(stop_name.clone()) }).unwrap();
    let current_datetime = Local::now();
    println!("{} // {}", current_datetime.format("%H:%M:%S %d.%m.%Y"), stop_name);
    let mut table = Table::new();
    // add header row to output and make it pretty with colors
    table.set_titles(row!(
        BbFw->String::from("line"),
        BbFw->String::from("destination"),
        BbFw->String::from("departure time"),
    ));
    for stop in response.data.expect("no response data").stops.expect("no stops in response")
    {
        if let Some(stop) = stop {
            for time in stop.stoptimes_without_patterns.expect("no stop times in response") {
                if let Some(time) = time {
                    // parse information about stop, departure and departing trip
                    let service_day_seconds = time.service_day.expect("no service day for stop time");
                    let trip = time.trip.expect("no trip info for stop time");
                    let realtime = time.realtime.expect("no realtime flag for stop time");
                    let departure_seconds;
                    if realtime {
                        departure_seconds = time.realtime_departure.expect("no realtime timestamp in stop time");
                    } else {
                        departure_seconds = time.scheduled_departure.expect("no scheduled timestamp in stop time");
                    }
                    let utc_datetime = Utc.timestamp(service_day_seconds as i64 + departure_seconds, 0);
                    let departure_datetime = utc_datetime.with_timezone(&Local);
                    let departure_duration = departure_datetime.signed_duration_since(current_datetime);
                    let departure_delay = time.departure_delay.expect("departure delay missing from time object");
                    let departure_delay_text: String;
                    match departure_delay.cmp(&0) {
                        Ordering::Equal => {
                            departure_delay_text = String::from("");
                        },
                        Ordering::Greater => {
                            departure_delay_text = format!("(+{})", departure_delay);
                        },
                        Ordering::Less => {
                            departure_delay_text = format!("({})", departure_delay);
                        }
                    }
                    let mut departure_string;
                    if  departure_duration <= Duration::minutes(
                        env::var("DEPARTURE_ALERT").unwrap_or(String::from("5")).parse::<i64>().unwrap()) {
                            let mins = departure_duration.num_minutes();
                            let secs = departure_duration.num_seconds() - (mins * 60);
                            // perform formatting of information based on few values
                            if mins > 0 {
                                if realtime {
                                    departure_string = format!("in ~{} min {} secs {}", mins, secs, departure_delay_text);
                                } else {
                                    departure_string = format!("in {} min {} secs", mins, secs);
                                }
                            } else {
                                if realtime {
                                    departure_string = format!("in ~{} secs {}", secs, departure_delay_text);
                                } else {
                                    departure_string = format!("in {} secs", secs);
                                }
                            }
                            // add row to table with highlighting when line is departing inside of DEPARTURE_ALERT
                            table.add_row(row!(
                                bFb->trip.route_short_name.expect("no route short name for trip"),
                                bF->trip.trip_headsign.expect("no headsign for trip"),
                                bF->departure_string
                            ));
                    } else {
                        // add row to table without highligting
                        if realtime {
                            departure_string = format!("~{} ({})", departure_datetime.format("%H:%M"), departure_delay);
                        } else {
                            departure_string = format!("{}", departure_datetime.format("%H:%M"));
                        }
                        table.add_row(row!(
                            Fb->trip.route_short_name.expect("no route short name for trip"),
                            trip.trip_headsign.expect("no headsign for trip"),
                            departure_string
                        ));
                    }
                }
            }
        }
        // add devider between stops if multiple
        table.add_empty_row();
    }
    // remove last row which is always on empty row
    table.remove_row(table.len() - 1);
    // print table out
    table.printstd();
    // return to shell with empty Ok
    Ok(())
}
// eof