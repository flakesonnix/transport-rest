// Binding tests: create client, request, deserialize, handle error.
// Fully offline via net/http/httptest.

package transportrest

import (
	"context"
	"net/http"
	"net/http/httptest"
	"strings"
	"testing"
	"time"
)

func newTestClient(t *testing.T, serverURL string) *Client {
	t.Helper()
	c, err := NewClient(ClientOptions{Provider: "db", BaseURL: serverURL})
	if err != nil {
		t.Fatalf("NewClient: %v", err)
	}
	return c
}

func TestLocationsHappyPath(t *testing.T) {
	var gotPath, gotQuery string
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		gotPath = r.URL.Path
		gotQuery = r.URL.RawQuery
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`[
			{"type":"stop","id":"8011160","name":"Berlin Hbf",
			 "location":{"type":"location","latitude":52.525,"longitude":13.369}},
			{"type":"location","name":"Alexanderplatz","poi":true}
		]`))
	}))
	defer srv.Close()

	client := newTestClient(t, srv.URL)
	result, err := client.Locations(context.Background()).Query("Berlin").Results(5).Get()
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	if gotPath != "/locations" || !strings.Contains(gotQuery, "query=Berlin") {
		t.Fatalf("unexpected request: %s?%s", gotPath, gotQuery)
	}
	if len(result) != 2 {
		t.Fatalf("expected 2 results, got %d", len(result))
	}
	if result[0]["id"] != "8011160" {
		t.Fatalf("stop id mismatch: %+v", result[0])
	}
}

func TestLocationsRequiresQuery(t *testing.T) {
	client := newTestClient(t, "http://127.0.0.1:1")
	_, err := client.Locations(context.Background()).Get()
	if err == nil {
		t.Fatal("expected InvalidParameterError")
	}
	if _, ok := err.(*InvalidParameterError); !ok {
		t.Fatalf("wrong error type: %T", err)
	}
}

func TestDeparturesParsesBoard(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		if !strings.HasSuffix(r.URL.Path, "/departures") {
			t.Errorf("path: %s", r.URL.Path)
		}
		if q := r.URL.Query().Get("bus"); q != "false" {
			t.Errorf("product filter missing: %q", q)
		}
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"departures":[{"tripId":"t1",
			"line":{"id":"ICE 599","mode":"train"},"delay":120.0,
			"when":"2026-08-01T12:00:00+02:00"}],
			"realtimeDataUpdatedAt":1754000000}`))
	}))
	defer srv.Close()

	client := newTestClient(t, srv.URL)
	result, err := client.Departures(context.Background(), "8011160").
		Products(func(p *ProductSelection) *ProductSelection { return p.Bus(false).Tram(false) }).
		Get()
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	if result.RealtimeDataUpdatedAt == nil || *result.RealtimeDataUpdatedAt != 1754000000 {
		t.Fatalf("realtime timestamp mismatch")
	}
	if len(result.Departures) != 1 {
		t.Fatalf("departures: %d", len(result.Departures))
	}
}

func TestRateLimitedWithRetryAfter(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Retry-After", "30")
		w.WriteHeader(http.StatusTooManyRequests)
		_, _ = w.Write([]byte(`{"message":"Too Many Requests"}`))
	}))
	defer srv.Close()

	client := newTestClient(t, srv.URL)
	_, err := client.Locations(context.Background()).Query("x").Get()
	rl, ok := err.(*RateLimitedError)
	if !ok {
		t.Fatalf("expected RateLimitedError, got %T (%v)", err, err)
	}
	if !rl.HasRetryAfter() || rl.RetryAfter != 30*time.Second {
		t.Fatalf("retry-after: %+v", rl)
	}
}

func TestStructuredApiError404(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusNotFound)
		_, _ = w.Write([]byte(`{"message":"Stop not found."}`))
	}))
	defer srv.Close()

	client := newTestClient(t, srv.URL)
	_, err := client.Locations(context.Background()).Query("x").Get()
	api, ok := err.(*ApiError)
	if !ok {
		t.Fatalf("expected ApiError, got %T", err)
	}
	if api.Status != 404 || api.Message != "Stop not found." {
		t.Fatalf("api error: %+v", api)
	}
}

func TestNonJson502BecomesHttpError(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.WriteHeader(http.StatusBadGateway)
		_, _ = w.Write([]byte("<html>Bad Gateway</html>"))
	}))
	defer srv.Close()

	client := newTestClient(t, srv.URL)
	_, err := client.Locations(context.Background()).Query("x").Get()
	herr, ok := err.(*HttpError)
	if !ok {
		t.Fatalf("expected HttpError, got %T", err)
	}
	if herr.Status != 502 || !strings.Contains(herr.BodySnippet, "<html>") {
		t.Fatalf("http error: %+v", herr)
	}
}

func TestCapabilityGating(t *testing.T) {
	client := newTestClient(t, "http://127.0.0.1:1")
	if client.Supports("radar") {
		t.Fatal("db must not support radar by default")
	}
	custom, err := NewClient(ClientOptions{
		BaseURL: "http://127.0.0.1:1", Provider: "custom", EnableCapabilities: []string{"radar"},
	})
	if err != nil {
		t.Fatalf("custom client: %v", err)
	}
	if !custom.Supports("radar") {
		t.Fatal("enabled capability missing")
	}
}

func TestUnknownFieldsAreIgnored(t *testing.T) {
	srv := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, _ *http.Request) {
		w.Header().Set("Content-Type", "application/json")
		_, _ = w.Write([]byte(`{"type":"stop","id":"futuristic","brandNewField":[1,2]}`))
	}))
	defer srv.Close()

	client := newTestClient(t, srv.URL)
	stop, err := client.Locations(context.Background()).Query("f").Get()
	if err != nil {
		t.Fatalf("Get: %v", err)
	}
	if len(stop) != 1 || stop[0]["type"] != "stop" {
		t.Fatalf("unexpected: %+v", stop)
	}
}
