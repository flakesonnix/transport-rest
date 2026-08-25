// Handwritten endpoint builders for the transport.rest C# binding.
using System.Globalization;
using System.Text.Json;
using TransportRest.Models;

namespace TransportRest;

public static class BuilderUtil
{
    public static string B(bool v) => v ? "true" : "false";
    public static string Num(double v) => v.ToString(CultureInfo.InvariantCulture);
    public static string Iso(DateTimeOffset when) => when.ToString("yyyy-MM-dd'T'HH:mm:sszzz", CultureInfo.InvariantCulture);
}

public sealed class ProductSelection
{
    private readonly List<(string Key, bool Enabled)> entries = new();

    public ProductSelection Set(string key, bool enabled)
    {
        for (var i = 0; i < entries.Count; i++)
            if (entries[i].Key == key) { entries[i] = (key, enabled); return this; }
        entries.Add((key, enabled));
        return this;
    }

    public ProductSelection NationalExpress(bool v) => Set("nationalExpress", v);
    public ProductSelection National(bool v) => Set("national", v);
    public ProductSelection RegionalExpress(bool v) => Set("regionalExpress", v);
    public ProductSelection Regional(bool v) => Set("regional", v);
    public ProductSelection Suburban(bool v) => Set("suburban", v);
    public ProductSelection Subway(bool v) => Set("subway", v);
    public ProductSelection Tram(bool v) => Set("tram", v);
    public ProductSelection Bus(bool v) => Set("bus", v);
    public ProductSelection Ferry(bool v) => Set("ferry", v);
    public ProductSelection Taxi(bool v) => Set("taxi", v);
    public ProductSelection Express(bool v) => Set("express", v);

    internal void Encode(List<(string, string)> parameters)
    {
        foreach (var (key, enabled) in entries)
            parameters.Add((key, enabled ? "true" : "false"));
    }
}

internal abstract class BuilderBase<T>
{
    protected readonly TransportRestClient client;
    private readonly Func<JsonElement, T> parse;
    private readonly string path;
    private readonly string? capability;

    protected BuilderBase(TransportRestClient client, string path,
        Func<JsonElement, T> parse, string? capability = null)
    {
        this.client = client;
        this.path = path;
        this.parse = parse;
        this.capability = capability;
    }

    private List<(string, string)> current = new();
    protected List<(string, string)> Parameters { get; } = new();

    protected void SetParameters(List<(string, string)> replacement)
    {
        Parameters.Clear();
        Parameters.AddRange(replacement);
    }

    protected void Require(bool condition, string? parameter, string reason)
    {
        if (!condition) throw new InvalidParameterException(parameter, reason);
    }

    public Task<T> GetAsync(CancellationToken cancellationToken = default) =>
        client.GetJsonAsync(path, Parameters, parse, capability, cancellationToken);

}

public sealed class LocationsBuilder : BuilderBase<IReadOnlyList<LocationResult>>
{
    private string? query;

    internal LocationsBuilder(TransportRestClient client)
        : base(client, "/locations", d => JsonSerializer.Deserialize<IReadOnlyList<LocationResult>>(d)!) { }

    public LocationsBuilder Query(string q)
    {
        Require(!string.IsNullOrWhiteSpace(q), "query", "a non-empty search term is required");
        query = q;
        return this;
    }

    public LocationsBuilder Results(int n) { Parameters.Add(("results", n.ToString())); return this; }
    public LocationsBuilder Fuzzy(bool v) { Parameters.Add(("fuzzy", BuilderUtil.B(v))); return this; }
    public LocationsBuilder Language(string l) { Parameters.Add(("language", l)); return this; }

    public new Task<IReadOnlyList<LocationResult>> GetAsync(CancellationToken ct = default)
    {
        Require(query is not null, "query", "Query() is required before GetAsync()");
        Parameters.Insert(0, ("query", query!));
        return base.GetAsync(ct);
    }
}

public abstract class BoardBuilder<T> : BuilderBase<T>
{
    protected BoardBuilder(TransportRestClient client, string path, Func<JsonElement, T> parse)
        : base(client, path, parse) { }

    public BoardBuilder<T> When(DateTimeOffset when) { Parameters.Add(("when", BuilderUtil.Iso(when))); return this; }
    public BoardBuilder<T> Direction(string d) { Parameters.Add(("direction", d)); return this; }
    public BoardBuilder<T> Duration(int minutes) { Parameters.Add(("duration", minutes.ToString())); return this; }
    public BoardBuilder<T> Results(int n) { Parameters.Add(("results", n.ToString())); return this; }
    public BoardBuilder<T> Stopovers(bool v) { Parameters.Add(("stopovers", BuilderUtil.B(v))); return this; }
    public BoardBuilder<T> Remarks(bool v) { Parameters.Add(("remarks", BuilderUtil.B(v))); return this; }
    public BoardBuilder<T> MoreStops(IEnumerable<string> ids) { Parameters.Add(("moreStops", string.Join(",", ids))); return this; }

