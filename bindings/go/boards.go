// Handwritten board (departures/arrivals) & journey & trip builders.
package transportrest

import (
	"context"
	"strings"
	"time"
)

func validateStopID(id string) error {
	if strings.TrimSpace(id) == "" {
		return &InvalidParameterError{Parameter: "stop_id", Reason: "must not be empty"}
	}
	return nil
}

type boardBuilder struct {
	client *Client
	ctx    context.Context
	path   string
	params []queryParam
	when   *time.Time
}

func (b *boardBuilder) When(t time.Time) *boardBuilder { b.when = &t; return b }
func (b *boardBuilder) Direction(d string) *boardBuilder {
	b.params = append(b.params, queryParam{"direction", d})
	return b
}
func (b *boardBuilder) Duration(m int) *boardBuilder {
	b.params = append(b.params, queryParam{"duration", intStr(m)})
	return b
}
func (b *boardBuilder) Results(n int) *boardBuilder {
	b.params = append(b.params, queryParam{"results", intStr(n)})
	return b
}
func (b *boardBuilder) Stopovers(v bool) *boardBuilder {
	b.params = append(b.params, queryParam{"stopovers", formatBool(v)})
	return b
}
func (b *boardBuilder) IncludeRelatedStations(v bool) *boardBuilder {
	b.params = append(b.params, queryParam{"includeRelatedStations", formatBool(v)})
	return b
}
func (b *boardBuilder) LinesOfStops(v bool) *boardBuilder {
	b.params = append(b.params, queryParam{"linesOfStops", formatBool(v)})
	return b
}
func (b *boardBuilder) Remarks(v bool) *boardBuilder {
	b.params = append(b.params, queryParam{"remarks", formatBool(v)})
	return b
}
func (b *boardBuilder) Language(l string) *boardBuilder {
	b.params = append(b.params, queryParam{"language", l})
	return b
}
func (b *boardBuilder) MoreStops(ids []string) *boardBuilder {
	b.params = append(b.params, queryParam{"moreStops", strings.Join(ids, ",")})
	return b
}
func (b *boardBuilder) Products(configure func(*ProductSelection) *ProductSelection) *boardBuilder {
	sel := configure(&ProductSelection{})
	b.params = append(b.params, sel.entries...)
	return b
}

func (b *boardBuilder) encode() []queryParam {
	all := []queryParam{}
	if b.when != nil {
		all = append(all, queryParam{"when", b.when.UTC().Format("2006-01-02T15:04:05Z07:00")})
	}
	return append(all, b.params...)
}

// Departures queries the departure board of a stop.
func (c *Client) Departures(ctx context.Context, stopID string) *DeparturesBuilder {
	return &DeparturesBuilder{boardBuilder{client: c, ctx: ctx,
		path: "/stops/" + encodePathSegment(stopID) + "/departures"}}
}

// Arrivals queries the arrival board of a stop.
func (c *Client) Arrivals(ctx context.Context, stopID string) *ArrivalsBuilder {
	return &ArrivalsBuilder{boardBuilder{client: c, ctx: ctx,
		path: "/stops/" + encodePathSegment(stopID) + "/arrivals"}}
}

type DeparturesBuilder struct{ boardBuilder }

// Get executes the departure board query.
func (b *DeparturesBuilder) Get() (*DeparturesResponse, error) {
	var out DeparturesResponse
	if err := b.client.getJSON(b.ctx, b.path, b.encode(), &out, ""); err != nil {
		return nil, err
	}
	return &out, nil
}

type ArrivalsBuilder struct{ boardBuilder }

// Get executes the arrival board query.
func (b *ArrivalsBuilder) Get() (*ArrivalsResponse, error) {
	var out ArrivalsResponse
	if err := b.client.getJSON(b.ctx, b.path, b.encode(), &out, ""); err != nil {
		return nil, err
	}
	return &out, nil
}

// Journeys starts a route search.
func (c *Client) Journeys(ctx context.Context, from, to JourneyPlace) *JourneysBuilder {
	return &JourneysBuilder{client: c, ctx: ctx, from: from, to: to}
}

type JourneysBuilder struct {
	client *Client
	ctx    context.Context
	from   JourneyPlace
	to     JourneyPlace
	via    *JourneyPlace
	params []queryParam
}

func (b *JourneysBuilder) Via(p JourneyPlace) *JourneysBuilder { b.via = &p; return b }
func (b *JourneysBuilder) Departure(t time.Time) *JourneysBuilder {
	b.params = append(b.params, queryParam{"departure", t.UTC().Format(time.RFC3339)})
	return b
}
func (b *JourneysBuilder) Arrival(t time.Time) *JourneysBuilder {
	b.params = append(b.params, queryParam{"arrival", t.UTC().Format(time.RFC3339)})
	return b
}
func (b *JourneysBuilder) EarlierThan(ref string) *JourneysBuilder {
	b.params = append(b.params, queryParam{"earlierThan", ref})
	return b
}
func (b *JourneysBuilder) LaterThan(ref string) *JourneysBuilder {
	b.params = append(b.params, queryParam{"laterThan", ref})
	return b
}
func (b *JourneysBuilder) Results(n int) *JourneysBuilder {
	b.params = append(b.params, queryParam{"results", intStr(n)})
	return b
}
func (b *JourneysBuilder) Transfers(n int) *JourneysBuilder {
	b.params = append(b.params, queryParam{"transfers", intStr(n)})
	return b
}
func (b *JourneysBuilder) Products(configure func(*ProductSelection) *ProductSelection) *JourneysBuilder {
	sel := configure(&ProductSelection{})
	b.params = append(b.params, sel.entries...)
	return b
}

// Get executes the journey search.
func (b *JourneysBuilder) Get() (*JourneysResponse, error) {
	params := b.from.encode("from")
	params = append(params, b.to.encode("to")...)
	if b.via != nil {
		params = append(params, b.via.encode("via")...)
	}
	params = append(params, b.params...)
	var out JourneysResponse
	if err := b.client.getJSON(b.ctx, "/journeys", params, &out, ""); err != nil {
		return nil, err
	}
	return &out, nil
}

// Trip fetches one trip by ID.
func (c *Client) Trip(ctx context.Context, id string) *TripBuilder {
	return &TripBuilder{client: c, ctx: ctx, path: "/trips/" + encodePathSegment(id)}
}

type TripBuilder struct {
	client *Client
	ctx    context.Context
	path   string
	params []queryParam
}

func (b *TripBuilder) Stopovers(v bool) *TripBuilder {
	b.params = append(b.params, queryParam{"stopovers", formatBool(v)})
	return b
}
func (b *TripBuilder) Remarks(v bool) *TripBuilder {
	b.params = append(b.params, queryParam{"remarks", formatBool(v)})
	return b
}
func (b *TripBuilder) Polyline(v bool) *TripBuilder {
	b.params = append(b.params, queryParam{"polyline", formatBool(v)})
	return b
}

// Get executes the trip lookup.
func (b *TripBuilder) Get() (*TripResponse, error) {
	var out TripResponse
	if err := b.client.getJSON(b.ctx, b.path, b.params, &out, ""); err != nil {
		return nil, err
	}
	return &out, nil
}
