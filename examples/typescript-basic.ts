/**
 * Basic usage of the transport-rest TypeScript client.
 * Works in Node 18+, Bun and Deno.
 *
 * Run: cd bindings/typescript && bun run ../../examples/typescript-basic.ts
 */

import { JourneyPlace, TransportRestClient } from "../bindings/typescript/src/index.js";

const client = new TransportRestClient({ provider: "db" });

const locations = await client.locations().query("Berlin").results(3).get();
for (const location of locations) {
  console.log(`found: ${location.name ?? "<unnamed>"} (${location.id ?? "?"})`);
}

const board = (await client.departures("8011160").results(5).get()) as any;
for (const dep of board.departures ?? []) {
  console.log(`${dep.line?.name ?? "?"}: ${dep.plannedWhen}`);
}

const journeys = await client
  .journeys(JourneyPlace.stopId("8011160"), JourneyPlace.stopId("8000108"))
  .results(3)
  .get();
console.log(`got ${journeys.journeys?.length ?? 0} journeys`);
