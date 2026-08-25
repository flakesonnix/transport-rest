//! Basic usage of the transport-rest Rust core.
//!
//! Run: cargo run --example rust-basic (see crates/transport-rest/examples/)

use transport_rest::{JourneyPlace, TransportRestClient};

#[tokio::main]
async fn main() -> Result<(), transport_rest::TransportRestError> {
    let client = TransportRestClient::new();

    // 1) Find stops by name.
    let locations = client.locations().query("Berlin").results(3).get().await?;
    for location in &locations {
        println!("found: {} ({})", location.name().unwrap_or("<unnamed>"), location.id().unwrap_or("?"));
    }

    // 2) Departure board for the first stop-like result.
    if let Some(id) = locations.iter().find_map(|l| l.as_stop().map(|s| s.id.clone())) {
        let departures = client.departures(&id).results(5).get().await?;
        for dep in &departures.departures {
            println!(
                "{:>6} {:?}",
                dep.line.as_ref().and_then(|l| l.name.clone()).unwrap_or_default(),
                dep.when.map(|w| w.to_rfc2822()),
            );
        }
    }

    // 3) Route search Berlin Hbf -> Leipzig Hbf.
    let journeys = client
        .journeys(JourneyPlace::StopId("8011160".into()), JourneyPlace::StopId("8000108".into()))
        .results(3)
        .get()
        .await?;

    for journey in &journeys.journeys {
        println!(
            "journey with {} legs, price: {:?}",
            journey.legs.len(),
            journey.price.as_ref().map(|p| p.amount)
        );
    }
    Ok(())
}
