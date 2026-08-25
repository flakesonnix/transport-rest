/** Endpoint builders mirroring the Rust core API. */

import type { TransportRestClient, Capability, ModelParser } from "./client.js";
import { ApiError } from "./errors.js";
import {
  bool as b,
  iso,
  require,
  JourneyPlace,
  encodePathSegment,
  type QueryParam,
} from "./util.js";
import type {
  ArrivalsResponse,
  DeparturesResponse,
  JourneysResponse,
  JourneyResponse,
  LocationResult,
  RadarResponse,
  ReachableFromResponse,
  Station,
  StopOrStation,
  TripResponse,
  TripsResponse,
} from "./models.js";

export { JourneyPlace };

function parseLocations(data: unknown): LocationResult[] {
  if (!Array.isArray(data)) throw new TypeError("expected array of locations");
  return data as LocationResult[];
}

function parseStations(data: unknown): Station[] {
  if (!Array.isArray(data)) throw new TypeError("expected array of stations");
  return data as Station[];
}

export class Builder<T> {
  constructor(
    protected readonly client: TransportRestClient,
    protected readonly path: string,
    private readonly model: ModelParser<T>,
    protected readonly capability?: Capability,
  ) {}

  protected params: QueryParam[] = [];

  protected opt(key: string, value: string | number | undefined): void {
    if (value !== undefined) this.params.push([key, String(value)]);
  }

  get(): Promise<T> {
    return this.client.get_json(this.path, this.params, this.model, this.capability);
  }
}

export class LocationsBuilder extends Builder<LocationResult[]> {
  constructor(client: TransportRestClient) {
    super(client, "/locations", parseLocations);
  }

  query(q: string): this {
    require(q && q.trim(), "query", "a non-empty search term is required");
    this.params.unshift(["query", q]);
    return this;
  }
  fuzzy(value: boolean): this {
    this.opt("fuzzy", b(value));
    return this;
  }
  results(n: number): this {
    this.opt("results", n);
    return this;
  }
  stops(value: boolean): this {
    this.opt("stops", b(value));
    return this;
  }
  addresses(value: boolean): this {
    this.opt("addresses", b(value));
    return this;
  }
  poi(value: boolean): this {
    this.opt("poi", b(value));
    return this;
  }
  linesOfStops(value: boolean): this {
    this.opt("linesOfStops", b(value));
    return this;
  }
  language(language: string): this {
    this.opt("language", language);
    return this;
  }

  override async get(): Promise<LocationResult[]> {
    const q = this.params.find(([k]) => k === "query");
    require(q !== undefined, undefined, "query() is required before get()");
    return await super.get();
  }
}

export class NearbyBuilder extends Builder<LocationResult[]> {
  private lat?: number;
  private lon?: number;

  constructor(client: TransportRestClient) {
    super(client, "/locations/nearby", parseLocations);
  }

  latitude(v: number): this {
    require(v >= -90 && v <= 90, "latitude", "must be within [-90, 90]");
    this.lat = v;
    return this;
  }
  longitude(v: number): this {
    require(v >= -180 && v <= 180, "longitude", "must be within [-180, 180]");
    this.lon = v;
    return this;
  }
  results(n: number): this {
    this.opt("results", n);
    return this;
  }
  distance(meters: number): this {
    this.opt("distance", meters);
    return this;
  }
  stops(value: boolean): this {
    this.opt("stops", b(value));
    return this;
  }
  poi(value: boolean): this {
    this.opt("poi", b(value));
    return this;
  }
  language(language: string): this {
    this.opt("language", language);
    return this;
  }

  override async get(): Promise<LocationResult[]> {
    require(
      this.lat !== undefined && this.lon !== undefined,
      undefined,
      "latitude and longitude are both required",
    );
    this.params.unshift(["longitude", String(this.lon)]);
    this.params.unshift(["latitude", String(this.lat)]);
    return await super.get();
  }
}

export class StopBuilder extends Builder<StopOrStation> {
  constructor(client: TransportRestClient, id: string) {
    super(client, `/stops/${encodePathSegment(id)}`, (d) => d as StopOrStation);
    require(id && id.trim(), "id", "stop ID must not be empty");
  }

  linesOfStops(value: boolean): this {
    this.opt("linesOfStops", b(value));
    return this;
  }
  language(language: string): this {
    this.opt("language", language);
    return this;
  }
}

interface BoardState {
  when?: Date;
  direction?: string;
  duration?: number;
  results?: number;
  stopovers?: boolean;
  includeRelatedStations?: boolean;
  linesOfStops?: boolean;
  remarks?: boolean;
  language?: string;
  moreStops?: string[];
  products?: ProductSelection;
}

