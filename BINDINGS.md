# Bindings

Dieser Artikel beschreibt die technischen Entscheidungen je Sprache:
Binding-Technologie, Async-Support, Fehler-Mapping, Packaging und Tests.

## Übersicht

| Sprache | Technologie | Native Abhängigkeit für Nutzer | Async | Getestet via |
|---|---|---|---|---|
| Rust | handgeschriebener Core (reqwest + rustls) | nein (crates.io) | tokio-Futures, Cancellation durch Drop | `cargo test` (wiremock) |
| Python | **pure stdlib** (urllib + asyncio-Fassade) | nein | `get_async()` via Thread-Pool | `unittest` (http.server) |
| TypeScript | natives `fetch` | nein | Promises / AbortController | `bun test` + `tsc --strict` |
| Go | net/http | nein | `context.Context` | `go test` (httptest) |
| C# | HttpClient + System.Text.Json | nein | Tasks + CancellationToken | xunit (CI) |
| Java | java.net.http + Jackson | Jackson (Standard im JVM-Ökosystem) | blockierend (v2: CompletableFuture) | JUnit 5 (CI) |

## Warum keine FFI-Bindings (PyO3/napi-rs/UniFFI)?

Die Aufgabe präferiert PyO3/napi-rs/UniFFI „falls sinnvoll“. Nach Abwägung
wurden native Implementierungen gewählt:

1. **Distribution**: FFI bedeutet plattformspezifische Binaries
   (Linux/macOS/Windows × glibc/musl × x86_64/aarch64). Native Pakete sind
   reine Quellpakete und laufen überall.
2. **Async-Mapping**: Ein Rust-Core mit tokio müsste in jedes Sprach-Runtime
   gebrückt werden (pyo3-async-runtimes, napi tokio-Tasks, …). Jede Brücke ist
   ein eigener Fehler-/Cancellation-Komplex. Natives async ist in jeder
   Sprache idiomatischer.
3. **Fehler-Mapping**: Strukturierte Fehler lassen sich in jeder Zielsprache
   direkt als Exception/Error-Typ ausdrücken; über FFI wären sie serialisierte
   Strings.
4. **Schema statt Code als SSOT**: Die Komplexität liegt im *Datenmodell*
   (~40 Typen, ~20 Endpunkte), nicht in der HTTP-Logik. Der Generator erzeugt
   die Typen aus `schema/*.json`; der `--check`-Drift-Guard verhindert,
   dass Bindings auseinanderlaufen.

Die Rust-Core-Crate bleibt Referenzimplementierung und selbst das
Rust-Angebot dieses Repos.

## Rust (crates/transport-rest)

* **HTTP**: reqwest 0.12, rustls (keine OpenSSL-Abhängigkeit), HTTP/2,
  gzip/brotli.
* **Timeouts**: 30 s Request / 10 s Connect (konfigurierbar).
* **Connection Reuse**: implizit über den reqwest-Pool; Client ist `Clone`
  und teilt den Pool.
* **Proxy**: `TransportRestClientBuilder::proxy(reqwest::Proxy)`.
* **Cancellation**: Futures sind Drop-safe abgebrochen.
* **Kein `unwrap`/`panic`** in Library-Code (clippy-lintet dagegen;
  einzige dokumentierte Ausnahme: TLS-Init in `new()`).

### Packaging
crates.io (`transport-rest`), Semantic Versioning, Release-Job in
`.github/workflows/release.yml`.

## Python (bindings/python)

**Abweichung von der Präferenz**: PyO3/maturin war angedacht; ohne
CPython-Header/pip in dieser Umgebung untestbar, und Wheels würden native
Binaries für Endnutzer bedeuten. Gewählt: pure stdlib.

* **Sync API**: `client.locations().query("Berlin").get()`
* **Async API**: `await client.locations().query("Berlin").get_async()`
  (Offloading in einen Thread-Pool; sicher innerhalb laufender Event-Loops)
* **Fehler**: Exception-Hierarchie in `errors.py`, inklusive
  `RateLimitedError.retry_after`.
* **Timeouts**: Socket-Timeout pro Request.
* **Typing**: `py.typed` + generierte Dataclasses in `models_gen.py`;
  unbekannte Felder werden ignoriert.

