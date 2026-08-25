// Handwritten builder model for the transport.rest Java binding.
package io.transportrest;

/** A place referenced by journey queries. */
public final class JourneyPlace {
    private final String form;
    private final String id;
    private final String name;
    private final String addressText;
    private final Double latitude;
    private final Double longitude;

    private JourneyPlace(String form, String id, String name, String addressText,
                         Double latitude, Double longitude) {
        this.form = form;
        this.id = id;
        this.name = name;
        this.addressText = addressText;
        this.latitude = latitude;
        this.longitude = longitude;
    }

    public static JourneyPlace stopId(String id) { return new JourneyPlace("id", id, null, null, null, null); }
    public static JourneyPlace name(String name) { return new JourneyPlace("name", null, name, null, null, null); }
    public static JourneyPlace poi(String id, double lat, double lon) {
        return new JourneyPlace("poi", id, null, null, lat, lon);
    }
    public static JourneyPlace address(double lat, double lon, String address) {
        return new JourneyPlace("address", null, null, address, lat, lon);
    }

    void encode(String prefix, List<String[]> params) {
        switch (form) {
            case "id" -> params.add(new String[]{prefix, id});
            case "name" -> params.add(new String[]{prefix + ".name", name});
            case "poi" -> {
                params.add(new String[]{prefix + ".id", id});
                params.add(new String[]{prefix + ".latitude", TransportRestClient.num(latitude)});
                params.add(new String[]{prefix + ".longitude", TransportRestClient.num(longitude)});
            }
            default -> {
                params.add(new String[]{prefix + ".latitude", TransportRestClient.num(latitude)});
                params.add(new String[]{prefix + ".longitude", TransportRestClient.num(longitude)});
                params.add(new String[]{prefix + ".address", addressText});
            }
        }
    }

    void validate(String parameter) {
        if ("poi".equals(form) && (id == null || id.isBlank()))
            throw new InvalidParameterException(parameter + ".id", "POI id must not be empty");
        if ("address".equals(form) && (addressText == null || addressText.isBlank()))
            throw new InvalidParameterException(parameter + ".address", "address must not be empty");
    }
}

