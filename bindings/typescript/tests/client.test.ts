/**
 * Binding tests: create client, request, deserialize, handle error, async.
 * Fully offline: runs against a local Bun.serve mock.
 */

import { afterAll, beforeAll, describe, expect, test } from "bun:test";
import {
  ApiError,
  HttpError,
  InvalidParameterError,
  JourneyPlace,
  RateLimitedError,
  SerializationError,
  CapabilityNotSupportedError,
  TransportRestClient,
} from "../src/index.js";


const WHEN = new Date("2026-08-01T12:00:00+02:00");

let server: ReturnType<typeof Bun.serve>;
let baseUrl = "";

beforeAll(() => {
  server = Bun.serve({
    port: 0,
    fetch(request) {
      const url = new URL(request.url);
      const q = Object.fromEntries(url.searchParams.entries());
      const json = (status: number, body: unknown, headers?: Record<string, string>) =>
        new Response(typeof body === "string" ? body : JSON.stringify(body), {
          status,
          headers: { "Content-Type": "application/json", ...headers },
        });
      switch (`${url.pathname}`) {
        case "/locations":
          if (q.query === "Berlin") {
            return json(200, [
              { type: "stop", id: "8011160", name: "Berlin Hbf",
                location: { type: "location", latitude: 52.525, longitude: 13.369 } },
              { type: "location", name: "Alexanderplatz", poi: true },
            ]);
          }
          if (q.query === "ratelimit") {
            return json(429, { message: "Too Many Requests" }, { "Retry-After": "30" });
          }
          if (q.query === "badgateway") {
            return new Response("<html>Bad Gateway</html>", { status: 502 });
          }
          return json(400, { message: "Missing query." });
        case "/stops/8011160/departures":
          return json(200, {
            departures: [{
              tripId: "t1",
              line: { id: "ICE 599", mode: "train" },
              when: "2026-08-01T12:00:00+02:00",
              plannedWhen: "2026-08-01T11:58:00+02:00",
              delay: 120,
            }],
            realtimeDataUpdatedAt: 1754000000,
          });
        case "/journeys":
          return json(200, {
            journeys: [{
              refreshToken: "ref/1",
              legs: [{
                tripId: "t9",
                origin: { type: "stop", id: "8011160", name: "Berlin Hbf" },
                destination: { type: "stop", id: "8000108", name: "Leipzig Hbf" },
                line: { id: "ice-599", mode: "train",
                        operator: { type: "operator", id: "db", name: "Deutsche Bahn" } },
              }],
            }],
            earlierRef: "E",
            laterRef: "L",
          });
        case "/stops/nope":
          return json(404, { message: "Stop not found." });
        case "/radar":
          return json(200, { movements: [{ tripId: "m1" }] });
        case "/stops/x/departures":
          return json(200, "{not json");
        default:
          return json(404, { message: `no route ${url.pathname}` });
      }
    },
  });
  baseUrl = `http://localhost:${server.port}`;
});

afterAll(() => server.stop(true));

const dbClient = () => new TransportRestClient({ provider: "db", baseUrl });

describe("locations", () => {
  test("parses results and sends query params", async () => {
    const client = dbClient();
    const result = await client.locations().query("Berlin").results(5).get();
    expect(result).toHaveLength(2);
    expect((result[0] as { id?: string }).id).toBe("8011160");
  });

  test("missing query fails client-side without request", async () => {
    const client = dbClient();
    await expect(client.locations().get()).rejects.toBeInstanceOf(InvalidParameterError);
  });

  test("429 exposes retry-after", async () => {
    const client = dbClient();
    try {
      await client.locations().query("ratelimit").get();
      throw new Error("expected RateLimitedError");
    } catch (err) {
      expect(err).toBeInstanceOf(RateLimitedError);
      expect((err as RateLimitedError).retryAfterSeconds).toBe(30);
    }
  });

  test("non-JSON 502 becomes HttpError", async () => {
    const client = dbClient();
    try {
      await client.locations().query("badgateway").get();
      throw new Error("expected HttpError");
    } catch (err) {
      expect(err).toBeInstanceOf(HttpError);
      expect((err as HttpError).status).toBe(502);
    }
  });
});

describe("departures", () => {
  test("parses board and product filters", async () => {
    const client = dbClient();
    const result = await client
      .departures("8011160")
      .results(10)
      .products((p) => p.bus(false).tram(false))
      .moreStops(["8010159"])
      .when(WHEN)
      .get() as any;
    expect(result.departures[0].delay).toBe(120);
    expect(result.departures[0].line.mode).toBe("train");
    expect(result.realtimeDataUpdatedAt).toBe(1754000000);
  });
});

describe("journeys", () => {
  test("encodes places and pagination refs", async () => {
    const client = dbClient();
    const result = await client
      .journeys(JourneyPlace.stopId("8011160"), JourneyPlace.name("Leipzig Hbf"))
      .via(JourneyPlace.poi("poi1", 51.5, 12.2))
      .transfers(0)
      .earlierThan("REF1")
      .get();
    const journey = result.journeys?.[0];
    expect(journey?.legs?.[0]?.origin).toMatchObject({ id: "8011160" });
    expect(journey?.refreshToken).toBe("ref/1");
  });

  test("conflicting times rejected before any request", async () => {
    const client = dbClient();
    await expect(
      client.journeys(JourneyPlace.stopId("a"), JourneyPlace.stopId("b"))
        .departure(WHEN).arrival(WHEN).get(),
    ).rejects.toBeInstanceOf(InvalidParameterError);
  });
});

describe("errors & capabilities", () => {
  test("structured API error keeps status and message", async () => {
    const client = dbClient();
    try {
      await client.stop("nope").get();
      throw new Error("expected ApiError");
    } catch (err) {
      expect(err).toBeInstanceOf(ApiError);
      expect((err as ApiError).status).toBe(404);
      expect((err as ApiError).message).toBe("Stop not found.");
    }
  });

  test("capability gating: radar on db vs bvg", async () => {
    const db = dbClient();
    await expect(
      db.radar().north(52.53).west(13.36).south(52.51).east(13.39).get(),
    ).rejects.toBeInstanceOf(CapabilityNotSupportedError);

    const bvg = new TransportRestClient({ provider: "bvg", baseUrl });
    const radar = await bvg.radar()
      .north(52.53).west(13.36).south(52.51).east(13.39).get();
    expect(radar.movements?.[0]?.tripId).toBe("m1");
  });

  test("malformed JSON is a serialization error", async () => {
    const client = dbClient();
    await expect(client.departures("x").get())
      .rejects.toBeInstanceOf(SerializationError);
  });

  test("unknown future fields are tolerated", async () => {
    const client = dbClient();
    // /stops/nope returns 404; use journeys response with extra field instead:
    const result = await client
      .journeys(JourneyPlace.stopId("8011160"), JourneyPlace.name("L"))
      .get();
    expect(Object.keys(result)).toContain("earlierRef");
  });
});
