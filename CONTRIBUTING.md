# Contributing

Danke fürs Mitmachen! Dieses Repo folgt einem schema-getriebenen Workflow:
Änderungen an der API-Oberfläche landen **zuerst** in `schema/*.json` und
werden dann generiert.

## Setup

* Rust stable (`rustup`) – Core + Generator
* Python 3.10+ – Python-Binding (nur stdlib, kein pip nötig)
* Bun oder Node 18+ + npm – TypeScript-Binding
* Go 1.22+, .NET 8 SDK, JDK 17+/Maven – optionale Bindings
  (CI baut & testet sie auch ohne lokale Toolchains)

## Workflow

1. **Schema ändern** (falls API-oberflächlich relevant):
   ```sh
   $EDITOR schema/*.json
   cargo run -p transport-rest-gen -- --schema-dir schema type-script --out bindings/typescript/src
   cargo run -p transport-rest-gen -- --schema-dir schema python     --out bindings/python/transport_rest
   cargo run -p transport-rest-gen -- --schema-dir schema go         --out bindings/go
   cargo run -p transport-rest-gen -- --schema-dir schema csharp     --out bindings/csharp
   cargo run -p transport-rest-gen -- --schema-dir schema java       --out bindings/java
   cargo run -p transport-rest-gen -- --schema-dir schema meta       --out bindings
   ```
2. **Core anpassen** (Rust) und/oder natives Binding ergänzen.
3. **Tests** je betroffenem Binding lokal ausführen (alles offline möglich,
   siehe README „Development“).
4. **Konventionen**:
   - Generierte Dateien niemals von Hand editieren (Header `DO NOT EDIT`).
   - Kleine, nachvollziehbare Commits im Conventional-Commits-Stil
     (`feat(core): …`, `fix(bindings): …`, `docs: …`).
   - Kein `unwrap()`/`panic!()` in Library-Code.
5. **PR**: CI muss grün sein (fmt, clippy `-D warnings`, alle Tests,
   Drift-Check der generierten Dateien).

## Neue Endpunkte aufnehmen

1. Endpunkt in `schema/endpoints.json` dokumentieren (Parameter inkl.
   `required`, Defaults, Capability, dbOnly).
2. Response-Modelle in `schema/models-*.json` ergänzen; offene Enums nutzen.
3. Generator laufen lassen (oben).
4. Rust-Builder in `crates/transport-rest/src/api/` implementieren +
   wiremock-Tests.
5. Native Bindings ergänzen (Go/C#/Java können im selben PR folgen).

## Release-Prozess

1. Versionen bumpen (Workspace + Binding-Manifeste), CHANGELESS? Nein:
   Semantic Versioning — Breaking Changes → Major, neue Endpunkte/Felder →
   Minor, Fixes → Patch.
2. Tag `vX.Y.Z` pushen → `.github/workflows/release.yml` publiziert
   crates.io / PyPI / npm / NuGet; Maven benötigt Sonatype-Zugangsdaten +
   GPG-Signatur (Secrets: `CARGO_REGISTRY_TOKEN`, `PYPI_API_TOKEN`,
   `NPM_TOKEN`, `NUGET_API_KEY`).

## Verhaltensregeln

Sei nett. Data Quality über Datenmenge.
