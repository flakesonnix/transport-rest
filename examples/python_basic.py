"""Basic usage of the transport-rest Python binding.

Run: PYTHONPATH=bindings/python python3 examples/python_basic.py
"""

from transport_rest import JourneyPlace, TransportRestClient


def main() -> None:
    client = TransportRestClient()

    locations = client.locations().query("Berlin").results(3).get()
    for location in locations:
        print(f"found: {location.name} ({location.id})")

    board = client.departures("8011160").results(5).get()
    for dep in board.departures:
        name = dep.line.name if dep.line else "?"
        print(f"{name}: {dep.planned_when}")


async def async_demo() -> None:
    import asyncio

    client = TransportRestClient(provider="bvg")
    result = await client.locations().query("Alexanderplatz").get_async()
    print(f"[async] {len(result)} results")


if __name__ == "__main__":
    main()