export class ProductSelection {
  private readonly entries: QueryParam[] = [];

  set(key: string, enabled: boolean): this {
    const existing = this.entries.findIndex(([k]) => k === key);
    if (existing >= 0) this.entries[existing] = [key, b(enabled)];
    else this.entries.push([key, b(enabled)]);
    return this;
  }
  nationalExpress(v: boolean): this { return this.set("nationalExpress", v); }
  national(v: boolean): this { return this.set("national", v); }
  regionalExpress(v: boolean): this { return this.set("regionalExpress", v); }
  regional(v: boolean): this { return this.set("regional", v); }
  suburban(v: boolean): this { return this.set("suburban", v); }
  subway(v: boolean): this { return this.set("subway", v); }
  tram(v: boolean): this { return this.set("tram", v); }
  bus(v: boolean): this { return this.set("bus", v); }
  ferry(v: boolean): this { return this.set("ferry", v); }
  taxi(v: boolean): this { return this.set("taxi", v); }
  express(v: boolean): this { return this.set("express", v); }

  encodeInto(params: QueryParam[]): void {
    params.push(...this.entries);
  }
}

export class DeparturesBuilder extends Builder<DeparturesResponse | ArrivalsResponse> {
  private readonly board: BoardState = {};
  private productsSelection?: ProductSelection;

  static create(
    client: TransportRestClient,
    stopId: string,
    kind: "departures" | "arrivals",
  ): DeparturesBuilder {
    require(stopId && stopId.trim(), "stopId", "must not be empty");
    const model: ModelParser<DeparturesResponse> =
      kind === "departures"
        ? (d) => d as DeparturesResponse
        : (d) => d as unknown as DeparturesResponse;
    return new DeparturesBuilder(
      client,
      `/stops/${encodePathSegment(stopId)}/${kind}`,
      model,
    );
  }

  private constructor(client: TransportRestClient, path: string, model: ModelParser<DeparturesResponse>) {
    super(client, path, model);
  }

  when(when: Date): this {
    this.board.when = when;
    return this;
  }
  direction(direction: string): this {
    this.board.direction = direction;
    return this;
  }
  duration(minutes: number): this {
    this.board.duration = minutes;
    return this;
  }
  results(n: number): this {
    this.board.results = n;
    return this;
  }
  stopovers(value: boolean): this {
    this.board.stopovers = value;
    return this;
  }
  includeRelatedStations(value: boolean): this {
    this.board.includeRelatedStations = value;
    return this;
  }
  linesOfStops(value: boolean): this {
    this.board.linesOfStops = value;
    return this;
  }
  remarks(value: boolean): this {
    this.board.remarks = value;
    return this;
  }
  language(language: string): this {
    this.board.language = language;
    return this;
  }
  moreStops(stopIds: string[]): this {
    this.board.moreStops = stopIds;
    return this;
  }
  products(configure: (p: ProductSelection) => ProductSelection): this {
    this.productsSelection = configure(new ProductSelection());
    return this;
  }

  override async get(): Promise<DeparturesResponse> {
    if (this.board.when) this.params.unshift(["when", iso(this.board.when)]);
    if (this.board.direction !== undefined) this.params.unshift(["direction", this.board.direction]);
    if (this.board.duration !== undefined) this.params.unshift(["duration", String(this.board.duration)]);
    if (this.board.results !== undefined) this.params.unshift(["results", String(this.board.results)]);
    if (this.board.stopovers !== undefined) this.params.unshift(["stopovers", b(this.board.stopovers)]);
    if (this.board.includeRelatedStations !== undefined)
      this.params.unshift(["includeRelatedStations", b(this.board.includeRelatedStations)]);
    if (this.board.linesOfStops !== undefined)
      this.params.unshift(["linesOfStops", b(this.board.linesOfStops)]);
    if (this.board.remarks !== undefined) this.params.unshift(["remarks", b(this.board.remarks)]);
    if (this.board.language !== undefined) this.params.unshift(["language", this.board.language]);
    if (this.board.moreStops?.length) this.params.unshift(["moreStops", this.board.moreStops.join(",")]);
    this.productsSelection?.encodeInto(this.params);
    return (await super.get()) as DeparturesResponse;
  }
}

