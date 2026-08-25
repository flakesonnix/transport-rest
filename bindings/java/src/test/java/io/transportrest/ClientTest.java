// Binding tests: create client, request, deserialize, handle error.
// Fully offline against com.sun.net.httpserver.HttpServer.

package io.transportrest;

import com.fasterxml.jackson.databind.JsonNode;
import com.sun.net.httpserver.HttpServer;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.net.InetSocketAddress;
import java.nio.charset.StandardCharsets;

import static io.transportrest.Errors.ApiException;
import static io.transportrest.Errors.InvalidParameterException;
import static org.junit.jupiter.api.Assertions.*;

class ClientTest {
    private HttpServer server;
    private String baseUrl;
    private TransportRestClient client;

    @BeforeEach
    void setUp() throws IOException {
        server = HttpServer.create(new InetSocketAddress("127.0.0.1", 0), 0);
        server.createContext("/locations", exchange -> {
            byte[] body = """
                [{"type":"stop","id":"8011160","name":"Berlin Hbf"}]"""
                .getBytes(StandardCharsets.UTF_8);
            exchange.getResponseHeaders().add("Content-Type", "application/json");
            exchange.sendResponseHeaders(200, body.length);
            try (var out = exchange.getResponseBody()) { out.write(body); }
        });
        server.createContext("/stops/nope", exchange -> {
            byte[] body = "{\"message\":\"Stop not found.\"}".getBytes(StandardCharsets.UTF_8);
            exchange.sendResponseHeaders(404, body.length);
            try (var out = exchange.getResponseBody()) { out.write(body); }
        });
        server.start();
        baseUrl = "http://127.0.0.1:" + server.getAddress().getPort();
        client = TransportRestClient.newClient().provider("db").baseUrl(baseUrl).build();
    }

    @AfterEach
    void tearDown() {
        server.stop(0);
    }

    @Test
    void locationsHappyPath() {
        JsonNode result = client.locations().query("Berlin").results(5).get();
        assertEquals(1, result.size());
        assertEquals("8011160", result.get(0).get("id").asText());
    }

    @Test
    void missingQueryFailsClientSide() {
        assertThrows(InvalidParameterException.class,
            () -> client.locations().get());
    }

    @Test
    void apiError404IsStructured() {
        ApiException ex = assertThrows(ApiException.class,
            () -> client.stop("nope").get());
        assertEquals(404, ex.status);
        assertTrue(ex.getMessage().contains("Stop not found."));
    }
}
