// Handwritten client core for the transport.rest Java binding (java.net.http).
package io.transportrest;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.io.IOException;
import java.net.URI;
import java.net.URLEncoder;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.Set;

import static io.transportrest.Errors.ApiException;
import static io.transportrest.Errors.CapabilityNotSupportedException;
import static io.transportrest.Errors.HttpException;
import static io.transportrest.Errors.InvalidParameterException;
import static io.transportrest.Errors.NetworkException;
import static io.transportrest.Errors.RateLimitedException;
import static io.transportrest.Errors.RequestTimeoutException;
import static io.transportrest.Errors.SerializationException;

/** Client for one transport.rest instance. Thread-safe. */
public final class TransportRestClient {
    public static final Map<String, String> PROVIDERS = Map.of(
        "db", "https://v6.db.transport.rest",
        "bvg", "https://v6.bvg.transport.rest",
        "vbb", "https://v6.vbb.transport.rest",
        "poland", "https://poland.transport.rest"
    );

    private static final Map<String, Set<String>> CAPABILITIES = Map.of(
        "db", Set.of("stations"),
        "bvg", Set.of("stops_search", "radar", "reachable_from", "trips_by_name"),
        "vbb", Set.of("stops_search", "radar", "reachable_from", "trips_by_name"),
        "poland", Set.of("radar", "reachable_from", "trips_by_name")
    );

    private final String provider;
    private final String baseUrl;
    private final HttpClient http;
    private final String userAgent;
    private final long maxResponseBytes;
    private final Duration timeout;
    private final ObjectMapper mapper = new ObjectMapper();
    private final Set<String> capabilities;

    public TransportRestClient(Builder builder) {
        this.provider = builder.provider == null ? "db" : builder.provider;
        String base = builder.baseUrl;
        if (base == null) base = PROVIDERS.get(this.provider);
        if (base == null)
            throw new InvalidParameterException("base_url",
                "unknown provider '" + this.provider + "' requires a baseUrl");
        if (!base.startsWith("http://") && !base.startsWith("https://"))
            throw new InvalidParameterException("base_url", "must be an absolute http(s) URL");
        this.baseUrl = base.replaceAll("/+$", "");
        this.timeout = builder.timeout == null ? Duration.ofSeconds(30) : builder.timeout;
        this.userAgent = builder.userAgent == null ? "transport-rest-java/0.1.0" : builder.userAgent;
        this.maxResponseBytes = builder.maxResponseBytes <= 0 ? 16L << 20 : builder.maxResponseBytes;
        this.capabilities = CAPABILITIES.getOrDefault(this.provider, Set.of());
        for (String cap : builder.enableCapabilities) capabilities.add(cap);
        this.http = HttpClient.newBuilder().connectTimeout(timeout).build();
    }

    /** Fluent constructor. */
    public static Builder newClient() { return new Builder(); }

    public static final class Builder {
        private String provider;
        private String baseUrl;
        private Duration timeout;
        private String userAgent;
        private long maxResponseBytes;
        private List<String> enableCapabilities = List.of();

        public Builder provider(String p) { this.provider = p; return this; }
        public Builder baseUrl(String u) { this.baseUrl = u; return this; }
        public Builder timeout(Duration d) { this.timeout = d; return this; }
        public Builder userAgent(String ua) { this.userAgent = ua; return this; }
        public Builder maxResponseBytes(long n) { this.maxResponseBytes = n; return this; }
        public Builder enableCapabilities(List<String> caps) { this.enableCapabilities = caps; return this; }
        public TransportRestClient build() { return new TransportRestClient(this); }
    }

    public String provider() { return provider; }

    public boolean supports(String capability) { return capabilities.contains(capability); }

    // -- resource accessors --------------------------------------------------

    public LocationsBuilder locations() { return new LocationsBuilder(this); }

    public DeparturesBuilder departures(String stopId) {
        requireNonEmpty(stopId, "stop_id");
        return new DeparturesBuilder(this,
            "/stops/" + encodePath(stopId) + "/departures", "departures");
    }

    public DeparturesBuilder arrivals(String stopId) {
        requireNonEmpty(stopId, "stop_id");
        return new DeparturesBuilder(this,
            "/stops/" + encodePath(stopId) + "/arrivals", "arrivals");
    }

