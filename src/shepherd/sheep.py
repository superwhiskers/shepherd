import random
from itertools import pairwise
from math import floor

import networkx as nx
from scipy.stats import pmean

from shepherd.feed import Feed, Response, Responses
from shepherd.graph import SimulationGraph
from shepherd.ids import SheepId


def p_positive(weight: float) -> float:
    """Calculate the probability of a positive rating given the input sum of weights along the shortest path"""
    return 2 ** (-weight)


def p_neutral(weight: float) -> float:
    """Calculate the probability of a neutral rating given the input sum of weights along the shortest path"""
    return (9**weight) / (10**weight)


def calculate_response_from_weight(weight: float) -> Response:
    """Calculate the response given the input sum of weights along the shortest path"""
    chance = random.random()
    if chance <= p_positive(weight):
        return Response.POSITIVE
    elif chance <= p_neutral(weight):
        return Response.NEUTRAL
    else:
        return Response.NEGATIVE


def process_feed(
    graph: SimulationGraph, sheep: SheepId, feed: Feed
) -> Responses:
    """Process a feed given the tag graph, sheep id, and feed"""
    responses = []

    for item in feed.items:
        match list(
            nx.all_shortest_paths(
                graph.graph,
                source=sheep,
                target=item,
                weight=lambda _a, _b, e: floor(e["weight"]),
            )
        ):
            case []:
                # to keep the model simple, we always respond negatively to
                # content for which no path exists
                #
                # the assumptions being made here for this to work are:
                # - the tag graph is taken to be axiomatic
                # - everything is comprehensively tagged and no more existing
                #   tags fit
                responses.append(Response.NEGATIVE)
            case [path]:
                responses.append(
                    calculate_response_from_weight(
                        sum(
                            map(
                                lambda t: graph.graph.edges[t[0], t[1]][
                                    "weight"
                                ],
                                pairwise(path),
                            )
                        )
                    )
                )
            case paths:
                responses.append(
                    calculate_response_from_weight(
                        float(
                            pmean(
                                list(
                                    map(
                                        lambda path: sum(
                                            map(
                                                lambda t: graph.graph.edges[
                                                    t[0], t[1]
                                                ]["weight"],
                                                pairwise(path),
                                            )
                                        ),
                                        paths,
                                    )
                                ),
                                2,
                            )
                        )
                    )
                )

    return Responses(items=responses)
