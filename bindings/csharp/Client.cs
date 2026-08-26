// Handwritten client core for the transport.rest C# binding.
using System.Text;
using System.Text.Json;
using TransportRest.Models;

namespace TransportRest;

public static class Providers
{
    public const string Db = "https://v6.db.transport.rest";
    public const string Bvg = "https://v6.bvg.transport.rest";
    public const string Vbb = "https://v6.vbb.transport.rest";
    public const string Poland = "https://poland.transport.rest";

    public static readonly IReadOnlyDictionary<string, string> All = new Dictionary<string, string>
    {
        ["db"] = Db,
        ["bvg"] = Bvg,
        ["vbb"] = Vbb,
        ["poland"] = Poland,
    };

    public static readonly IReadOnlyDictionary<string, string[]> Capabilities =
        new Dictionary<string, string[]>
    {
        ["db"] = new[] { "stations" },
        ["bvg"] = new[] { "stops_search", "radar", "reachable_from", "trips_by_name" },
        ["vbb"] = new[] { "stops_search", "radar", "reachable_from", "trips_by_name" },
        ["poland"] = new[] { "radar", "reachable_from", "trips_by_name" },
    };
}

/// <summary>Structured error taxonomy mirroring the Rust core.</summary>
public class TransportRestException : Exception
{
    protected TransportRestException(string message) : base(message) { }
}

public class NetworkException : TransportRestException
{
    public NetworkException(string url, Exception cause)
        : base($"network error for {url}: {cause.Message}") => Url = url;

    public string Url { get; }
}

public class RequestTimeoutException : TransportRestException
{
    public RequestTimeoutException(string kind, string? url)
        : base($"request timed out ({kind}){(url is null ? "" : $" for {url}")}")
    {
        Kind = kind;
        Url = url;
    }

    public string Kind { get; }
    public string? Url { get; }
}

public class HttpException : TransportRestException
{
    public HttpException(int status, string method, string url, string bodySnippet)
        : base($"unexpected HTTP response: HTTP {status} from {method} {url}: {bodySnippet}")
    {
        Status = status;
        Method = method;
        Url = url;
        BodySnippet = bodySnippet;
    }

    public int Status { get; }
    public string Method { get; }
    public string Url { get; }
    public string BodySnippet { get; }
}

public class ApiException : TransportRestException
{
    public ApiException(int status, string url, string message, JsonElement? body)
        : base(string.IsNullOrEmpty(message) ? $"API error (HTTP {status})" : message)
    {
        Status = status;
        Url = url;
        Message_ = message;
        Body = body;
    }

    public int Status { get; }
    public string Url { get; }
    public string Message_ { get; }
    public JsonElement? Body { get; }
}

public class RateLimitedException : ApiException
{
    private static readonly string[] RetryHeader = { "Retry-After" };

    public RateLimitedException(string url, string message, JsonElement? body, TimeSpan? retryAfter)
        : base(429, url,
            retryAfter is { } ra
                ? $"rate limited (HTTP 429), retry after {ra.TotalSeconds:F0}s: {message}"
                : $"rate limited (HTTP 429): {message}", body)
    {
        RetryAfter = retryAfter;
    }

    public TimeSpan? RetryAfter { get; }
}

public class SerializationException : TransportRestException
{
    public SerializationException(string reason, string? url)
        : base($"failed to deserialize response{(url is null ? "" : $" for {url}")}: {reason}")
    {
        Reason = reason;
    }

    public string Reason { get; }
}

public class InvalidParameterException : TransportRestException
{
    public InvalidParameterException(string? parameter, string reason)
        : base($"invalid parameter '{parameter ?? "<none>"}': {reason}")
    {
        Parameter = parameter;
    }

    public string? Parameter { get; }
}

public class CapabilityNotSupportedException : TransportRestException
{
    public CapabilityNotSupportedException(string capability, string provider)
        : base($"capability '{capability}' is not supported by provider '{provider}'")
    {
        Capability = capability;
        ProviderName = provider;
    }

    public string Capability { get; }
    public string ProviderName { get; }
}

