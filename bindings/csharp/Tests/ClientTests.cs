// Binding tests: create client, request, deserialize, handle error.
// Fully offline against a local HttpListener mock.

using System.Net;
using System.Net.Sockets;
using System.Text;
using TransportRest;
using TransportRest.Models;
using Xunit;

namespace TransportRest.Tests;

public class ClientTests
{
    private static int FreePort()
    {
        var l = new TcpListener(IPAddress.Loopback, 0);
        l.Start();
        int port = ((IPEndPoint)l.LocalEndpoint).Port;
        l.Stop();
        return port;
    }

    private static HttpListener StartMock(Action<HttpListenerContext> respond, int port)
    {
        var listener = new HttpListener();
        listener.Prefixes.Add($"http://127.0.0.1:{port}/");
        listener.Start();
        Task.Run(async () =>
        {
            while (listener.IsListening)
            {
                try
                {
                    var ctx = await listener.GetContextAsync();
                    respond(ctx);
                }
                catch (Exception) when (!listener.IsListening)
                {
                    break;
                }
            }
        });
        return listener;
    }

    [Fact]
    public async Task Locations_ParsesResults_AndSendsQuery()
    {
        string? seenQuery = null;
        int port = FreePort();
        var listener = StartMock(ctx =>
        {
            seenQuery = ctx.Request.Url?.Query;
            byte[] body = Encoding.UTF8.GetBytes(
                """[{"type":"stop","id":"8011160","name":"Berlin Hbf"}]""");
            ctx.Response.ContentType = "application/json";
            ctx.Response.ContentLength64 = body.Length;
            ctx.Response.OutputStream.Write(body);
            ctx.Response.Close();
        }, port);

        using var client = new TransportRestClient(baseUrl: $"http://127.0.0.1:{port}/");
        var result = await client.Locations().Query("Berlin").Results(5).GetAsync();
        listener.Stop();

        Assert.Single(result);
        Assert.Equal("8011160", result[0].Raw.GetProperty("id").GetString());
        Assert.Contains("query=Berlin", seenQuery);
    }

    [Fact]
    public void MissingQuery_ThrowsBeforeRequest()
    {
        using var client = new TransportRestClient(baseUrl: "http://127.0.0.1:1/");
        var builder = client.Locations();
        Assert.Throws<InvalidParameterException>(() => builder.GetAsync());
    }
}
