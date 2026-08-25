# transport.rest – API-Analyse

> Stand der Analyse: August 2026. Diese Datei ist das Ergebnis von Phase 1 und die
> Grundlage für die interne Schema-Repräsentation (`schema/transport-rest.ir.json`).

## 1. Was ist transport.rest?

`transport.rest` ist **keine einzelne API**, sondern eine Sammlung unabhängig
betriebener, aber strukturell nahezu identischer REST-Instanzen für verschiedene
Verkehrsverbünde/Betreiber (Betreiber: derhuerst & Mitwirkende):

| Instanz | Region | Backend | Besonderheiten |
|---|---|---|---|
| `v6.db.transport.rest` | Deutschland (DB) | `db-vendo-client` (dbnav/db/dbweb) | zusätzlich `/stations`, `/stations/{id}`; **kein** `/radar`, **kein** `/stops/reachable-from` |
| `v6.bvg.transport.rest` | Berlin/Brandenburg (BVG) | HAFAS (`bvg-rest@6`) | zusätzlich `/stops` (Namenssuche), `/radar`, `/stops/reachable-from` |
| `v6.vbb.transport.rest` | VBB | HAFAS (`hafas-rest-api`) | wie BVG |
| `poland.transport.rest` | Polen | HAFAS | wie BVG |
| `1.flixbus.transport.rest` | Flixbus | eigenes Projekt (`meinfernbus-rest`) | abweichendes Schema, **nicht** Teil dieser Library |
| `v1.nottingham-city.transport.rest` | Nottingham | eigenes Projekt | abweichend, **nicht** Teil dieser Library |

