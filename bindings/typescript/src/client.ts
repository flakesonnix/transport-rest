/** transport.rest client core (fetch-based; Node 18+, Bun, Deno, browsers). */

import {
  ApiError,
  CapabilityNotSupportedError,
  HttpError,
  NetworkError,
  RateLimitedError,
  RequestTimeoutError,
  SerializationError,
} from "./errors.js";
import type { JourneyPlace as JourneyPlaceT, QueryParam } from "./util.js";
import {
  DeparturesBuilder,
  JourneysBuilder,
  LocationsBuilder,
  NearbyBuilder,
  RadarBuilder,
  ReachableFromBuilder,
  RefreshJourneyBuilder,
  StationBuilder,
  StationsBuilder,
  StopsSearchBuilder,
  StopBuilder,
  TripBuilder,
  TripsByNameBuilder,
} from "./builders.js";

export const PROVIDERS = {
  db: "https://v6.db.transport.rest",
  bvg: "https://v6.bvg.transport.rest",
  vbb: "https://v6.vbb.transport.rest",
  poland: "https://poland.transport.rest",
} as const;

export type ProviderId = keyof typeof PROVIDERS;

export type Capability =
  | "radar"
  | "reachable_from"
  | "trips_by_name"
  | "stations"
  | "stops_search";

const PROVIDER_CAPABILITIES: Record<ProviderId, Capability[]> = {
  db: ["stations"],
  bvg: ["stops_search", "radar", "reachable_from", "trips_by_name"],
  vbb: ["stops_search", "radar", "reachable_from", "trips_by_name"],
  poland: ["radar", "reachable_from", "trips_by_name"],
};

export interface ClientOptions {
  provider?: ProviderId;
  /** Overrides the provider default (self-hosted instances, tests). */
  baseUrl?: string;
  /** Overall request timeout in milliseconds (default 30s). */
  timeoutMs?: number;
  userAgent?: string;
  /** Guard against oversized responses (default 16 MiB). */
  maxResponseBytes?: number;
  /** Force-enable endpoint groups for instances that support them. */
  enableCapabilities?: Capability[];
  /** Custom fetch implementation (tests, proxies, SSRF-hardened agents). */
  fetchImpl?: typeof fetch;
}

/** Parse function turning raw JSON into typed models. */
export type ModelParser<T> = (data: unknown) => T;

export class TransportRestClient {
  private readonly baseUrl: string;
  private readonly timeoutMs: number;
  private readonly userAgent: string;
  private readonly maxResponseBytes: number;
  private readonly capabilities: Set<Capability>;
  private readonly fetchImpl: typeof fetch;
  private readonly providerId: ProviderId;

  constructor(options: ClientOptions = {}) {
    const providerId = options.provider ?? "db";
    const baseUrl = options.baseUrl ?? PROVIDERS[providerId];
    if (!/^https?:\/\//.test(baseUrl)) {
      throw new Error(`invalid baseUrl '${baseUrl}': must be an absolute http(s) URL`);
    }
    this.providerId = providerId;
    this.baseUrl = baseUrl.replace(/\/+$/, "");
    this.timeoutMs = options.timeoutMs ?? 30_000;
    this.userAgent = options.userAgent ?? "transport-rest-js/0.1.0";
    this.maxResponseBytes = options.maxResponseBytes ?? 16 * 1024 * 1024;
    this.fetchImpl = options.fetchImpl ?? fetch;
    this.capabilities = new Set([
      ...(PROVIDER_CAPABILITIES[providerId] ?? []),
      ...(options.enableCapabilities ?? []),
    ]);
  }

  get baseUrl_(): string {
    return this.baseUrl;
  }

  get provider(): string {
    return this.providerId;
  }

  supports(capability: Capability): boolean {
    return this.capabilities.has(capability);
  }

  // -- resource accessors ---------------------------------------------------

  locations(): LocationsBuilder {
    return new LocationsBuilder(this);
  }
  nearby(): NearbyBuilder {
    return new NearbyBuilder(this);
  }
  stop(id: string): StopBuilder {
    return new StopBuilder(this, id);
  }
  departures(stopId: string): DeparturesBuilder {
    return DeparturesBuilder.create(this, stopId, "departures");
  }
  arrivals(stopId: string): DeparturesBuilder {
    return DeparturesBuilder.create(this, stopId, "arrivals");
  }
  journeys(from: JourneyPlaceT, to: JourneyPlaceT): JourneysBuilder {
    return new JourneysBuilder(this, from, to);
  }
  refreshJourney(refreshToken: string): RefreshJourneyBuilder {
    return new RefreshJourneyBuilder(this, refreshToken);
  }
  trip(id: string): TripBuilder {
    return new TripBuilder(this, id);
  }
  radar(): RadarBuilder {
    return new RadarBuilder(this);
  }
  reachableFrom(): ReachableFromBuilder {
    return new ReachableFromBuilder(this);
  }
  tripsByName(query: string): TripsByNameBuilder {
    return new TripsByNameBuilder(this, query);
  }
  stations(): StationsBuilder {
    return new StationsBuilder(this);
  }
  station(id: string): StationBuilder {
    return new StationBuilder(this, id);
  }
  stopsSearch(): StopsSearchBuilder {
    return new StopsSearchBuilder(this);
  }

// -- execution ------------------------------------------------------------

