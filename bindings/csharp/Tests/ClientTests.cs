// Binding tests: create client, request, deserialize, handle error.
// Fully offline against a local Kestrel-free HttpListener mock.
using System.Net;
using System.Text;
using TransportRest;
using TransportRest.Models;
using Xunit;

namespace TransportRest.Tests;

public class ClientTests
{
    private static HttpListener StartMock(out string baseUrl)
    {
        var listener = new HttpListener();
        listener.Prefixes.Add("http://127.0.0.1:0/");
        listener.Start();
        var port = ((IPEndPoint)listener.LocalEndpoint).Port;
        baseUrl = $"http://127.0.0.1:{port}/";
        return listener;
    }

    [Fact]
    public async Task Locations_ParsesResults_AndSendsQuery()
    {
        string? seenQuery = null;
        var listener = StartMock(out var baseUrl);
        _ = Task.Run(async () =>
        {
            while (listener.IsListening)
            {
                var ctx = await listener.GetContextAsync();
                seenQuery = ctx.Request.Url?.Query;
                var body = Encoding.UTF8.GetBytes(
                    """[{"type":"stop","id":"8011160","name":"Berlin Hbf"}]""");
                ctx.Response.ContentType = "application/json";
                ctx.Response.ContentLength64 = body.Length;
                await ctx.Response.OutputStream.WriteAsync(body);
                ctx.Response.Close();
            }
        });

        using var client = new TransportRestClient(baseUrl: baseUrl);
        var result = await client.Locations().Query("Berlin").Results(5).GetAsync();

        Assert.Single(result);
        Assert.Equal("8011160", result[0].Id);
        Assert.Contains("query=Berlin", seenQuery);
        listener.Stop();
    }

    [Fact]
    public void MissingQuery_ThrowsBeforeRequest()
    {
        using var client = new TransportRestClient(baseUrl: "http://127.0.0.1:1/");
        Assert.Throws<InvalidParameterException>(() => client.Locations().GetAsync());
    }
}
