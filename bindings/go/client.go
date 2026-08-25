// Handwritten client core for the transport.rest Go binding.
package transportrest

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"net/url"
	"strconv"
	"strings"
	"sync/atomic"
	"time"
)

// Providers and their default base URLs.
var Providers = map[string]string{
	"db":     "https://v6.db.transport.rest",
	"bvg":    "https://v6.bvg.transport.rest",
	"vbb":    "https://v6.vbb.transport.rest",
	"poland": "https://poland.transport.rest",
}

var providerCapabilities = map[string][]string{
	"db":     {"stations"},
	"bvg":    {"stops_search", "radar", "reachable_from", "trips_by_name"},
	"vbb":    {"stops_search", "radar", "reachable_from", "trips_by_name"},
	"poland": {"radar", "reachable_from", "trips_by_name"},
}

// ClientOptions configures a Client.
type ClientOptions struct {
	// Provider selects a known instance; default "db".
	Provider string
	// BaseURL overrides the provider default.
	BaseURL string
	// HTTPClient allows custom transports/proxies.
	HTTPClient *http.Client
	// Timeout applies per request when HTTPClient is nil; default 30s.
	Timeout time.Duration
	// UserAgent sent with every request.
	UserAgent string
	// MaxResponseBytes guards against oversized responses; default 16 MiB.
	MaxResponseBytes int64
	// EnableCapabilities force-enables endpoint groups.
	EnableCapabilities []string
}

// Client talks to one transport.rest instance. Safe for concurrent use.
type Client struct {
	provider   string
	baseURL    string
	http       *http.Client
	userAgent  string
	maxBytes   int64
	caps       map[string]bool
	nextStream atomic.Uint64
}

// NewClient creates a client; ctx is kept for future use in dialer options.
func NewClient(opts ClientOptions) (*Client, error) {
	if opts.Provider == "" {
		opts.Provider = "db"
	}
	base := opts.BaseURL
	if base == "" {
		var ok bool
		base, ok = Providers[opts.Provider]
		if !ok {
			return nil, &InvalidParameterError{Parameter: "provider", Reason: fmt.Sprintf("unknown provider %q requires BaseURL", opts.Provider)}
		}
	}
	if !strings.HasPrefix(base, "http://") && !strings.HasPrefix(base, "https://") {
		return nil, &InvalidParameterError{Parameter: "base_url", Reason: "must be an absolute http(s) URL"}
	}
	httpClient := opts.HTTPClient
	if httpClient == nil {
		httpClient = &http.Client{Timeout: 30 * time.Second}
		if opts.Timeout > 0 {
			httpClient.Timeout = opts.Timeout
		}
	}
	maxBytes := opts.MaxResponseBytes
	if maxBytes == 0 {
		maxBytes = 16 << 20
	}
	ua := opts.UserAgent
	if ua == "" {
		ua = "transport-rest-go/0.1.0"
	}
	caps := map[string]bool{}
	for _, c := range providerCapabilities[opts.Provider] {
		caps[c] = true
	}
	for _, c := range opts.EnableCapabilities {
		caps[c] = true
	}
	return &Client{
		provider:  opts.Provider,
		baseURL:   strings.TrimRight(base, "/"),
		http:      httpClient,
		userAgent: ua,
		maxBytes:  maxBytes,
		caps:      caps,
	}, nil
}

// Provider returns the configured provider id.
func (c *Client) Provider() string { return c.provider }

// Supports reports whether the endpoint group may be used.
func (c *Client) Supports(capability string) bool { return c.caps[capability] }

func (c *Client) checkCapability(capability string) error {
	if !c.caps[capability] {
		return &CapabilityNotSupportedError{Capability: capability, Provider: c.provider}
	}
	return nil
}

type queryParam struct{ key, value string }

func encodePathSegment(s string) string {
	return url.PathEscape(s)
}

func (c *Client) buildURL(path string, params []queryParam) string {
	var b strings.Builder
	b.WriteString(c.baseURL)
	b.WriteString(path)
	for i, p := range params {
		if i == 0 {
			b.WriteString("?")
		} else {
			b.WriteString("&")
		}
		b.WriteString(url.QueryEscape(p.key))
		b.WriteString("=")
		b.WriteString(url.QueryEscape(p.value))
	}
	return b.String()
}

func (c *Client) getJSON(ctx context.Context, path string, params []queryParam, out any, capability string) error {
	if capability != "" {
		if err := c.checkCapability(capability); err != nil {
			return err
		}
	}
	target := c.buildURL(path, params)
	req, err := http.NewRequestWithContext(ctx, http.MethodGet, target, nil)
	if err != nil {
		return &InvalidParameterError{Parameter: "request", Reason: err.Error()}
	}
	req.Header.Set("Accept", "application/json")
	req.Header.Set("User-Agent", c.userAgent)

	resp, err := c.http.Do(req)
	if err != nil {
		if ctxErr := ctx.Err(); ctxErr != nil {
			return &RequestTimeoutError{Kind_: TimeoutRequest, URL: target}
		}
		return &NetworkError{URL: target, Cause: err}
	}
	defer func() { _ = resp.Body.Close() }()

	body, readErr := io.ReadAll(io.LimitReader(resp.Body, c.maxBytes+1))
	if readErr != nil {
		return &NetworkError{URL: target, Cause: readErr}
	}
	if int64(len(body)) > c.maxBytes {
		return &SerializationError{URL: target, Reason: fmt.Sprintf("response exceeds configured maximum of %d bytes", c.maxBytes)}
	}

	if resp.StatusCode < 200 || resp.StatusCode > 299 {
		return classifyHTTPError(resp.StatusCode, resp.Header, body, target)
	}
	if err := json.Unmarshal(body, out); err != nil {
		return &SerializationError{URL: target, Reason: err.Error()}
	}
	return nil
}

func classifyHTTPError(status int, header http.Header, body []byte, target string) TransportRestError {
	text := strings.TrimSpace(string(body))
	snippet := text
	if len(snippet) > 512 {
		snippet = snippet[:512]
	}
	var parsed struct {
		Message string `json:"message"`
	}
	isObject := strings.HasPrefix(text, "{")
	if isObject {
		_ = json.Unmarshal([]byte(text), &parsed)
	}
	if status == http.StatusTooManyRequests {
		api := &ApiError{Status: status, URL: target, Message: firstNonEmpty(parsed.Message, "rate limited"), Body: body}
		var retryAfter *time.Duration
		if ra := header.Get("Retry-After"); ra != "" {
			if secs, err := strconv.Atoi(strings.TrimSpace(ra)); err == nil && secs >= 0 {
				d := time.Duration(secs) * time.Second
				retryAfter = &d
			}
		}
		return NewRateLimitedError(api, retryAfter)
	}
	if parsed.Message != "" || isObject {
		msg := firstNonEmpty(parsed.Message, "unspecified API error")
		return &ApiError{Status: status, URL: target, Message: msg, Body: body}
	}
	return &HttpError{Status: status, Method: http.MethodGet, URL: target, BodySnippet: firstNonEmpty(snippet, "<no body>")}
}

func firstNonEmpty(values ...string) string {
	for _, v := range values {
		if v != "" {
			return v
		}
	}
	return ""
}