export class JourneysBuilder extends Builder<JourneysResponse> {
  // NOTE: field names deliberately avoid clashing with the fluent method
  // names below (useDefineForClassFields would otherwise shadow them).
  private readonly _from: JourneyPlace;
  private readonly _to: JourneyPlace;
  private _via?: JourneyPlace;
  private _departure?: Date;
  private _arrival?: Date;
  private _earlierThan?: string;
  private _laterThan?: string;
  private readonly options: QueryParam[] = [];
  private productsSelection?: ProductSelection;

  constructor(client: TransportRestClient, from: JourneyPlace, to: JourneyPlace) {
    super(client, "/journeys", (d) => d as JourneysResponse);
    this._from = from;
    this._to = to;
  }

  via(place: JourneyPlace): this {
    this._via = place;
    return this;
  }
  departure(when: Date): this {
    this._departure = when;
    return this;
  }
  arrival(when: Date): this {
    this._arrival = when;
    return this;
  }
  earlierThan(ref: string): this {
    this._earlierThan = ref;
    return this;
  }
  laterThan(ref: string): this {
    this._laterThan = ref;
    return this;
  }
  results(n: number): this { this.options.push(["results", String(n)]); return this; }
  stopovers(v: boolean): this { this.options.push(["stopovers", b(v)]); return this; }
  transfers(n: number): this { this.options.push(["transfers", String(n)]); return this; }
  transferTime(minutes: number): this { this.options.push(["transferTime", String(minutes)]); return this; }
  accessibility(value: "partial" | "complete"): this {
    this.options.push(["accessibility", value]);
    return this;
  }
  bike(v: boolean): this { this.options.push(["bike", b(v)]); return this; }
  startWithWalking(v: boolean): this { this.options.push(["startWithWalking", b(v)]); return this; }
  walkingSpeed(speed: "slow" | "normal" | "fast"): this {
    this.options.push(["walkingSpeed", speed]);
    return this;
  }
  tickets(v: boolean): this { this.options.push(["tickets", b(v)]); return this; }
  polylines(v: boolean): this { this.options.push(["polylines", b(v)]); return this; }
  remarks(v: boolean): this { this.options.push(["remarks", b(v)]); return this; }
  scheduledDays(v: boolean): this { this.options.push(["scheduledDays", b(v)]); return this; }
  notOnlyFastRoutes(v: boolean): this { this.options.push(["notOnlyFastRoutes", b(v)]); return this; }
  bestprice(v: boolean): this { this.options.push(["bestprice", b(v)]); return this; }
  loyaltyCard(card: string): this { this.options.push(["loyaltyCard", card]); return this; }
  firstClass(v: boolean): this { this.options.push(["firstClass", b(v)]); return this; }
  routingMode(mode: string): this { this.options.push(["routingMode", mode]); return this; }
  products(configure: (p: ProductSelection) => ProductSelection): this {
    this.productsSelection = configure(new ProductSelection());
    return this;
  }

  override async get(): Promise<JourneysResponse> {
    this._from.validate("from");
    this._to.validate("to");
    this._via?.validate("via");
    require(
      !(this._departure && this._arrival),
      undefined,
      "departure and arrival are mutually exclusive",
    );
    require(
      !((this._earlierThan || this._laterThan) && (this._departure || this._arrival)),
      undefined,
      "earlierThan/laterThan cannot be combined with departure/arrival",
    );
    require(
      !(this._earlierThan && this._laterThan),
      undefined,
      "earlierThan and laterThan are mutually exclusive",
    );

    const params: QueryParam[] = [];
    this._from.encode("from", params);
    this._to.encode("to", params);
    if (this._via) this._via.encode("via", params);
    if (this._departure) params.push(["departure", iso(this._departure)]);
    if (this._arrival) params.push(["arrival", iso(this._arrival)]);
    if (this._earlierThan) params.push(["earlierThan", this._earlierThan]);
    if (this._laterThan) params.push(["laterThan", this._laterThan]);
    params.push(...this.options);
    this.productsSelection?.encodeInto(params);
    this.params = params;
    return await super.get();
  }
}

export class RefreshJourneyBuilder extends Builder<JourneyResponse> {
  private ticketsValue?: boolean;
  private polylinesValue?: boolean;

  constructor(client: TransportRestClient, refreshToken: string) {
    super(
      client,
      `/journeys/${encodePathSegment(refreshToken)}`,
      (d) => d as JourneyResponse,
    );
    require(refreshToken && refreshToken.trim(), "refreshToken", "must not be empty");
  }

  stopovers(v: boolean): this { this.opt("stopovers", b(v)); return this; }
  tickets(v: boolean): this { this.ticketsValue = v; return this; }
  polylines(v: boolean): this { this.polylinesValue = v; return this; }
  remarks(v: boolean): this { this.opt("remarks", b(v)); return this; }
  language(language: string): this { this.opt("language", language); return this; }