    public BoardBuilder<T> Products(Func<ProductSelection, ProductSelection> configure)
    {
        configure(new ProductSelection()).Encode(Parameters);
        return this;
    }
}

public sealed class DeparturesBuilder : BoardBuilder<DeparturesResponse>
{
    internal static DeparturesBuilder Departures(TransportRestClient c, string stopId) =>
        new(c, $"/stops/{Uri.EscapeDataString(stopId)}/departures");

    internal static ArrivalsAdapter Arrivals(TransportRestClient c, string stopId) =>
        new(c, $"/stops/{Uri.EscapeDataString(stopId)}/arrivals");

    private DeparturesBuilder(TransportRestClient client, string path)
        : base(client, path,
            d => JsonSerializer.Deserialize<DeparturesResponse>(d)!)
    {
        Require(!string.IsNullOrWhiteSpace(stopId), "stop_id", "must not be empty");
    }
}

public sealed class ArrivalsAdapter
{
    private readonly TransportRestClient client;
    private readonly string path;

    internal ArrivalsAdapter(TransportRestClient client, string path)
    {
        this.client = client;
        this.path = path;
    }

    public Task<ArrivalsResponse> GetAsync(CancellationToken ct = default) =>
        client.GetJsonAsync(path, new List<(string, string)>(),
            d => JsonSerializer.Deserialize<ArrivalsResponse>(d)!);
}

public sealed class JourneysBuilder : BuilderBase<JourneysResponse>
{
    private readonly JourneyPlace from;
    private readonly JourneyPlace to;
    private JourneyPlace? via;

    internal JourneysBuilder(TransportRestClient client, JourneyPlace from, JourneyPlace to)
        : base(client, "/journeys", d => JsonSerializer.Deserialize<JourneysResponse>(d)!)
    {
        this.from = from;
        this.to = to;
    }

    public JourneysBuilder Via(JourneyPlace place) { via = place; return this; }

    public JourneysBuilder Departure(DateTimeOffset when) { Parameters.Add(("departure", BuilderUtil.Iso(when))); return this; }
    public JourneysBuilder Arrival(DateTimeOffset when) { Parameters.Add(("arrival", BuilderUtil.Iso(when))); return this; }
    public JourneysBuilder EarlierThan(string reference) { Parameters.Add(("earlierThan", reference)); return this; }
    public JourneysBuilder LaterThan(string reference) { Parameters.Add(("laterThan", reference)); return this; }
    public JourneysBuilder Results(int n) { Parameters.Add(("results", n.ToString())); return this; }
    public JourneysBuilder Transfers(int n) { Parameters.Add(("transfers", n.ToString())); return this; }

    public JourneysBuilder Products(Func<ProductSelection, ProductSelection> configure)
    {
        configure(new ProductSelection()).Encode(Parameters);
        return this;
    }

    public new Task<JourneysResponse> GetAsync(CancellationToken ct = default)
    {
        from.Validate("from");
        to.Validate("to");
        via?.Validate("via");
        var params2 = new List<(string, string)>();
        from.Encode("from", params2);
        to.Encode("to", params2);
        via?.Encode("via", params2);
        params2.AddRange(Parameters);
        SetParameters(params2);
        return base.GetAsync(ct);
    }
}

public sealed class TripBuilder : BuilderBase<TripResponse>
{
    internal TripBuilder(TransportRestClient client, string tripId)
        : base(client, $"/trips/{Uri.EscapeDataString(tripId)}",
            d => JsonSerializer.Deserialize<TripResponse>(d)!)
    {
        Require(!string.IsNullOrWhiteSpace(tripId), "trip_id", "must not be empty");
    }

    public TripBuilder Stopovers(bool v) { Parameters.Add(("stopovers", BuilderUtil.B(v))); return this; }
    public TripBuilder Remarks(bool v) { Parameters.Add(("remarks", BuilderUtil.B(v))); return this; }
    public TripBuilder Polyline(bool v) { Parameters.Add(("polyline", BuilderUtil.B(v))); return this; }
}

public sealed class RadarBuilder : BuilderBase<RadarResponse>
{
    private readonly Dictionary<string, double> box = new();

    internal RadarBuilder(TransportRestClient client)
        : base(client, "/radar", d => JsonSerializer.Deserialize<RadarResponse>(d)!, "radar") { }

    public RadarBuilder North(double v) { box["north"] = v; return this; }
    public RadarBuilder West(double v) { box["west"] = v; return this; }
    public RadarBuilder South(double v) { box["south"] = v; return this; }
    public RadarBuilder East(double v) { box["east"] = v; return this; }
    public RadarBuilder Results(int n) { Parameters.Add(("results", n.ToString())); return this; }

    public new Task<RadarResponse> GetAsync(CancellationToken ct = default)
    {
        Require(box.Count == 4, "bbox", "north, west, south and east are all required");
        foreach (var (key, value) in box.OrderBy(kv => kv.Key))
            Parameters.Add((key, BuilderUtil.Num(value)));
        return base.GetAsync(ct);
    }
}