/// <summary>A place referenced by journey queries.</summary>
public sealed record JourneyPlace
{
    public static JourneyPlace FromStopId(string id) => new() { Form = "id", Id = id };
    public static JourneyPlace FromName(string name) => new() { Form = "name", Name = name };
    public static JourneyPlace Poi(string id, double lat, double lon) =>
        new() { Form = "poi", Id = id, Latitude = lat, Longitude = lon };
    public static JourneyPlace Address(double lat, double lon, string address) =>
        new() { Form = "address", Latitude = lat, Longitude = lon, Address_ = address };

    public string? Form { get; init; }
    public string? Id { get; init; }
    public string? Name { get; init; }
    public string? Address_ { get; init; }
    public double? Latitude { get; init; }
    public double? Longitude { get; init; }

    internal void Encode(string prefix, List<(string, string)> parameters)
    {
        switch (Form)
        {
            case "id":
                parameters.Add((prefix, Id ?? ""));
                break;
            case "name":
                parameters.Add((prefix + ".name", Name ?? ""));
                break;
            case "poi":
                parameters.Add((prefix + ".id", Id ?? ""));
                parameters.Add((prefix + ".latitude", Latitude!.Value.ToString(System.Globalization.CultureInfo.InvariantCulture)));
                parameters.Add((prefix + ".longitude", Longitude!.Value.ToString(System.Globalization.CultureInfo.InvariantCulture)));
                break;
            case "address":
                parameters.Add((prefix + ".latitude", Latitude!.Value.ToString(System.Globalization.CultureInfo.InvariantCulture)));
                parameters.Add((prefix + ".longitude", Longitude!.Value.ToString(System.Globalization.CultureInfo.InvariantCulture)));
                parameters.Add((prefix + ".address", Address_ ?? ""));
                break;
        }
    }

    internal void Validate(string parameter)
    {
        if (Form == "poi" && string.IsNullOrWhiteSpace(Id))
            throw new InvalidParameterException(parameter + ".id", "POI id must not be empty");
        if (Form == "address" && string.IsNullOrWhiteSpace(Address_))
            throw new InvalidParameterException(parameter + ".address", "address must not be empty");
    }
}

/// <summary>Client for one transport.rest instance. Thread-safe.</summary>
public sealed class TransportRestClient : IDisposable
{
    private readonly HttpClient http;
    private readonly bool ownsHttpClient;
    private readonly string baseUrl;
    private readonly int timeoutMs;
    private readonly string userAgent;
    private readonly long maxResponseBytes;
    private readonly string provider;
    private readonly HashSet<string> capabilities;

    public TransportRestClient(
        string provider = "db",
        string? baseUrl = null,
        HttpClient? httpClient = null,
        int timeoutMs = 30_000,
        string userAgent = "transport-rest-cs/0.1.0",
        long maxResponseBytes = 16 * 1024 * 1024,
        IEnumerable<string>? enableCapabilities = null)
    {
        this.provider = provider;
        this.baseUrl = (baseUrl ?? Providers.All.GetValueOrDefault(provider)
            ?? throw new InvalidParameterException("base_url",
                $"unknown provider '{provider}' requires a base URL")).TrimEnd('/');
        this.timeoutMs = timeoutMs;
        this.userAgent = userAgent;
        this.maxResponseBytes = maxResponseBytes;
        ownsHttpClient = httpClient is null;
        http = httpClient ?? new HttpClient { Timeout = TimeSpan.FromMilliseconds(timeoutMs) };
        capabilities = new HashSet<string>(Providers.Capabilities.GetValueOrDefault(provider) ?? Array.Empty<string>());
        foreach (var cap in enableCapabilities ?? Enumerable.Empty<string>())
            capabilities.Add(cap);
    }

    public string ProviderName => provider;
    public string BaseUrl => baseUrl;

    public bool Supports(string capability) => capabilities.Contains(capability);

    private void CheckCapability(string capability)
    {
        if (!capabilities.Contains(capability))
            throw new CapabilityNotSupportedException(capability, provider);
    }

