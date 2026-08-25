"""transport-rest-py: typed client for the transport.rest transit APIs.

Pure standard-library implementation; sync and async usage::

    from transport_rest import TransportRestClient

    client = TransportRestClient()          # Deutsche Bahn instance
    stops = client.locations().query("Berlin").results(5).get()

    stops = await client.locations().query("Berlin").get_async()
"""

from .client import PROVIDERS, TransportRestClient
from .builders import JourneyPlace, ProductSelection
from . import errors, models_gen

__version__ = "0.1.0"

__all__ = [
    "TransportRestClient",
    "JourneyPlace",
    "ProductSelection",
    "PROVIDERS",
    "errors",
    "models_gen",
    "__version__",
]
