// Handwritten endpoint builder for the transport.rest Java binding.
package io.transportrest;

import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.LinkedHashMap;
import java.util.function.Function;

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
