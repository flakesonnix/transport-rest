package io.transportrest;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.List;
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

/** Departure/arrival board builder. */
public final class DeparturesBuilder {
    private final TransportRestClient client;
    private final String path;
    private final String kind;
    private final List<String[]> params = new ArrayList<>();

    DeparturesBuilder(TransportRestClient client, String path, String kind) {
        this.client = client;
        this.path = path;
        this.kind = kind;
    }

    public DeparturesBuilder when(OffsetDateTime when) {
        params.add(new String[]{"when", TransportRestClient.iso(when)});
        return this;
    }

    public DeparturesBuilder duration(int minutes) {
        params.add(new String[]{"duration", String.valueOf(minutes)});
        return this;
    }

    public DeparturesBuilder results(int n) {
        params.add(new String[]{"results", String.valueOf(n)});
        return this;
    }

    public DeparturesBuilder products(Function<ProductSelection, ProductSelection> configure) {
        configure.apply(new ProductSelection()).encode(params);
        return this;
    }

    public com.fasterxml.jackson.databind.JsonNode get() {
        return client.getJson(path, params, null);
    }
}

/** Journey search builder. */
public final class JourneysBuilder {
    private final TransportRestClient client;
    private final JourneyPlace from;
    private final JourneyPlace to;
    private JourneyPlace via;
    private final List<String[]> options = new ArrayList<>();

    JourneysBuilder(TransportRestClient client, JourneyPlace from, JourneyPlace to) {
        this.client = client;
        this.from = from;
        this.to = to;
    }

    public JourneysBuilder via(JourneyPlace place) { this.via = place; return this; }
    public JourneysBuilder departure(OffsetDateTime when) {
        options.add(new String[]{"departure", TransportRestClient.iso(when)});
        return this;
    }
    public JourneysBuilder arrival(OffsetDateTime when) {
        options.add(new String[]{"arrival", TransportRestClient.iso(when)});
        return this;
    }
    public JourneysBuilder earlierThan(String ref) {
        options.add(new String[]{"earlierThan", ref});
        return this;
    }
    public JourneysBuilder laterThan(String ref) {
        options.add(new String[]{"laterThan", ref});
        return this;
    }
    public JourneysBuilder transfers(int n) {
        options.add(new String[]{"transfers", String.valueOf(n)});
        return this;
    }
    public JourneysBuilder products(Function<ProductSelection, ProductSelection> configure) {
        configure.apply(new ProductSelection()).encode(options);
        return this;
    }

    public com.fasterxml.jackson.databind.JsonNode get() {
        from.validate("from");
        to.validate("to");
        if (via != null) via.validate("via");
        List<String[]> params = new ArrayList<>();
        from.encode("from", params);
        to.encode("to", params);
        if (via != null) via.encode("via", params);
        params.addAll(options);
        return client.getJson("/journeys", params, null);
    }
}

/** Trip lookup builder. */
public final class TripBuilder {
    private final TransportRestClient client;
    private final String path;
    private final List<String[]> params = new ArrayList<>();

    TripBuilder(TransportRestClient client, String path) {
        this.client = client;
        this.path = path;
    }

    public TripBuilder stopovers(boolean v) {
        params.add(new String[]{"stopovers", TransportRestClient.boolStr(v)});
        return this;
    }

    public com.fasterxml.jackson.databind.JsonNode get() {
        return client.getJson(path, params, null);
    }
}

/** Radar builder (capability-gated). */
public final class RadarBuilder {
    private final TransportRestClient client;
    private final Map<String, Double> box = new LinkedHashMap<>();
    private final List<String[]> params = new ArrayList<>();

    RadarBuilder(TransportRestClient client) { this.client = client; }

    public RadarBuilder north(double v) { box.put("north", v); return this; }
    public RadarBuilder west(double v) { box.put("west", v); return this; }
    public RadarBuilder south(double v) { box.put("south", v); return this; }
    public RadarBuilder east(double v) { box.put("east", v); return this; }
    public RadarBuilder results(int n) { params.add(new String[]{"results", String.valueOf(n)}); return this; }

    public com.fasterxml.jackson.databind.JsonNode get() {
        if (box.size() != 4)
            throw new Errors.InvalidParameterException(null,
                "north, west, south and east are all required");
        List<String[]> all = new ArrayList<>();
        box.forEach((key, value) -> all.add(new String[]{key, TransportRestClient.num(value)}));
        all.addAll(params);
        return client.getJson("/radar", all, "radar");
    }
}
