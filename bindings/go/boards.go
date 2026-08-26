// Handwritten departure/arrival board builders for the transport.rest Go binding.
package transportrest

import (
	"context"
	"strings"
	"time"
)

// boardOptions carries the shared board query state; each exported builder
// owns a copy and exposes fluent setters returning its own type.
type boardOptions struct {
	when                   *time.Time
	direction              *string
	duration               *int
	results                *int
	stopovers              *bool
	includeRelatedStations *bool
	linesOfStops           *bool
	remarks                *bool
	language               *string
	moreStops              []string
	products               []queryParam
}

func (o *boardOptions) encode(into []queryParam) []queryParam {
	if o.when != nil {
		into = append(into, queryParam{"when", o.when.UTC().Format(time.RFC3339)})
	}
	if o.direction != nil {
		into = append(into, queryParam{"direction", *o.direction})
	}
	if o.duration != nil {
		into = append(into, queryParam{"duration", intStr(*o.duration)})
	}
	if o.results != nil {
		into = append(into, queryParam{"results", intStr(*o.results)})
	}
	if o.stopovers != nil {
		into = append(into, queryParam{"stopovers", formatBool(*o.stopovers)})
	}
	if o.includeRelatedStations != nil {
		into = append(into, queryParam{"includeRelatedStations", formatBool(*o.includeRelatedStations)})
	}
	if o.linesOfStops != nil {
		into = append(into, queryParam{"linesOfStops", formatBool(*o.linesOfStops)})
	}
	if o.remarks != nil {
		into = append(into, queryParam{"remarks", formatBool(*o.remarks)})
	}
	if o.language != nil {
		into = append(into, queryParam{"language", *o.language})
	}
	if len(o.moreStops) > 0 {
		into = append(into, queryParam{"moreStops", strings.Join(o.moreStops, ",")})
	}
	into = append(into, o.products...)
	return into
}

func ptrOf[T any](v T) *T { return &v }

// DeparturesBuilder queries the departure board of a stop.
type DeparturesBuilder struct {
	client *Client
	ctx    context.Context
	path   string
	opts   boardOptions
}

// Departures starts a departure board query.
func (c *Client) Departures(ctx context.Context, stopID string) *DeparturesBuilder {
	return &DeparturesBuilder{client: c, ctx: ctx,
		path: "/stops/" + encodePathSegment(stopID) + "/departures"}
}

func (b *DeparturesBuilder) When(t time.Time) *DeparturesBuilder { b.opts.when = ptrOf(t); return b }
func (b *DeparturesBuilder) Direction(d string) *DeparturesBuilder {
	b.opts.direction = ptrOf(d)
	return b
}
func (b *DeparturesBuilder) Duration(m int) *DeparturesBuilder { b.opts.duration = ptrOf(m); return b }
func (b *DeparturesBuilder) Results(n int) *DeparturesBuilder  { b.opts.results = ptrOf(n); return b }
func (b *DeparturesBuilder) Stopovers(v bool) *DeparturesBuilder {
	b.opts.stopovers = ptrOf(v)
	return b
}
func (b *DeparturesBuilder) IncludeRelatedStations(v bool) *DeparturesBuilder {
	b.opts.includeRelatedStations = ptrOf(v)
	return b
}
func (b *DeparturesBuilder) LinesOfStops(v bool) *DeparturesBuilder {
	b.opts.linesOfStops = ptrOf(v)
	return b
}
func (b *DeparturesBuilder) Remarks(v bool) *DeparturesBuilder { b.opts.remarks = ptrOf(v); return b }
func (b *DeparturesBuilder) Language(l string) *DeparturesBuilder {
	b.opts.language = ptrOf(l)
	return b
}
func (b *DeparturesBuilder) MoreStops(ids []string) *DeparturesBuilder {
	b.opts.moreStops = ids
	return b
}
func (b *DeparturesBuilder) Products(configure func(*ProductSelection) *ProductSelection) *DeparturesBuilder {
	b.opts.products = configure(&ProductSelection{}).entries
	return b
}

// Get executes the departure board query.
func (b *DeparturesBuilder) Get() (*DeparturesResponse, error) {
	var out DeparturesResponse
	params := b.opts.encode(nil)
	if err := b.client.getJSON(b.ctx, b.path, params, &out, ""); err != nil {
		return nil, err
	}
	return &out, nil
}

// ArrivalsBuilder queries the arrival board of a stop.
type ArrivalsBuilder struct {
	client *Client
	ctx    context.Context
	path   string
	opts   boardOptions
}

// Arrivals starts an arrival board query.
func (c *Client) Arrivals(ctx context.Context, stopID string) *ArrivalsBuilder {
	return &ArrivalsBuilder{client: c, ctx: ctx,
		path: "/stops/" + encodePathSegment(stopID) + "/arrivals"}
}

func (b *ArrivalsBuilder) When(t time.Time) *ArrivalsBuilder   { b.opts.when = ptrOf(t); return b }
func (b *ArrivalsBuilder) Direction(d string) *ArrivalsBuilder { b.opts.direction = ptrOf(d); return b }
func (b *ArrivalsBuilder) Duration(m int) *ArrivalsBuilder     { b.opts.duration = ptrOf(m); return b }
func (b *ArrivalsBuilder) Results(n int) *ArrivalsBuilder      { b.opts.results = ptrOf(n); return b }
func (b *ArrivalsBuilder) Stopovers(v bool) *ArrivalsBuilder   { b.opts.stopovers = ptrOf(v); return b }
func (b *ArrivalsBuilder) IncludeRelatedStations(v bool) *ArrivalsBuilder {
	b.opts.includeRelatedStations = ptrOf(v)
	return b
}
func (b *ArrivalsBuilder) LinesOfStops(v bool) *ArrivalsBuilder {
	b.opts.linesOfStops = ptrOf(v)
	return b
}
func (b *ArrivalsBuilder) Remarks(v bool) *ArrivalsBuilder         { b.opts.remarks = ptrOf(v); return b }
func (b *ArrivalsBuilder) Language(l string) *ArrivalsBuilder      { b.opts.language = ptrOf(l); return b }
func (b *ArrivalsBuilder) MoreStops(ids []string) *ArrivalsBuilder { b.opts.moreStops = ids; return b }
func (b *ArrivalsBuilder) Products(configure func(*ProductSelection) *ProductSelection) *ArrivalsBuilder {
	b.opts.products = configure(&ProductSelection{}).entries
	return b
}

// Get executes the arrival board query.
func (b *ArrivalsBuilder) Get() (*ArrivalsResponse, error) {
	var out ArrivalsResponse
	params := b.opts.encode(nil)
	if err := b.client.getJSON(b.ctx, b.path, params, &out, ""); err != nil {
		return nil, err
	}
	return &out, nil
}
