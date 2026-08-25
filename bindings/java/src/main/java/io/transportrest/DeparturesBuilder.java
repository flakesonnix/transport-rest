// Handwritten endpoint builder for the transport.rest Java binding.
package io.transportrest;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.LinkedHashMap;
import java.util.function.Function;

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
