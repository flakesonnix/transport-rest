# Architektur

## Überblick

```text
transport-rest/
├── schema/                     # Single Source of Truth (IR)
│   ├── base.json               # Metadaten, Provider, Capabilities
│   ├── types.json              # Enums, Aliase, tagged unions
│   ├── models-core.json        # Location/Stop/Station/Line/…
│   ├── models-journey.json     # Journey/Trip/Departure/… + Envelopes
│   └── endpoints.json          # Endpunkte inkl. aller Query-Parameter
│
├── crates/
│   ├── transport-rest/         # Rust Core (Referenzimplementierung)
│   │   ├── src/client.rs       # GET-Pipeline: URL, Limits, Fehlerklassifikation
│   │   ├── src/builder.rs      # Client-Konfiguration
│   │   ├── src/request.rs      # Query-Encoding, JourneyPlace
│   │   ├── src/products.rs     # Produktfilter
│   │   ├── src/api/            # Endpoint-Builder (eine Datei je Ressource)
│   │   ├── src/models/         # typisierte Wire-Modelle (serde)
│   │   └── tests/              # Unit + wiremock-Integrationstests
│   └── transport-rest-gen/     # deterministischer Generator
│       └── src/{schema,ts,python,go,csharp,java,meta}.rs
│
├── bindings/
│   ├── python/                 # pure-Python SDK (sync + asyncio-Fassade)
│   ├── typescript/             # natives TS (fetch), bun:getestet
│   ├── go/                     # net/http, httptest-getestet
│   ├── csharp/                 # HttpClient/System.Text.Json
│   └── java/                   # java.net.http/Jackson
├── examples/
├── docs/
└── .github/workflows/          # CI je Binding + Drift-Check + Release
```

## Datenfluss

1. **Analyse** (`docs/API_ANALYSIS.md`) → Erkenntnisse über transport.rest.
2. **IR** (`schema/*.json`) → maschinenlesbare Destillation der Analyse.
3. **Generator** liest die IR und erzeugt Modelle + `api-meta.json` für alle
   Sprachen. Output ist byte-deterministisch; CI prüft Drift via `--check`.
4. **Rust Core** implementiert Client-Logik handgeschrieben als
   Referenzimplementierung.
5. **Native Bindings** implementieren dieselbe Logik idiomatisch in ihrer
   Sprache – kein FFI, keine nativen Binaries für Endnutzer.

## Entscheidungen & Begründungen

### Warum native Bindings statt FFI in den Rust-Core?

Die Aufgabe lässt beides zu („Falls eine andere Sprache technisch besser
geeignet ist, erkläre die Entscheidung“). Wir haben uns für **native
Implementierungen** entschieden:

* **Keine Distribution-Probleme**: Rust-Binaries müssten pro Plattform/CPU
  gebaut und ausgeliefert werden (cgo, P/Invoke, JNA). Native Bindings sind
  reiner Quellcode bzw. normale Pakete des jeweiligen Ökosystems.
* **Idiomatik**: Go-Nutzer erwarten `context.Context`, C#-Nutzer
  `CancellationToken` und `HttpClient`, Java-Nutzer `java.net.http`. Ein FFI-
  Wrapper könnte das nur nachbilden.
* **Wartbarkeit**: Die Client-Logik ist klein (GET + Query-Encoding +
  Fehlerklassifikation). Der eigentliche Komplexitätsträger – das Schema –
  lebt einmalig in der IR und wird generiert. Drift zwischen den Sprachen
  verhindert der `--check`-Modus im CI.

### Warum ist der Rust-Core handgeschrieben, obwohl es einen Generator gibt?

Der Generator *könnte* auch Rust-Modelle emittieren; serde-tolerante Modelle
(`Other(String)`-Fallbacks, tolerante Zahlen, Unions mit Verbatim-Capture)
sind aber deutlich idiomatischer von Hand geschrieben. Der Rust-Core dient als
ausführbare Spezifikation; alle anderen Sprachen generieren ihre *Typen* und
lesen ihre *Endpunkt-Metadaten* aus der IR.

### Python: pure stdlib statt PyO3

PyO3/maturin war präferiert, ist in dieser Umgebung aber weder baubar
(keine CPython-Header) noch testbar (kein pip). Ein PyO3-Binding hätte zudem
plattformabhängige Wheels zur Folge. Das ausgelieferte Binding ist daher eine
**pure-Python**-Implementierung (nur Standardbibliothek) mit:

* synchroner API (`builder.get()`)
* asyncio-Fassade (`await builder.get_async()` — Thread-Offloading)

Ein PyO3-Pfad bleibt in BINDINGS.md als Alternative dokumentiert und kann im
CI ergänzt werden, sobald python3-dev verfügbar ist.

### TypeScript: natives fetch statt napi-rs

napi-rs würde vorkompilierte Binaries pro Plattform erzwingen und Browser/Deno
ausschließen. `fetch` (Node 18+, Bun, Deno, Browser) macht das Package
universell einsetzbar bei identischer Funktionalität.

## Robustheitsregeln (API-Kompatibilität)

1. **Unbekannte Felder** werden beim Deserialisieren ignoriert
   (Rust: serde-Default; TS: Index-Signatur; Java: `@JsonAnySetter`;
   C#: `[JsonExtensionData]`; Go: per Definition).
2. **Unbekannte Enum-Werte** landen in einem `Other`-Fallback
   (offene Enums laut IR).
3. **Null vs. absent vs. leer** wird überall über Option-Typen abgedeckt.
4. **Tolerante Zahlenformate**: Delays kommen als int oder float.
5. **Tagged unions** (`Stop|Station|Location`) diskriminieren über `type`
   und fassen Unbekanntes verbatim.
6. **Dokumentierte Abweichungen** der Upstream-API sind in
   docs/API_ANALYSIS.md §8 festgehalten (z. B. `Trip.operator`,
   `serviceDays` vs. `scheduledDays`).

## Sicherheit

* TLS-Validierung immer an (rustls / OS-Truststore); keine Insecure-Flags.
* Base-URL muss absolute http(s)-URL sein → kein Path-Traversal in Hostname.
* Pfadsegmente werden strikt percent-encoded (Trip-IDs enthalten `/`, `#`).
* Response-Limit (Default 16 MiB) gegen oversized/hostile Responses,
  streaming-basiert geprüft.
* Keine Secrets im Code; Proxy-Konfiguration optional über den Builder.
* SSRF: Die Library kontaktiert ausschließlich die konfigurierte Base-URL;
  keine Nutzer-kontrollierten Redirect-Ziele (Redirect-Limit 5).