    // -- resource accessors --------------------------------------------------

    public LocationsBuilder Locations() => new(this);

    public DeparturesBuilder Departures(string stopId) => DeparturesBuilder.Departures(this, stopId);
    public ArrivalsAdapter Arrivals(string stopId) => DeparturesBuilder.Arrivals(this, stopId);

    public JourneysBuilder Journeys(JourneyPlace from, JourneyPlace to) => new(this, from, to);

    public TripBuilder Trip(string tripId) => new(this, tripId);

    public RadarBuilder Radar() => new(this);

    // -- execution -----------------------------------------------------------

    internal async Task<T> GetJsonAsync<T>(
        string path,
        List<(string Key, string Value)> parameters,
        Func<JsonElement, T> parse,
        string? capability = null,
        CancellationToken cancellationToken = default)
    {
        if (capability is not null) CheckCapability(capability);

        var query = string.Join("&", parameters.Select(p =>
            $"{Uri.EscapeDataString(p.Key)}={Uri.EscapeDataString(p.Value)}"));
        var url = $"{baseUrl}{path}{(query.Length > 0 ? "?" : "")}{query}";

        using var request = new HttpRequestMessage(HttpMethod.Get, url);
        request.Headers.Accept.ParseAdd("application/json");
        request.Headers.UserAgent.ParseAdd(userAgent);

        HttpResponseMessage response;
        try
        {
            response = await http.SendAsync(request, cancellationToken);
        }
        catch (TaskCanceledException) when (!cancellationToken.IsCancellationRequested)
        {
            throw new RequestTimeoutException("request", url);
        }
        catch (HttpRequestException ex)
        {
            throw new NetworkException(url, ex);
        }

        await using var stream = await response.Content.ReadAsStreamAsync(cancellationToken);
        using var buffered = new MemoryStream();
        await stream.CopyToAsync(buffered, cancellationToken);
        if (buffered.Length > maxResponseBytes)
            throw new SerializationException($"response exceeds configured maximum of {maxResponseBytes} bytes", url);

        var bodyBytes = buffered.ToArray();
        if ((int)response.StatusCode < 200 || (int)response.StatusCode > 299)
            throw ClassifyError(response, bodyBytes, url);

        JsonElement document;
        try
        {
            document = JsonDocument.Parse(bodyBytes).RootElement.Clone();
        }
        catch (JsonException ex)
        {
            throw new SerializationException($"body is not valid JSON: {ex.Message}", url);
        }
        try
        {
            return parse(document);
        }
        catch (JsonException ex)
        {
            throw new SerializationException($"response did not match expected schema: {ex.Message}", url);
        }
    }

    private static TransportRestException ClassifyError(HttpResponseMessage response, byte[] body, string url)
    {
        var status = (int)response.StatusCode;
        var text = Encoding.UTF8.GetString(body).Trim();
        var snippet = text.Length > 512 ? text[..512] : text;

        JsonElement? parsed = null;
        string? message = null;
        if (text.StartsWith('{'))
        {
            try
            {
                var doc = JsonDocument.Parse(text);
                parsed = doc.RootElement.Clone();
                if (doc.RootElement.TryGetProperty("message", out var m) && m.ValueKind == JsonValueKind.String)
                    message = m.GetString();
            }
            catch (JsonException) { /* fall through to HttpException */ }
        }

        if (status == 429)
        {
            TimeSpan? retryAfter = null;
            if (response.Headers.RetryAfter?.Delta is { } delta)
                retryAfter = delta;
            else if (response.Headers.RetryAfter?.Date is { } date)
                retryAfter = date - DateTimeOffset.UtcNow;
            return new RateLimitedException(url, message ?? "rate limited", parsed, retryAfter);
        }
        if (parsed is not null)
            return new ApiException(status, url, message ?? "unspecified API error", parsed);
        return new HttpException(status, "GET", url, string.IsNullOrEmpty(snippet) ? "<no body>" : snippet);
    }

    public void Dispose()
    {
        if (ownsHttpClient) http.Dispose();
    }
}
