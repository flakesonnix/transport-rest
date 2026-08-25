# transport-rest

> Typed, multi-language client libraries for the [transport.rest](https://transport.rest)
> public transit APIs — Deutsche Bahn, BVG/VBB Berlin & Brandenburg, Poland and
> any compatible self-hosted instance.

transport.rest is a family of free REST APIs wrapping official transit backends
(HAFAS / DB Vendo). This repository provides **one consistent API surface in six
languages**, generated from a single machine-readable schema:

| Language | Package | Status | Tests |
|---|---|---|---|
| Rust (core) | [`transport-rest`](crates/transport-rest) on crates.io | ✅ complete | unit + integration (wiremock) |
| Python 3.10+ | `transport-rest` on PyPI | ✅ complete | stdlib unittest, offline |
| TypeScript / JS | `transport-rest` on npm | ✅ complete | bun:test + tsc strict |
| Go 1.22+ | `bindings/go` module | ✅ complete | go test (httptest), CI |
| C# (.NET 8) | `TransportRest` on NuGet | ✅ complete | xunit, CI |
| Java 17+ | `io.transportrest:transport-rest` (Maven Central) | ✅ complete | JUnit 5, CI |

All clients share:

* **Idiomatic builder APIs** — `client.journeys(from, to).results(3).get()`
* **Strongly typed models** of the FPTF wire format
* **Forward compatibility** — unknown JSON fields are ignored, unknown enum
  values are captured instead of failing
* **Structured errors** — network vs timeout vs HTTP vs API vs rate limit
  (with `Retry-After`) vs serialization vs invalid parameter
* **Capability gating** — provider-specific endpoints (`/radar`, `/stations`, …)
  fail fast with a clear error instead of a mysterious 404
* **Safe defaults** — TLS certificate validation, request timeouts,
  response size limits, no automatic retries (the public instances are
  rate-limited to ~100 req/min)

## Quick start

### Rust

```rust
use transport_rest::{JourneyPlace, TransportRestClient};

#[tokio::main]
async fn main() -> Result<(), transport_rest::TransportRestError> {
    let client = TransportRestClient::new(); // Deutsche Bahn instance

    let stops = client.locations().query("Berlin").results(5).get().await?;
    for stop in &stops {
        println!("{} ({})", stop.name().unwrap_or("?"), stop.id().unwrap_or("?"));
    }

    let journeys = client
        .journeys(JourneyPlace::StopId("8011160".into()),   // Berlin Hbf
                  JourneyPlace::StopId("8000108".into()))   // Leipzig Hbf
        .transfers(0)
        .get()
        .await?;
    println!("{} journeys", journeys.journeys.len());
    Ok(())
}
```

```toml
[dependencies]
transport-rest = "0.1"
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

### Python

```python
from transport_rest import JourneyPlace, TransportRestClient

client = TransportRestClient()          # db; bvg/vbb/poland available

stops = client.locations().query("Berlin").results(5).get()
board = client.departures("8011160").results(10).get()

journeys = await (
    client.journeys(JourneyPlace.stop_id("8011160"),
                    JourneyPlace.stop_id("8000108"))
            .transfers(0)
            .get_async()                # asyncio-compatible
)
```

Pure standard library — no third-party dependencies.

### TypeScript

```ts
import { JourneyPlace, TransportRestClient } from "transport-rest";

const client = new TransportRestClient({ provider: "bvg" });
const stops = await client.locations().query("Alexanderplatz").get();
const board = await client.departures("900000100001").results(10).get();
```

Works in Node 18+, Bun and Deno; zero runtime dependencies.

### Go

```go
client, _ := transportrest.NewClient(transportrest.ClientOptions{Provider: "db"})
result, err := client.Locations(ctx).Query("Berlin").Results(5).Get()
```

### C#

```csharp
using var client = new TransportRestClient();
var result = await client.Locations().Query("Berlin").Results(5).GetAsync();
```

### Java

```java
var client = TransportRestClient.newClient().provider("db").build();
JsonNode result = client.locations().query("Berlin").get();
```

## Providers & capabilities

| Provider | Instance | Extra endpoints |
|---|---|---|
| `db` *(default)* | `v6.db.transport.rest` | `/stations`, `/stations/{id}` |
| `bvg` | `v6.bvg.transport.rest` | `/radar`, `/stops/reachable-from`, `/trips`, `/stops` |
| `vbb` | `v6.vbb.transport.rest` | like BVG |
| `poland` | `poland.transport.rest` | `/radar`, `/stops/reachable-from`, `/trips` |
| custom | any base URL | enable via options |

The core endpoints (`/locations`, `/locations/nearby`, `/stops/{id}`,
`/stops/{id}/departures`, `/stops/{id}/arrivals`, `/journeys`,
`/journeys/{ref}`, `/trips/{id}`) exist on every v6 instance.

## Error handling

Every binding exposes the same taxonomy (shown here in Rust):

```rust
match error {
    TransportRestError::Api(e) => eprintln!("API said: {} ({})", e.message, e.status),
    TransportRestError::RateLimited(e) => {
        if let Some(wait) = e.retry_after { tokio::time::sleep(wait).await; }
    }
    TransportRestError::Timeout(_) => eprintln!("too slow"),
    // Network | Http | Serialization | InvalidParameter | CapabilityNotSupported
}
```

See [docs/API_ANALYSIS.md](docs/API_ANALYSIS.md) for the full API analysis this
library is built on, [ARCHITECTURE.md](ARCHITECTURE.md) for how the code is
organized, and [BINDINGS.md](BINDINGS.md) for per-language design decisions.

## Development

```sh
# Rust core + generator
cargo test && cargo clippy --workspace --all-targets

# Regenerate bindings after schema changes (deterministic)
cargo run -p transport-rest-gen -- --schema-dir schema type-script --out bindings/typescript/src
# ... or verify they are up to date:
cargo run -p transport-rest-gen -- --schema-dir schema type-script --out bindings/typescript/src --check

# Python (stdlib only)
python3 -m unittest discover -s bindings/python/tests -t bindings/python

# TypeScript
cd bindings/typescript && npm install && bun test && bunx tsc --noEmit
```

## Scope & limitations

* The Flixbus and Nottingham instances use different schemas and are out of scope.
* No authentication is required by transport.rest today.
* The library deliberately performs **no automatic retries**; honor the public
  rate limits (~100 req/min, upstream DB APIs stricter).
* Data quality depends entirely on the upstream providers.

## License

MIT OR Apache-2.0