  checkCapability(capability: Capability): void {
    if (!this.capabilities.has(capability)) {
      throw new CapabilityNotSupportedError(capability, this.providerId);
    }
  }

  async get_json<T>(
    path: string,
    params: QueryParam[],
    model: ModelParser<T>,
    capability?: Capability,
  ): Promise<T> {
    if (capability) this.checkCapability(capability);
    const qs = params.map(([k, v]) => `${encodeURIComponent(k)}=${encodeURIComponent(v)}`).join("&");
    const url = `${this.baseUrl}${path}${qs ? `?${qs}` : ""}`;
    const controller = new AbortController();
    const timer = setTimeout(() => controller.abort(), this.timeoutMs);
    let response: Response;
    try {
      response = await this.fetchImpl(url, {
        headers: { Accept: "application/json", "User-Agent": this.userAgent },
        signal: controller.signal,
      });
    } catch (err) {
      if (controller.signal.aborted) {
        throw new RequestTimeoutError("request", url);
      }
      throw new NetworkError(url, err);
    } finally {
      clearTimeout(timer);
    }

    if (!response.ok) {
      throw await classifyError(response, url);
    }
    const data = await readCapped(response, this.maxResponseBytes, url);
    try {
      return model(JSON.parse(data));
    } catch (err) {
      if (err instanceof SyntaxError) {
        throw new SerializationError(`body is not valid JSON: ${String(err)}`, url);
      }
      throw new SerializationError(`response did not match expected schema: ${String(err)}`, url);
    }
  }
}

async function readCapped(response: Response, max: number, url: string): Promise<string> {
  const declared = response.headers.get("content-length");
  if (declared && Number(declared) > max) {
    throw new SerializationError(
      `response of ${declared} bytes exceeds configured maximum of ${max}`,
      url,
    );
  }
  const reader = response.body?.getReader();
  if (!reader) return await response.text();
  const chunks: Uint8Array[] = [];
  let total = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    total += value.byteLength;
    if (total > max) {
      void reader.cancel().catch(() => undefined);
      throw new SerializationError(
        `response exceeds configured maximum of ${max}`,
        url,
      );
    }
    chunks.push(value);
  }
  const merged = new TextDecoder("utf-8", { fatal: false }).decode(
    concat(chunks),
  );
  return merged;
}

function concat(chunks: Uint8Array[]): Uint8Array {
  const total = chunks.reduce((n, c) => n + c.byteLength, 0);
  const out = new Uint8Array(total);
  let offset = 0;
  for (const chunk of chunks) {
    out.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return out;
}

function snippet(text: string): string {
  return text.length > 512 ? `${text.slice(0, 512)}…` : text;
}

async function classifyError(response: Response, url: string): Promise<TransportRestErrorLike> {
  const status = response.status;
  const bodyText = (await response.text()).trim();
  let parsed: unknown;
  try {
    parsed = bodyText ? JSON.parse(bodyText) : undefined;
  } catch {
    parsed = undefined;
  }
  const message =
    parsed && typeof parsed === "object" && "message" in parsed
      ? String((parsed as { message: unknown }).message)
      : undefined;

  if (status === 429) {
    const retryAfter = response.headers.get("retry-after");
    const seconds = retryAfter && /^\d+$/.test(retryAfter.trim()) ? Number(retryAfter.trim()) : undefined;
    return new RateLimitedError(url, message ?? "rate limited", parsed, seconds);
  }
  if (parsed !== undefined && message !== undefined) {
    return new ApiError(status, url, message, parsed);
  }
  if (parsed !== null && parsed !== undefined && typeof parsed === "object") {
    return new ApiError(status, url, "unspecified API error", parsed);
  }
  return new HttpError(status, "GET", url, snippet(bodyText));
}

type TransportRestErrorLike =
  | ApiError
  | RateLimitedError
  | HttpError
  | SerializationError
  | RequestTimeoutError
  | NetworkError;
