from shepherd.feed import Feed, Responses
from shepherd.graph import SimulationGraph
from shepherd.ids import SheepId


def process_feed(
    graph: SimulationGraph, id: SheepId, feed: Feed
) -> Responses:
    return Responses(items=[])
