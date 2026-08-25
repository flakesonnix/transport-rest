// Handwritten endpoint builder for the transport.rest Java binding.
package io.transportrest;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.LinkedHashMap;
import java.util.function.Function;

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