    public JourneysBuilder journeys(JourneyPlace from, JourneyPlace to) {
        return new JourneysBuilder(this, from, to);
    }

    public TripBuilder trip(String tripId) {
        requireNonEmpty(tripId, "trip_id");
        return new TripBuilder(this, "/trips/" + encodePath(tripId));
    }

    public RadarBuilder radar() { return new RadarBuilder(this); }

    // -- execution -----------------------------------------------------------

    void checkCapability(String capability) {
        if (!capabilities.contains(capability))
            throw new CapabilityNotSupportedException(capability, provider);
    }

    JsonNode getJson(String path, List<String[]> params, String capability) {
        if (capability != null) checkCapability(capability);
        StringBuilder url = new StringBuilder(baseUrl).append(path);
        if (!params.isEmpty()) {
            url.append('?');
            for (int i = 0; i < params.size(); i++) {
                if (i > 0) url.append('&');
                url.append(URLEncoder.encode(params.get(i)[0], StandardCharsets.UTF_8))
                    .append('=')
                    .append(URLEncoder.encode(params.get(i)[1], StandardCharsets.UTF_8));
            }
        }
        URI uri = URI.create(url.toString());
        HttpRequest request = HttpRequest.newBuilder(uri)
            .timeout(timeout)
            .header("Accept", "application/json")
            .header("User-Agent", userAgent)
            .GET()
            .build();

        HttpResponse<byte[]> response;
        try {
            response = http.send(request, HttpResponse.BodyHandlers.ofByteArray());
        } catch (java.net.http.HttpTimeoutException e) {
            throw new RequestTimeoutException("request", uri.toString());
        } catch (IOException e) {
            throw new NetworkException(uri.toString(), e);
        } catch (InterruptedException e) {
            Thread.currentThread().interrupt();
            throw new NetworkException(uri.toString(), e);
        }

        byte[] body = response.body();
        int status = response.statusCode();
        if (status < 200 || status > 299) throw classify(status, response.headers(), body, uri.toString());

        try {
            return mapper.readTree(body);
        } catch (IOException e) {
            throw new SerializationException("body is not valid JSON: " + e.getMessage(), uri.toString());
        }
    }

    <T> T parse(JsonNode node, Class<T> type, String url) {
        try {
            return mapper.treeToValue(node, type);
        } catch (com.fasterxml.jackson.core.JacksonException e) {
            throw new SerializationException(
                "response did not match expected schema: " + e.getMessage(), url);
        }
    }

    private TransportRestException classify(int status, java.net.http.HttpHeaders headers,
                                            byte[] body, String url) {
        String text = new String(body, StandardCharsets.UTF_8).trim();
        String snippet = text.length() > 512 ? text.substring(0, 512) : text;
        JsonNode parsed = null;
        if (text.startsWith("{")) {
            try { parsed = mapper.readTree(text); } catch (IOException ignored) { }
        }
        String message = parsed != null && parsed.hasNonNull("message")
            ? parsed.get("message").asText() : null;
        if (status == 429) {
            Duration retryAfter = null;
            Optional<String> ra = headers.firstValue("Retry-After");
            if (ra.isPresent() && ra.get().matches("\\d+"))
                retryAfter = Duration.ofSeconds(Long.parseLong(ra.get()));
            return new RateLimitedException(url,
                message != null ? message : "rate limited", parsed, retryAfter);
        }
        if (parsed != null)
            return new ApiException(status, url,
                message != null ? message : "unspecified API error", parsed);
        return new HttpException(status, "GET", url, snippet.isEmpty() ? "<no body>" : snippet);
    }

    static String encodePath(String value) {
        return URLEncoder.encode(value, StandardCharsets.UTF_8).replace("+", "%20");
    }

    static void requireNonEmpty(String value, String parameter) {
        if (value == null || value.isBlank())
            throw new InvalidParameterException(parameter, "must not be empty");
    }

    static String boolStr(boolean v) { return v ? "true" : "false"; }

    static String iso(OffsetDateTime when) { return when.format(java.time.format.DateTimeFormatter.ISO_OFFSET_DATE_TIME); }

    static String num(double v) {
        return formatNumber(v);
    }

    private static String formatNumber(double v) {
        if (v == Math.floor(v) && !Double.isInfinite(v)) return String.format(Locale.ROOT, "%.1f", v);
        return String.valueOf(v);
    }
}
