# API-Referenz

Gemeinsame API-Oberfläche aller Bindings (Namen je nach Sprache leicht
angepasst: `get()`/`GetAsync()`, `snake_case`/`camelCase`/`PascalCase`).

## Client-Konstruktion

| Option | Default | Bedeutung |
|---|---|---|
| provider | `db` | `db`, `bvg`, `vbb`, `poland` oder custom |
| base_url | Provider-Default | Override für Self-Hosting/Tests |
| timeout | 30 s | Gesamt-Timeout pro Request |
| user_agent | `transport-rest-<lang>/0.1.0` | bitte aussagekräftig setzen |
| max_response_bytes | 16 MiB | Schutz vor oversized Responses |
| enable_capabilities | – | Capability für Custom-Instanzen erzwingen |
| proxy / http_client / fetch_impl | – | sprachabhängig |

## Endpunkte

### locations(query) → LocationResult[]
Suche nach Stops, POIs und Adressen.
Optionen: `fuzzy`, `results`, `stops`, `addresses`, `poi`, `lines_of_stops`,
`language`. **Validierung:** `query` erforderlich.

### nearby(latitude, longitude) → LocationResult[]
Stops in der Nähe. Optionen: `results` (8), `distance` (Meter), `stops`,
`poi`, `language`. **Validierung:** Koordinatenbereiche [-90..90]/[-180..180].

### stop(id) → Stop | Station
Einzelner Stop. Optionen: `lines_of_stops`, `language`.

### departures(stop_id) → {departures[], realtime_data_updated_at}
Abfahrtstafel. Optionen: `when`, `direction`*, `duration` (Minuten),
`results`, `stopovers`, `include_related_stations`, `lines_of_stops`,
`remarks`, `language`, `more_stops`** , `products(...)`.

\* wird nur von manchen Profilen ausgewertet.
\** nur DB-Instanz; nicht mit dbnav/dbweb-Profil.

### arrivals(stop_id) → {arrivals[], ...}
Wie `departures`.

### journeys(from, to) → {journeys[], earlier_ref, later_ref, ...}
Routing. `from`/`to`/`via` sind jeweils:

* Stop-ID (`"8011160"`),
* Name (`JourneyPlace::name("Berlin Hbf")`),
* POI (id + Koordinaten),
* Adresse (Koordinaten + Text).

Zeit-Optionen (mutually exclusive, client-seitig validiert):
`departure` XOR `arrival` XOR (`earlier_than` XOR `later_than`).

Weitere Optionen: `results` (3), `stopovers`, `transfers`, `transfer_time`,
`accessibility` (`partial`|`complete`), `bike`, `start_with_walking`,
`walking_speed` (`slow`|`normal`|`fast`), `tickets`, `polylines`,
`sub_stops`, `entrances`, `remarks`, `scheduled_days`, `products(...)`.

DB-spezifisch: `loyalty_card`, `first_class`, `age`, `age_group`,
`routing_mode`, `not_only_fast_routes`, `bestprice`,
`deutschland_ticket_connections_only`.

**Pagination:** `earlier_ref`/`later_ref` einer Antwort als
`earlier_than`/`later_than` der nächsten Anfrage. Hinweis: Beim
Default-RoutingMode der DB-Instanz ist Pagination eingeschränkt;
`routing_mode("HYBRID")` liefert vollständige Pagination + stornierte
Verbindungen.

### refresh_journey(refresh_token) → {journey, ...}
Realtime-Auffrischung einer vorherigen Verbindung. Optionen: `stopovers`,
`tickets` XOR `polylines` (validiert), `remarks`, `scheduled_days`, `language`,
db: `not_only_fast_routes`, `bestprice`.

### trip(trip_id) → {trip, ...}
Fahrtdetails. Optionen: `stopovers`, `remarks`, `polyline`, `language`.

## Capability-gated (nicht auf jeder Instanz)

| Methode | Capability | Instanzen |
|---|---|---|
| radar().north/west/south/east(...) | `radar` | bvg, vbb, poland |
| reachable_from().latitude/longitude(...) | `reachable_from` | bvg, vbb, poland |
| trips_by_name(query) | `trips_by_name` | bvg, vbb, poland |
| stations() / station(id) | `stations` | db |
| stops_search() | `stops_search` | bvg, vbb |

Aufruf ohne Capability → `CapabilityNotSupported`-Fehler *vor* dem Request.

## Produktfilter

Flache Boolean-Query-Parameter; nur explizit gesetzte Keys werden gesendet
(Server-Default: alles inklusive). Bekannte Keys:
`nationalExpress`, `national`, `regionalExpress`, `regional`, `suburban`,
`subway`, `tram`, `bus`, `ferry`, `taxi`, `express`.
Beliebige provider-spezifische Keys via generischem Setter.

```rust
client.departures("8011160")
    .products(|p| p.bus(false).tram(false))
```

```python
client.departures("8011160").products(lambda p: p.bus(False))
```

```ts
client.departures("8011160").products(p => p.bus(false).tram(false));
```

## Fehler-Taxonomie

```text
TransportRestError
├── Network                    # DNS/TCP/TLS
├── Timeout{kind}              # connect | request
├── Http{status,url,body_snippet}          # nicht interpretierbare Fehlerseite
├── Api{status,url,message,body}           # {"message": "..."} der Instanz
├── RateLimited{retry_after}               # 429 + Retry-After (Header)
├── Serialization                          # invalid JSON / Schema-Drift
├── InvalidParameter{parameter,reason}     # client-seitige Validierung
└── CapabilityNotSupported{capability,provider}
```

## Datenmodelle (Auszug)

Vollständig generiert aus der IR; alle Felder optional bis auf die unten
genannten Kernfelder. Unbekannte Felder/Werte werden toleriert.

* **Location**: id?, name?, address?, poi?, latitude?, longitude?,
  altitude?, distance?
* **Stop** *(id)*: name?, station?, location?, products?, ids?, lines?,
  entrances?, is_meta?, load_factor?, transit_authority?
* **Station** *(id)*: wie Stop zzgl. stops? (Sub-Stops), regions?, facilities?
* **Line**: id?, name?, fahrtNr?, product?, mode? (offener Enum),
  operator?, express/metro/night?, symbol?
* **Operator**: id?, name?
* **Departure**: tripId?, stop?, line?, direction?, provenance?, when?,
  plannedWhen?, delay? (Sekunden), platform?, plannedPlatform?, cancelled?,
  remarks[], previousStopovers?, nextStopovers?
* **Stopover**: stop?, arrival/departure (+planned/prognosed/delay/platform),
  passBy?, cancelled?, additional?
* **Leg**: tripId?, origin*, destination*, line?, walking?, transfer?,
  distance?, reachable?, cancelled?, stopovers?, remarks[], polyline?
* **Journey**: legs*, refreshToken?, price?, remarks[], serviceDays?
* **Trip** *(id)*: wie Leg plus currentLocation?, alternatives?
* **Remark**: kind* (hint/status/warning/… offener Enum), code?, summary?,
  text?, priority?, affectedLines[]?

`*` = laut Spec verpflichtend, dennoch tolerant deserialisiert
(fehlende Pflichtfelder erzeugen `Serialization`-Fehler).