### Packaging
PyPI-Projekt `transport-rest` (setuptools, pyproject.toml). Release-Job baut
Sdist/Wheel und lädt via twine hoch.

**Geplanter PyO3-Pfad** (optional, CI-only): Crate `bindings/python-native`
mit pyo3 + pyo3-async-runtimes; Sync-API über eine dedizierte Runtime-Thread,
Async-API via `future_into_py`. Aufnahme sobald python3-dev in CI verfügbar
ist — die öffentliche API bleibt identisch.

## TypeScript (bindings/typescript)

* **fetch-basiert**, Node 18+/Bun/Deno/Browser, null Laufzeitabhängigkeiten.
* **Timeouts**: AbortController pro Request.
* **Streaming-Limit**: Response-Reader erzwingt `maxResponseBytes`.
* **Modelle**: generierte `models.ts` (Interfaces + offene String-Unions);
  Unions als Diskriminierungen auf `type`.
* **Fehler**: Klassen-Hierarchie (`ApiError`, `RateLimitedError` mit
  `retryAfterSeconds`, …).

### Packaging
npm `transport-rest`. Es wird TS-Quellcode publiziert (`types` zeigt direkt
auf `.ts`) — kompatibel mit ts-node/tsx/bun/esbuild; ein dist/-Build kann bei
Bedarf ergänzt werden.

## Go (bindings/go)

* **net/http**, kontextbasierte Builder
  (`client.Locations(ctx).Query("Berlin").Get()`).
* **Fehler**: konkrete Typen (`*ApiError`, `*RateLimitedError` mit
  `RetryAfter`, …) hinter dem `TransportRestError`-Interface (`errors.Is/
  As`-kompatibel).
* **Modelle**: generierte `models_gen.go` (Pointer = nullable),
  `gofmt`-sauber.

### Packaging
Go-Module via Tag (`bindings/go/go.mod`, Submodul-Pfad). CI führt
`gofmt -l`, `go vet` und `go test` aus.

## C# (bindings/csharp)

* **HttpClient** (injizierbar für IHttpClientFactory-Szenarien),
  System.Text.Json (kein Newtonsoft).
* **Nullable reference types** aktiviert; `[JsonExtensionData]` fängt
  zukünftige Felder.
* **Async**: `GetAsync(CancellationToken)`.
* **Fehler**: Exception-Hierarchie inkl. `RateLimitedException.RetryAfter`.

### Packaging
NuGet `TransportRest` (`dotnet pack` im Release-Job).

## Java (bindings/java)

* **java.net.http.HttpClient** (JDK 17+, kein Apache-HttpClient nötig),
  **Jackson** für JSON (De-facto-Standard).
* **Sync API** (blockierend); CompletableFuture-Variante geplant für v2.
* **Fehler**: verschachtelte Klassen in `Errors` (eine public Top-Level-Klasse
  pro Datei gemäß Java-Sprachregel).
* **Modelle**: generierte Klassen mit `@JsonIgnoreProperties(ignoreUnknown)`
  + `@JsonAnySetter`-Fallback-Map.

### Packaging
Maven Central (`io.transportrest:transport-rest`); Deployment benötigt
GPG/Sonatype-Konfiguration, siehe CONTRIBUTING.md.

## Teststrategie aller Bindings

Jedes Binding deckt dieselbe Matrix offline ab:

1. Client erstellen (Provider/Capabilities/Base-URL-Validierung)
2. Request stellen & Parameter-Encoding prüfen (Query, Produkte, Pagination)
3. Response deserialisieren (inkl. unbekannter Felder)
4. Fehler behandeln: API 4xx, 429+Retry-After, non-JSON 5xx,
   malformed JSON, Timeout
5. Capability-Gating (radar auf DB vs. BVG)
6. Async-Verhalten (wo vorhanden)

Alle Mocks laufen lokal (wiremock / http.server / Bun.serve / httptest /
HttpListener / com.sun.net.httpserver) — keine Tests hängen vom Internet ab.
Ein separates optionaler Job kann Live-Smoke-Tests gegen
`v6.db.transport.rest` ausführen (manuell getriggert).
