// Code generated headers aside: this file is hand-written.
// Structured error taxonomy mirroring the Rust core.

package transportrest

import (
	"context"
	"fmt"
	"net/http"
	"strconv"
	"time"
)

// TransportRestError is the base interface implemented by all library errors.
type TransportRestError interface {
	error
	// Kind returns a stable identifier of the error class
	// ("network", "timeout", "http", "api", "rate_limited",
	// "serialization", "invalid_parameter", "capability").
	Kind() string
}

// TimeoutKind classifies which request phase timed out.
type TimeoutKind string

const (
	TimeoutConnect TimeoutKind = "connect"
	TimeoutRequest TimeoutKind = "request"
)

// NetworkError is a connection-level failure (DNS, TCP, TLS).
type NetworkError struct {
	URL   string
	Cause error
}

func (e *NetworkError) Error() string { return fmt.Sprintf("network error for %s: %v", e.URL, e.Cause) }
func (e *NetworkError) Kind() string  { return "network" }
func (e *NetworkError) Unwrap() error { return e.Cause }

// RequestTimeoutError reports an exceeded timeout.
type RequestTimeoutError struct {
	Kind_ TimeoutKind
	URL   string
}

func (e *RequestTimeoutError) Error() string {
	return fmt.Sprintf("request timed out (%s) for %s", e.Kind_, e.URL)
}
func (e *RequestTimeoutError) Kind() string { return "timeout" }

// HttpError is a non-success response that was not a structured API error.
type HttpError struct {
	Status      int
	Method      string
	URL         string
	BodySnippet string
}

func (e *HttpError) Error() string {
	return fmt.Sprintf("unexpected HTTP response: HTTP %d from %s %s: %s", e.Status, e.Method, e.URL, e.BodySnippet)
}
func (e *HttpError) Kind() string { return "http" }

// ApiError is a structured error body {"message": "..."} from the instance.
type ApiError struct {
	Status  int
	URL     string
	Message string
	Body    []byte
}

func (e *ApiError) Error() string { return fmt.Sprintf("API error (HTTP %d): %s", e.Status, e.Message) }
func (e *ApiError) Kind() string  { return "api" }

// RateLimitedError is HTTP 429 including an optional Retry-After hint.
type RateLimitedError struct {
	Api           *ApiError
	RetryAfter    time.Duration
	hasRetryAfter bool
}

func NewRateLimitedError(api *ApiError, retryAfter *time.Duration) *RateLimitedError {
	e := &RateLimitedError{Api: api}
	if retryAfter != nil {
		e.RetryAfter = *retryAfter
		e.hasRetryAfter = true
	}
	return e
}

func (e *RateLimitedError) HasRetryAfter() bool { return e.hasRetryAfter }
func (e *RateLimitedError) Error() string {
	if e.hasRetryAfter {
		return fmt.Sprintf("rate limited (HTTP 429), retry after %s: %s", e.RetryAfter, e.Api.Message)
	}
	return fmt.Sprintf("rate limited (HTTP 429): %s", e.Api.Message)
}
func (e *RateLimitedError) Kind() string { return "rate_limited" }

// SerializationError covers invalid JSON and schema violations.
type SerializationError struct {
	URL    string
	Reason string
}

func (e *SerializationError) Error() string {
	return fmt.Sprintf("failed to deserialize response for %s: %s", e.URL, e.Reason)
}
func (e *SerializationError) Kind() string { return "serialization" }

// InvalidParameterError reports client-side validation failures.
type InvalidParameterError struct {
	Parameter string
	Reason    string
}

func (e *InvalidParameterError) Error() string {
	return fmt.Sprintf("invalid parameter '%s': %s", e.Parameter, e.Reason)
}
func (e *InvalidParameterError) Kind() string { return "invalid_parameter" }

// CapabilityNotSupportedError reports endpoints unavailable on this provider.
type CapabilityNotSupportedError struct {
	Capability string
	Provider   string
}

func (e *CapabilityNotSupportedError) Error() string {
	return fmt.Sprintf("capability '%s' is not supported by provider '%s'; enable it explicitly via EnableCapability", e.Capability, e.Provider)
}
func (e *CapabilityNotSupportedError) Kind() string { return "capability" }

// Context helpers so callers can pass ctx through builders naturally.
var _ = context.Background
var _ = http.MethodGet
var _ = strconv.Itoa
