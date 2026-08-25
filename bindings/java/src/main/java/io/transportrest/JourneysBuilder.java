// Handwritten endpoint builder for the transport.rest Java binding.
package io.transportrest;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.LinkedHashMap;
import java.util.function.Function;

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
