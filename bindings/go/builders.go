// Handwritten endpoint builders for the transport.rest Go binding.
package transportrest

import (
	"context"
	"strings"
	"time"
)

// JourneyPlace describes from/to/via places of journey queries.
type JourneyPlace struct {
	form, id, name, address string
	lat, lon                float64
}

func StopID(id string) JourneyPlace      { return JourneyPlace{form: "id", id: id} }
func PlaceName(name string) JourneyPlace { return JourneyPlace{form: "name", name: name} }
func POI(id string, lat, lon float64) JourneyPlace {
	return JourneyPlace{form: "poi", id: id, lat: lat, lon: lon}
}
func Address(lat, lon float64, address string) JourneyPlace {
	return JourneyPlace{form: "address", address: address, lat: lat, lon: lon}
}

func (p JourneyPlace) encode(prefix string) []queryParam {
	switch p.form {
	case "id":
		return []queryParam{{prefix, p.id}}
	case "name":
		return []queryParam{{prefix + ".name", p.name}}
	case "poi":
		return []queryParam{
			{prefix + ".id", p.id},
			{prefix + ".latitude", formatFloat(p.lat)},
			{prefix + ".longitude", formatFloat(p.lon)},
		}
	default:
		return []queryParam{
			{prefix + ".latitude", formatFloat(p.lat)},
			{prefix + ".longitude", formatFloat(p.lon)},
			{prefix + ".address", p.address},
		}
	}
}

func formatFloat(f float64) string {
	return strconvFormat(f)
}

// ProductSelection filters transport products (unset keys are omitted).
type ProductSelection struct{ entries []queryParam }

func (p *ProductSelection) set(key string, enabled bool) *ProductSelection {
	value := "false"
	if enabled {
		value = "true"
	}
	for i := range p.entries {
		if p.entries[i].key == key {
			p.entries[i].value = value
			return p
		}
	}
	p.entries = append(p.entries, queryParam{key, value})
	return p
}

func (p *ProductSelection) NationalExpress(v bool) *ProductSelection {
	return p.set("nationalExpress", v)
}
func (p *ProductSelection) National(v bool) *ProductSelection { return p.set("national", v) }
func (p *ProductSelection) RegionalExpress(v bool) *ProductSelection {
	return p.set("regionalExpress", v)
}
func (p *ProductSelection) Regional(v bool) *ProductSelection { return p.set("regional", v) }
func (p *ProductSelection) Suburban(v bool) *ProductSelection { return p.set("suburban", v) }
func (p *ProductSelection) Subway(v bool) *ProductSelection   { return p.set("subway", v) }
func (p *ProductSelection) Tram(v bool) *ProductSelection     { return p.set("tram", v) }
func (p *ProductSelection) Bus(v bool) *ProductSelection      { return p.set("bus", v) }
func (p *ProductSelection) Ferry(v bool) *ProductSelection    { return p.set("ferry", v) }
func (p *ProductSelection) Taxi(v bool) *ProductSelection     { return p.set("taxi", v) }
func (p *ProductSelection) Express(v bool) *ProductSelection  { return p.set("express", v) }

// Locations starts a locations search.
func (c *Client) Locations(ctx context.Context) *LocationsBuilder {
	return &LocationsBuilder{client: c, ctx: ctx}
}

type LocationsBuilder struct {
	client *Client
	ctx    context.Context
	params []queryParam
	query  string
}

func (b *LocationsBuilder) Query(q string) *LocationsBuilder { b.query = q; return b }
func (b *LocationsBuilder) Fuzzy(v bool) *LocationsBuilder {
	b.params = append(b.params, queryParam{"fuzzy", formatBool(v)})
	return b
}
func (b *LocationsBuilder) Results(n int) *LocationsBuilder {
	b.params = append(b.params, queryParam{"results", intStr(n)})
	return b
}
func (b *LocationsBuilder) Stops(v bool) *LocationsBuilder {
	b.params = append(b.params, queryParam{"stops", formatBool(v)})
	return b
}
func (b *LocationsBuilder) Addresses(v bool) *LocationsBuilder {
	b.params = append(b.params, queryParam{"addresses", formatBool(v)})
	return b
}
func (b *LocationsBuilder) POI(v bool) *LocationsBuilder {
	b.params = append(b.params, queryParam{"poi", formatBool(v)})
	return b
}
func (b *LocationsBuilder) LinesOfStops(v bool) *LocationsBuilder {
	b.params = append(b.params, queryParam{"linesOfStops", formatBool(v)})
	return b
}
func (b *LocationsBuilder) Language(l string) *LocationsBuilder {
	b.params = append(b.params, queryParam{"language", l})
	return b
}

// Get executes the search.
func (b *LocationsBuilder) Get() ([]LocationResult, error) {
	if strings.TrimSpace(b.query) == "" {
		return nil, &InvalidParameterError{Parameter: "query", Reason: "a non-empty search term is required"}
	}
	all := append([]queryParam{{"query", b.query}}, b.params...)
	var out []LocationResult
	if err := b.client.getJSON(b.ctx, "/locations", all, &out, ""); err != nil {
		return nil, err
	}
	return out, nil
}

func formatBool(v bool) string {
	if v {
		return "true"
	}
	return "false"
}
