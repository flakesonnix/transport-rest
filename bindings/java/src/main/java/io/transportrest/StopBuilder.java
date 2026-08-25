// Handwritten endpoint builder for the transport.rest Java binding.
package io.transportrest;

import java.util.ArrayList;
import java.util.List;

/** Single stop/station lookup builder. */
public final class StopBuilder {
    private final TransportRestClient client;
    private final String path;
    private final List<String[]> params = new ArrayList<>();

    StopBuilder(TransportRestClient client, String path) {
        this.client = client;
        this.path = path;
    }

    public StopBuilder linesOfStops(boolean v) {
        params.add(new String[]{"linesOfStops", TransportRestClient.boolStr(v)});
        return this;
    }

    public com.fasterxml.jackson.databind.JsonNode get() {
        return client.getJson(path, params, null);
    }
}