  override async get(): Promise<JourneyResponse> {
    require(
      !(this.ticketsValue && this.polylinesValue),
      undefined,
      "tickets and polylines are mutually exclusive",
    );
    return await super.get();
  }
}

export class TripBuilder extends Builder<TripResponse> {
  constructor(client: TransportRestClient, id: string) {
    super(client, `/trips/${encodePathSegment(id)}`, (d) => d as TripResponse);
    require(id && id.trim(), "id", "trip ID must not be empty");
  }

  stopovers(v: boolean): this { this.opt("stopovers", b(v)); return this; }
  remarks(v: boolean): this { this.opt("remarks", b(v)); return this; }
  polyline(v: boolean): this { this.opt("polyline", b(v)); return this; }
  language(language: string): this { this.opt("language", language); return this; }
}

export class TripsByNameBuilder extends Builder<TripsResponse> {
  constructor(client: TransportRestClient, query: string) {
    super(client, "/trips", (d) => d as TripsResponse, "trips_by_name");
    this.opt("query", query || "*");
  }

  onlyCurrentlyRunning(v: boolean): this { this.opt("onlyCurrentlyRunning", b(v)); return this; }
  lineName(name: string): this { this.opt("lineName", name); return this; }
}

export class RadarBuilder extends Builder<RadarResponse> {
  private box: Partial<Record<"north" | "west" | "south" | "east", number>> = {};

  constructor(client: TransportRestClient) {
    super(client, "/radar", (d) => d as RadarResponse, "radar");
  }

  north(v: number): this { this.box.north = v; return this; }
  west(v: number): this { this.box.west = v; return this; }
  south(v: number): this { this.box.south = v; return this; }
  east(v: number): this { this.box.east = v; return this; }
  results(n: number): this { this.opt("results", n); return this; }
  frames(n: number): this { this.opt("frames", n); return this; }
  duration(seconds: number): this { this.opt("duration", seconds); return this; }

  override async get(): Promise<RadarResponse> {
    const box = this.box;
    require(
      box.north !== undefined && box.west !== undefined &&
        box.south !== undefined && box.east !== undefined,
      undefined,
      "north, west, south and east are all required",
    );
    require(
      (box.south as number) <= (box.north as number) &&
        (box.west as number) <= (box.east as number),
      undefined,
      "bounding box is invalid: require south <= north and west <= east",
    );
    const ordered: QueryParam[] = [
      ["north", String(box.north)],
      ["west", String(box.west)],
      ["south", String(box.south)],
      ["east", String(box.east)],
    ];
    this.params = [...ordered, ...this.params];
    return (await super.get()) as RadarResponse;
  }
}

export class ReachableFromBuilder extends Builder<ReachableFromResponse> {
  private lat?: number;
  private lon?: number;

  constructor(client: TransportRestClient) {
    super(client, "/stops/reachable-from", (d) => d as ReachableFromResponse, "reachable_from");
  }

  latitude(v: number): this { this.lat = v; return this; }
  longitude(v: number): this { this.lon = v; return this; }
  maxTransfers(n: number): this { this.opt("maxTransfers", n); return this; }
  maxDuration(minutes: number): this { this.opt("maxDuration", minutes); return this; }

  override async get(): Promise<ReachableFromResponse> {
    require(this.lat !== undefined && this.lon !== undefined, undefined, "latitude and longitude are both required");
    this.params.unshift(["longitude", String(this.lon)]);
    this.params.unshift(["latitude", String(this.lat)]);
    return (await super.get()) as ReachableFromResponse;
  }
}

export class StationsBuilder extends Builder<Station[]> {
  constructor(client: TransportRestClient) {
    super(client, "/stations", parseStations, "stations");
  }

  query(q: string): this { this.opt("query", q); return this; }
  results(n: number): this { this.opt("results", n); return this; }
}

export class StationBuilder extends Builder<Station> {
  constructor(client: TransportRestClient, id: string) {
    super(client, `/stations/${encodePathSegment(id)}`, (d) => d as Station, "stations");
    require(id && id.trim(), "id", "station ID must not be empty");
  }
}

export class StopsSearchBuilder extends Builder<LocationResult[]> {
  constructor(client: TransportRestClient) {
    super(client, "/stops", parseLocations, "stops_search");
  }

  query(q: string): this { this.opt("query", q); return this; }
  limit(n: number): this { this.opt("limit", n); return this; }
}
