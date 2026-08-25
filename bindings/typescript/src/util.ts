/** Shared helpers of the TypeScript client. */

import { InvalidParameterError } from "./errors.js";

export function encodePathSegment(value: string): string {
  return encodeURIComponent(value);
}

export type QueryParam = [string, string];

export function bool(value: boolean): string {
  return value ? "true" : "false";
}

export function iso(when: Date): string {
  return when.toISOString().replace(/\.\d{3}Z$/, "+00:00");
  // transport.rest accepts RFC 3339; plain ISO with Z also works:
}

export function require(condition: unknown, parameter: string | undefined, reason: string): asserts condition {
  if (!condition) {
    throw new InvalidParameterError(parameter, reason);
  }
}

export interface JourneyPlaceForm {
  form: "id" | "name" | "poi" | "address";
  id?: string;
  name?: string;
  latitude?: number;
  longitude?: number;
  address?: string;
}

export class JourneyPlace {
  private constructor(readonly inner: JourneyPlaceForm) {}

  static stopId(id: string): JourneyPlace {
    return new JourneyPlace({ form: "id", id });
  }
  static name(name: string): JourneyPlace {
    return new JourneyPlace({ form: "name", name });
  }
  static poi(id: string, latitude: number, longitude: number): JourneyPlace {
    return new JourneyPlace({ form: "poi", id, latitude, longitude });
  }
  static address(latitude: number, longitude: number, address: string): JourneyPlace {
    return new JourneyPlace({ form: "address", latitude, longitude, address });
  }

  encode(prefix: string, params: QueryParam[]): void {
    const p = this.inner;
    switch (p.form) {
      case "id":
        params.push([prefix, p.id as string]);
        break;
      case "name":
        params.push([`${prefix}.name`, p.name as string]);
        break;
      case "poi":
        params.push([`${prefix}.id`, p.id as string]);
        params.push([`${prefix}.latitude`, String(p.latitude)]);
        params.push([`${prefix}.longitude`, String(p.longitude)]);
        break;
      case "address":
        params.push([`${prefix}.latitude`, String(p.latitude)]);
        params.push([`${prefix}.longitude`, String(p.longitude)]);
        params.push([`${prefix}.address`, p.address as string]);
        break;
    }
  }

  validate(parameter: string): void {
    const p = this.inner;
    if (p.form === "poi") require(p.id && p.id.trim(), `${parameter}.id`, "POI id must not be empty");
    if (p.form === "address") require(p.address && p.address.trim(), `${parameter}.address`, "address must not be empty");
  }
}
