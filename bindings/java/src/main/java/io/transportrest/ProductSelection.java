// Handwritten builder model for the transport.rest Java binding.
package io.transportrest;

/** Product filter; unset keys are omitted. */
public final class ProductSelection {
    private final Map<String, Boolean> entries = new LinkedHashMap<>();

    public ProductSelection set(String key, boolean enabled) { entries.put(key, enabled); return this; }
    public ProductSelection bus(boolean v) { return set("bus", v); }
    public ProductSelection tram(boolean v) { return set("tram", v); }
    public ProductSelection suburban(boolean v) { return set("suburban", v); }
    public ProductSelection subway(boolean v) { return set("subway", v); }
    public ProductSelection ferry(boolean v) { return set("ferry", v); }
    public ProductSelection express(boolean v) { return set("express", v); }
    public ProductSelection regional(boolean v) { return set("regional", v); }
    public ProductSelection national(boolean v) { return set("national", v); }
    public ProductSelection nationalExpress(boolean v) { return set("nationalExpress", v); }
    public ProductSelection regionalExpress(boolean v) { return set("regionalExpress", v); }
    public ProductSelection taxi(boolean v) { return set("taxi", v); }

    void encode(List<String[]> params) {
        entries.forEach((key, enabled) -> params.add(new String[]{key, TransportRestClient.boolStr(enabled)}));
    }
}
