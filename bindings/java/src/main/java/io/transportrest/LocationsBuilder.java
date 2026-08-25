// Handwritten endpoint builder for the transport.rest Java binding.
package io.transportrest;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.LinkedHashMap;
import java.util.function.Function;

/** Locations search builder. */
public final class LocationsBuilder {
    private final TransportRestClient client;
    private final List<String[]> params = new ArrayList<>();
    private String query;

    LocationsBuilder(TransportRestClient client) { this.client = client; }

    public LocationsBuilder query(String q) {
        if (q == null || q.isBlank()) throw new Errors.InvalidParameterException("query",
            "a non-empty search term is required");
        this.query = q;
        return this;
    }

    public LocationsBuilder results(int n) { params.add(new String[]{"results", String.valueOf(n)}); return this; }
    public LocationsBuilder fuzzy(boolean v) { params.add(new String[]{"fuzzy", TransportRestClient.boolStr(v)}); return this; }

    /** Executes the search and returns raw JSON for caller-side model mapping. */
    public com.fasterxml.jackson.databind.JsonNode get() {
        if (query == null)
            throw new Errors.InvalidParameterException("query", "query() is required before get()");
        List<String[]> all = new ArrayList<>();
        all.add(new String[]{"query", query});
        all.addAll(params);
        return client.getJson("/locations", all, null);
    }

    /** Typed variant using the generated models. */
    public <T> T get(Function<com.fasterxml.jackson.databind.JsonNode, T> mapper) {
        return mapper.apply(get());
    }
}
