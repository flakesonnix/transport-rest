/** Structured error taxonomy mirroring the Rust core. */

export abstract class TransportRestError extends Error {}

export type TimeoutKind = "connect" | "request";

export class NetworkError extends TransportRestError {
  constructor(readonly url: string | undefined, readonly cause: unknown) {
    super(`network error${url ? ` for ${url}` : ""}: ${String(cause)}`);
    this.name = "NetworkError";
  }
}

export class RequestTimeoutError extends TransportRestError {
  constructor(
    readonly kind: TimeoutKind,
    readonly url: string | undefined,
  ) {
    super(`request timed out (${kind})${url ? ` for ${url}` : ""}`);
    this.name = "RequestTimeoutError";
  }
}

export class HttpError extends TransportRestError {
  constructor(
    readonly status: number,
    readonly method: string,
    readonly url: string,
    readonly bodySnippet: string,
  ) {
    super(
      `unexpected HTTP response: HTTP ${status} from ${method} ${url}: ${
        bodySnippet || "<no body>"
      }`,
    );
    this.name = "HttpError";
  }
}

export class ApiError extends TransportRestError {
  constructor(
    readonly status: number,
    readonly url: string,
    readonly message: string,
    readonly body: unknown,
  ) {
    super(message || `API error (HTTP ${status})`);
    this.name = "ApiError";
  }
}

export class RateLimitedError extends ApiError {
  constructor(
    url: string,
    message: string,
    body: unknown,
    /** Suggested wait time from the Retry-After header, in seconds. */
    readonly retryAfterSeconds?: number,
  ) {
    const rendered =
      retryAfterSeconds !== undefined
        ? `rate limited (HTTP 429), retry after ${retryAfterSeconds}s: ${message}`
        : `rate limited (HTTP 429): ${message}`;
    super(429, url, rendered, body);
    this.name = "RateLimitedError";
  }
}

export class SerializationError extends TransportRestError {
  constructor(readonly reason: string, readonly url?: string) {
    super(
      `failed to deserialize response${url ? ` for ${url}` : ""}: ${reason}`,
    );
    this.name = "SerializationError";
  }
}

export class InvalidParameterError extends TransportRestError {
  constructor(
    readonly parameter: string | undefined,
    reason: string,
  ) {
    super(`invalid parameter '${parameter ?? "<none>"}': ${reason}`);
    this.name = "InvalidParameterError";
  }
}

export class CapabilityNotSupportedError extends TransportRestError {
  constructor(capability: string, provider: string) {
    super(
      `capability '${capability}' is not supported by provider '${provider}'; ` +
        "enable it explicitly via new TransportRestClient({ enableCapabilities }) " +
        "if you know the endpoint exists",
    );
    this.name = "CapabilityNotSupportedError";
  }
}
