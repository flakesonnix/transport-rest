// Handwritten error taxonomy mirroring the Rust core.
package io.transportrest;

import com.fasterxml.jackson.databind.JsonNode;
import java.time.Duration;

/** Structured error taxonomy mirroring the Rust core. */
public final class Errors {
    private Errors() {}

    /** Base class of all library errors. */
    public abstract static class TransportRestException extends RuntimeException {
        protected TransportRestException(String message) { super(message); }
    }

    /** Connection-level failure (DNS, TCP, TLS). */
    public static final class NetworkException extends TransportRestException {
        public final String url;
        public final Throwable cause;

        public NetworkException(String url, Throwable cause) {
            super("network error for " + url + ": " + cause.getMessage());
            this.url = url;
            this.cause = cause;
        }
    }

    /** The request exceeded its timeout. */
    public static final class RequestTimeoutException extends TransportRestException {
        public final String kind;
        public final String url; // nullable

        public RequestTimeoutException(String kind, String url) {
            super("request timed out (" + kind + ")" + (url == null ? "" : " for " + url));
            this.kind = kind;
            this.url = url;
        }
    }

    /** A non-success response that was not a structured API error. */
    public static final class HttpException extends TransportRestException {
        public final int status;
        public final String method;
        public final String url;
        public final String bodySnippet;

        HttpException(int status, String method, String url, String bodySnippet) {
            super("unexpected HTTP response: HTTP " + status + " from " + method + " " + url
                + ": " + (bodySnippet.isEmpty() ? "<no body>" : bodySnippet));
            this.status = status;
            this.method = method;
            this.url = url;
            this.bodySnippet = bodySnippet;
        }
    }

    /** Structured error body {"message": "..."} from the instance. */
    public static class ApiException extends TransportRestException {
        public final int status;
        public final String url;
        public final JsonNode body;

        public ApiException(int status, String url, String message, JsonNode body) {
            super(message == null || message.isEmpty()
                ? "API error (HTTP " + status + ")" : message);
            this.status = status;
            this.url = url;
            this.body = body;
        }
    }

    /** HTTP 429 including an optional Retry-After hint. */
    public static final class RateLimitedException extends ApiException {
        public final Duration retryAfter; // nullable

        RateLimitedException(String url, String message, JsonNode body, Duration retryAfter) {
            super(429, url,
                retryAfter != null
                    ? "rate limited (HTTP 429), retry after " + retryAfter.getSeconds() + "s: " + message
                    : "rate limited (HTTP 429): " + message,
                body);
            this.retryAfter = retryAfter;
        }
    }

    /** Invalid JSON or a schema violation. */
    public static final class SerializationException extends TransportRestException {
        public final String reason;
        public final String url; // nullable

        SerializationException(String reason, String url) {
            super("failed to deserialize response"
                + (url == null ? "" : " for " + url) + ": " + reason);
            this.reason = reason;
            this.url = url;
        }
    }

    /** Client-side validation failure. */
    public static final class InvalidParameterException extends TransportRestException {
        public final String parameter; // nullable
        public final String reason;

        InvalidParameterException(String parameter, String reason) {
            super("invalid parameter '" + (parameter == null ? "<none>" : parameter) + "': " + reason);
            this.parameter = parameter;
            this.reason = reason;
        }
    }

    /** Endpoint group unavailable on the configured provider. */
    public static final class CapabilityNotSupportedException extends TransportRestException {
        public final String capability;
        public final String provider;

        CapabilityNotSupportedException(String capability, String provider) {
            super("capability '" + capability + "' is not supported by provider '" + provider + "'");
            this.capability = capability;
            this.provider = provider;
        }
    }
}