Gemeinsame Basis aller v6-Instanzen ist
[`hafas-rest-api`](https://github.com/public-transport/hafas-rest-api) bzw.
[`db-vendo-client`](https://github.com/public-transport/db-vendo-client)
(Backend von `db-rest@6`). Die Datenmodelle folgen dem *Friendly Public Transport
Format* (FPTF v2-Draft) im `hafas-client`-Format.

## 2. Quellen (Single Source of Truth)

1. **OpenAPI 3.0.3-Spec des db-vendo-API-Servers**
   (`db-vendo-client/docs/openapi.yaml`, Version `6`) – maschinenlesbar, deckt alle
   Endpunkte + Komponenten-Schemas ab. Diese Spec wurde vollständig ausgewertet.
2. Routen-Implementierungen:
   - `public-transport/hafas-rest-api@master` → `routes/*.js` (generische Endpunkte,
     Query-Parser, Fehler-Handler)
   - `derhuerst/db-rest@6` → `api.js` (Profil-Switching `dbnav`/`db`/`dbweb`,
     `/stations`-Endpunkte), `routes/station{s}.js`
   - `derhuerst/bvg-rest@6` → `routes/stops.js`
3. Menschlesbare Doku: `https://v6.db.transport.rest/api.html`,
   `docs/readme.md` in den Repos.
4. Beispiel-Responses aus der offiziellen Doku (z. B. Stop `8010159` Halle (Saale) Hbf).

Die Live-Instanz stellt zusätzlich ihre OpenAPI-Spec unter
`/.well-known/service-desc` bereit (Swagger-Playground verlinkt sie).

## 3. Übergreifende Eigenschaften

- **Authentifizierung:** keine. Keine API-Keys.
- **Rate Limit:** 100 req/min (Burst 200), erzwungen am Reverse Proxy
  → HTTP 429 ohne spezielles Body-Format. Die db-vendo-Backends sind zusätzlich
  deutlich strenger limitiert; die API-Betreiber empfehlen sparsame Nutzung.
- **CORS:** aktiviert.
- **Caching:** `ETag` + `Cache-Control` werden gesendet (Locations ~5 min,
  Departures/Trips ~30 s, Journeys ~60 s). Clients sollten Conditional Requests
  (`If-None-Match`) unterstützen können; 304 ist ein möglicher Status.
- **Encoding:** JSON; `pretty=true` optional (Pretty-Printing).
- **Zeiten:** ISO-8601-Strings mit Offset (z. B. `2020-05-01T21:06:00+02:00`),
  Timezone der Instanz ist `Europe/Berlin`. Delays in Sekunden (Zahl oder `null`).
  `realtimeDataUpdatedAt`: Unix-Epoch (Sekunden) oder `null`.
- **Pagination:**
  - `/journeys`: `earlierThan`/`laterThan` mit den Refs aus der Antwort
    (`earlierRef`/`laterRef`); zusätzlich `Link`-Header (`prev`/`next`).
    Beim Default-`routingMode` (REALTIME) eingeschränkt; HYBRID unterstützt
    vollständige Pagination.
  - `/stops/{id}/departures|arrivals`: kein Cursor; `Link`-Header mit nächstem
    `when` (+`duration`). Die Library emuliert Pagination über `when`.
- **Fehlerformat:** JSON `{ "message": <string>, ... }`; Statuscodes:
  - `400` ungültige Parameter / fehlende Pflichtparameter
  - `404` nicht gefunden (Stop, Trip, Station …)
  - `429` Rate-Limited (Proxy)
  - `500` interner Fehler
  - `502` Upstream-(HAFAS/Vendo-)Fehler
  - `304` Not Modified (bei Conditional Requests)

## 4. Endpunkte (gemeinsamer Kern aller Instanzen)

Alle Endpunkte sind `GET`. `pretty` wird überall akzeptiert (nicht modelliert –
die Library fragt kompaktes JSON an).

### GET /locations
Suche nach Stops/Stations, POIs und Adressen.

| Parameter | Typ | Default | Anmerkung |
|---|---|---|---|
| `query` *(required)* | string | – | Suchbegriff |
| `fuzzy` | bool | `true` | |
| `results` | int | `10` | |
| `stops` | bool | `true` | |
| `addresses` | bool | `true` | |
| `poi` | bool | `true` | |
| `linesOfStops` | bool | `false` | bei db-vendo „not supported“, wird toleriert |
| `language` | string | `en` | |

Antwort: Array aus `Stop | Station | Location` (diskriminiert über `type`).

### GET /locations/nearby

| Parameter | Typ | Default |
|---|---|---|
| `latitude`, `longitude` *(required)* | number | – |
| `results` | int | `8` |
| `distance` | int | – (max. Fußweg in Metern) |
| `stops` | bool | `true` |
| `poi` | bool | `false` |
| `linesOfStops` | bool | `false` |
| `language` | string | `en` |

Antwort: Array aus `Stop | Station | Location`.

### GET /stops/{id}
Einzelner Stop/einzelne Station.

| Parameter | Typ | Default |
|---|---|---|
| `id` *(path, required)* | string | – |
| `linesOfStops` | bool | `false` |
| `language` | string | `en` |

Antwort: `Stop | Station`.

### GET /stops/{id}/departures · GET /stops/{id}/arrivals

| Parameter | Typ | Default |
|---|---|---|
| `id` *(path, required)* | string | – |
| `when` | date-time | now |
| `direction` | string | – | db-vendo: nur `dbweb`-Profil |
| `duration` | int | `10` (Minuten) |
| `results` | int | serverabhängig |
| `stopovers` | bool | `false` |
| `includeRelatedStations` | bool | `true` |
| `linesOfStops` | bool | `false` |
| `remarks` | bool | `true` |
| `language` | string | `en` |
| Produktfilter (flach): `nationalExpress`, `national`, `regionalExpress`, `regional`, `suburban`, `bus`, `ferry`, `subway`, `tram`, `taxi` | bool | `true` |
| `moreStops` | string | – | nur db-rest (bis 9 kommagetrennte EVAs, nicht dbnav/dbweb) |
| `profile` | enum | instanzabhängig | nur db: `dbnav`(default)/`db`/`dbweb` |

Antwort: `{ departures|arrivals: Departure[], realtimeDataUpdatedAt? }`.
`Departure` entspricht dem OpenAPI-Schema `Alternative` (FPTF „departure/arrival“).

### GET /journeys

`from*`/`to*`/`via*` bilden jeweils eine Location:
`{from,to,via}` = Stop-ID **oder** `{from,to,via}.name` **oder**
`.id`+`.latitude`+`.longitude` (POI) **oder** `.latitude`+`.longitude`+`.address`.

| Parameter | Typ | Default |
|---|---|---|
| `from*`, `to*` *(required, kombiniert)* | mixed | – |
| `via*` | mixed | – |
| `departure` | date-time | now | mutually exclusive mit `arrival` |
| `arrival` | date-time | now | mutually exclusive mit `departure` |
| `earlierThan` | string | – | mutually exclusive mit departure/arrival |
| `laterThan` | string | – | dito |
| `viaId` | string | – | Alias auf `via.id` |
| `results` | int | `3` |
| `stopovers` | bool | `false` |
| `transfers` | int | Server entscheidet |
| `transferTime` | int | `0` (Minuten) |
| `accessibility` | enum | – | `partial`/`complete` (HAFAS) |
| `bike` | bool | `false` |
| `startWithWalking` | bool | `true` |
| `walkingSpeed` | enum | `normal` | `slow`/`normal`/`fast` |
| `tickets` | bool | `false` | db: nur `/journeys/{ref}` |
| `polylines` | bool | `false` | db: nur `/journeys/{ref}` |
| `subStops`, `entrances` | bool | `true` | db-vendo: not supported |
| `remarks` | bool | `true` |
| `scheduledDays` | bool | `false` | db-vendo liefert Feld `serviceDays` statt `scheduledDays`! |
| `language` | string | `en` |
| **DB-spezifisch:** `loyaltyCard` (enum: `bahncard-{1st,2nd}-{25,50,100}`, `vorteilscard`, `halbtaxabo`, `generalabonnement-{1st,2nd}`, `nl-40`, `at-klimaticket`), `firstClass` (bool), `age` (int), `ageGroup` (enum `B`,`E`,`K`,`S`,`Y`), `routingMode` (enum `FULL`,`HYBRID`,`INFOS`,`OFF`,`REALTIME`,`SERVER_DEFAULT`), `notOnlyFastRoutes` (bool), `bestprice` (bool), `deutschlandTicketConnectionsOnly` (bool, laut api.html) |
| `profile` | enum | `dbnav` | nur db-Instanz |

Antwort: `{ journeys?: Journey[], earlierRef?, laterRef?, realtimeDataUpdatedAt? }`
(+ `Link`-Header-Pagination). Hinweis db-vendo: Pagination ist im
Default-RoutingMode eingeschränkt; `HYBRID` liefert vollständige Pagination und
enthält stornierte Verbindungen.

### GET /journeys/{ref}  (Refresh Journey)

`ref` = `refreshToken` einer vorherigen Journey (URL-encoded).

| Parameter | Typ | Default |
|---|---|---|
| `stopovers` | bool | `false` |
| `tickets` | bool | `false` | mutually exclusive mit `polylines` |
| `polylines` | bool | `false` | mutually exclusive mit `tickets` |
| `subStops`, `entrances` | bool | `true` |
| `remarks` | bool | `true` |
| `scheduledDays` | bool | `false` |
| `notOnlyFastRoutes`, `bestprice` | bool | `false` | db-spezifisch |
| `language` | string | `en` |

Antwort: `{ journey: Journey, realtimeDataUpdatedAt? }`.

### GET /trips/{id}

| Parameter | Typ | Default |
|---|---|---|
| `stopovers` | bool | `true` |
| `remarks` | bool | `true` |
| `polyline` | bool | `false` |
| `language` | string | `en` |

Antwort: `{ trip: Trip, realtimeDataUpdatedAt? }`.

## 5. Capability-abhängige Endpunkte (nicht überall verfügbar)

Diese Endpunkte existieren nur auf HAFAS-basierten Instanzen (BVG, VBB, poland…),
nicht auf `v6.db.transport.rest`:

### GET /radar  (capability: `radar`)
Parameter: `north`, `west`, `south`, `east` (required, BBox), `results`=256, `frames`=3,
`duration`=20 s, `polylines`, Produktfilter.
Antwort: `{ movements: Movement[], realtimeDataUpdatedAt? }` (GeoJSON-Features möglich).

### GET /stops/reachable-from  (capability: `reachableFrom`)
Parameter: `latitude`, `longitude` (required), `when`, `maxTransfers`=5, `maxDuration`=20 min,
Produktfilter.
Antwort: `{ reachable: Duration[], realtimeDataUpdatedAt? }`.

### GET /trips  (capability: `tripsByName`)
Parameter: `query` (Name, default `*`), `fromWhen`, `untilWhen`, `onlyCurrentlyRunning`,
`currentlyStoppingAt`, `lineName`, `operatorNames`, Produktfilter.
Antwort: `{ trips: Trip[], realtimeDataUpdatedAt? }`.

## 6. Instanzspezifische Zusatz-Endpunkte

### GET /stations, GET /stations/{id}  (nur DB; capability: `stations`)
Stationen aus dem `db-stations`-Datensatz (kein Live-HAFAS).
`/stations` Parameter: `query` (Autocomplete), `results`=3 (limit), `fuzzy`=false,
`completion`=true, `fields` (Projektion), Format `application/x-ndjson` möglich.
`/stations/{id}`: 404 wenn unbekannt.

### GET /stops  (nur BVG; capability: `stopsSearch`)
Namenssuche über `vbb-stations`-Datensatz: `query`, `limit`=5, `fuzzy`=false,
`completion`=true. Antwort: Array aus Stops.

## 7. Datenmodelle (aus der OpenAPI-Spec, FPTF-Format)

Wichtige Schemas und ihre Robustheits-Relevanz für einen Client:

- **Location**: `type:"location"`, `id?`, `name?`, `address?`, `poi?`,
  `latitude?`, `longitude?`, `altitude?`, `distance?` (Meter, bei nearby).
- **Stop**: `type:"stop"`, `id`, `name?`, `location?`, `station?` (übergeordnete
  Station), `products?` (Map name→bool), `ids?` (freie Map, z. B. `dhid`),
  `lines?`, `entrances?`, `reisezentrumOpeningHours?`, `loadFactor?`,
  `transitAuthority?`, `isMeta?`, `distance?`.
- **Station**: wie Stop, zusätzlich `stops?` (Sub-Stops), `regions?`, `facilities?`.
- **Line**: `type:"line"`, `id`, `name?`, `fahrtNr?`, `additionalName?`,
  `product?` (profilabhängiger String), `mode?` (enum: aircraft, bicycle, bus,
  car, gondola, taxi, train, walking, watercraft), `operator?`, `express?`,
  `metro?`, `night?`, `nr?`, `symbol?`, `directions?`, `routes?`, `adminCode?`,
  `productName?`, `public?`.
- **Operator**: `type:"operator"`, `id`, `name?`.
- **Departure/Arrival** (Spec-Name `Alternative`): `tripId`, `stop?`, `line?`,
  `direction?`, `provenance?`, `origin?`, `destination?`, `when?`, `plannedWhen?`,
  `prognosedWhen?`, `delay?` (Sekunden/null), `platform?`, `plannedPlatform?`,
  `prognosedPlatform?`, `cancelled?`, `loadFactor?`, `prognosisType?`,
  `remarks[]`, `previousStopovers?`, `nextStopovers?`, `frames?`, `polyline?`,
  `currentTripPosition?`, `location?` (aktuelle Fahrzeugposition bei Radar).
- **Stopover**: `stop`, `arrival?` (null am Anfang), `departure?` (null am Ende),
  `plannedArrival/departure?`, `arrivalDelay/departureDelay?`,
  `prognosedArrival/departure?`, Plattform-Varianten, `passBy?`, `cancelled?`,
  `additional?`, PrognosisTypes, `remarks[]`.
- **Leg**: wie Trip plus `reachable?`, `checkin?`, `cycle?`, `alternatives?`;
  Laufwege haben `walking:true` und ggf. `distance` statt `line`.
- **Journey**: `type:"journey"`, `legs[]`, `refreshToken`, `remarks[]`,
  `price? {amount, currency, hint}`, `cycle?`, `scheduledDays?`/
  db-vendo: `serviceDays` (Map Datum→bool).
- **Trip**: `id`, `origin`, `destination`, Zeit-/Plattform-Felder, `line?`,
  `stopovers?`, `remarks[]`, `polyline?`, `currentLocation?`, `cancelled?`,
  `loadFactor?`, `schedule?`, `operator?` (Achtung: in der Spec fälschlich
  `number` – real Objekt; Client tolerant auslegen).
- **Remark-Varianten** (`Hint`/`Status`/`Warning`): gemeinsame Felder
  `type` (`hint`, `status`, `warning`, `foreign-id`, …), `code?`, `summary?`,
  `text?`, `tripId?`; Warning zusätzlich `priority?`, `category?`, `products?`,
  `edges?`, `events?`, `validFrom/until?`, `modified?`, `company?`,
  `affectedLines?`, `fromStops/toStops?`, `icon?`.
- **PrognosisType**: `calculated`, `prognosed` (offen für weitere Werte).
- **Polyline**: GeoJSON `FeatureCollection` mit `Feature`s
  (`properties` = Stop/Location, `geometry` = Point).
- **Frame** (Radar): `origin`, `destination`, `t` (Offset in Frames).
- **Movement** (Radar): `location`, `line?`, `tripId?`, `direction?`,
  `nextStopovers?`, `frames?`, `polyline?`.
- **Duration** (reachable-from): `duration?` (Sekunden), `stations[]`.

**Robustheitsregeln** (siehe Aufgabe §15):
- Alle Objekt-Felder optional bis auf klar dokumentierte Kernfelder;
  unbekannte Felder werden ignoriert (serde-Default).
- Offene Enums (`mode`, `product`, `prognosisType`, `type`-Discriminators,
  `loadFactor`, Produkte) mit `Other(String)`-Fallback.
- `null` vs. abwesend vs. leer: überall `Option<T>`; `null`-Toleranz via
  `#[serde(default)]` + Double-Option-Pattern wo nötig.
- Zahlen, die eigentlich Objekte sein sollten (`Trip.operator`) und umgekehrt:
  tolerante Deserialisierung.

## 8. Annahmen & dokumentierte Abweichungen

1. `realtimeDataUpdatedAt` ist laut hafas-client-Code ein Unix-Timestamp in
   **Sekunden** (die OpenAPI-Spec sagt nur `number`/`integer`). Modelliert als
   `Option<i64>` ohne Einheiten-Interpretation, Dokumentation verweist auf API.
2. `Trip.operator` ist in der Spec als `number` deklariert, in Realität ein
   Operator-Objekt (Kopie aus Leg). Der Client akzeptiert beides.
3. db-vendo liefert bei `scheduledDays=true` das Feld unter `serviceDays` mit
   anderer Struktur (Map ISO-Datum→bool). Beide Varianten werden gemappt.
4. `direction` bei Departures wird von dbnav/dbweb-Profilen teils ignoriert
   („not supported“) – wir senden ihn trotzdem korrekt kodiert.
5. Die Flixbus- und Nottingham-Instanzen haben abweichende Schemas und sind
   explizit **out of scope**; die Base-URL ist frei konfigurierbar, sodass Nutzer
   kompatible Instanzen selbst ansprechen können.
6. Rate-Limits: 100 req/min ist die dokumentierte Proxy-Grenze; die
   db-vendo-Upstream-APIs sind strenger. Die Library macht deshalb **keine**
   automatischen Retries (außer optional konfigurierbar) und exponiert
   `Retry-After` aus 429-Antworten.
