from collections.abc import Iterable
from dataclasses import dataclass
from random import choices, randint
from typing import Optional

import networkx as nx
from ulid import ULID

import shepherd.sheep as sheep
from shepherd.epoch import Epoch
from shepherd.feed import Item
from shepherd.graph import SimulationGraph
from shepherd.ids import EpochId, ItemId, SheepId, ShepherdId, TagId
from shepherd.shepherd.base import Shepherd


@dataclass(frozen=True)
class PastureState:
    """
    State contained within a Pasture value

    This structure maps a Shepherd to a Sheep and a list of ItemIds which
    correspond to Items a Sheep has been given by the Shepherd
    """

    shepherd: Shepherd
    sheep: dict[SheepId, list[ItemId]]


@dataclass(frozen=True)
class FlockSettings:
    """Settings for the simulation"""

    """Bounds on the number of tags added at the start of each epoch"""
    n_tags_bounds: tuple[int, int] = (0, 5)

    """Bounds on the number of items added at the start of each epoch"""
    n_items_bounds: tuple[int, int] = (0, 50)

    """Bounds on the number of tags assigned to a new Item"""
    n_item_tags_bounds: tuple[int, int] = (5, 10)

    """Bounds on the number of tags a sheep has"""
    n_sheep_tags_bounds: tuple[int, int] = (5, 25)

    """Bounds on the initial number of tags used to seed the simulation"""
    initial_n_tags_bounds: tuple[int, int] = (20, 40)

    """Bounds on the initial number of items used to seed the simulation"""
    initial_n_items_bounds: tuple[int, int] = (40, 60)

    """Bounds on the initial number of sheep added to the simulation"""
    initial_n_sheep_bounds: tuple[int, int] = (20, 40)

    """
    An approximate measure of how many tags belong in a group

    This is used to determine the upper limit on how many groups should be
    added when there is a sufficient amount of tags orphaned from a group

    Additionally, this is used when adding the first tags. This should be
    at most the lower bound of initial_n_tags_bounds, but ideally much lower
    than that
    """
    average_tags_per_group: int = 7

    """
    The threshold of orphaned tags at which new groups will be formed

    This should be at most the lower bound of initial_n_tags_bounds
    """
    orphaned_tag_threshold: int = 20


class Flock:
    """A flock simulation object"""

    simulation_graph: SimulationGraph
    pastures: dict[ShepherdId, PastureState] = {}
    settings: FlockSettings
    epochs: list[Epoch]

    # we may be able to grab these from the graph but i don't think it'd be
    # very efficient to do often
    tags: list[TagId]
    tag_groups: list[list[TagId]]
    tag_orphans: list[TagId]

    # likewise here
    sheep: list[SheepId]
    items: list[ItemId]

    def __init__(
        self,
        shepherds: Iterable[Shepherd],
        settings: FlockSettings,
    ) -> None:
        self.simulation_graph = SimulationGraph()
        self.settings = settings
        self.tag_orphans = []
        self.epochs = []

        self.tags = [
            TagId(ULID())
            for _ in range(randint(*self.settings.initial_n_tags_bounds))
        ]
        self.simulation_graph.add_tags(self.tags)
        self.tag_groups, tag_orphans = (
            self.simulation_graph.add_new_tag_groups(
                len(self.tags) // self.settings.average_tags_per_group,
                self.tags,
            )
        )
        self.tag_orphans.extend(tag_orphans)

        self.sheep = [
            SheepId(ULID())
            for _ in range(randint(*self.settings.initial_n_sheep_bounds))
        ]
        self.simulation_graph.add_sheep(self.sheep)
        self.simulation_graph.connect_extremities(
            self.sheep, self.tags, self.settings.n_sheep_tags_bounds
        )

        for shepherd in shepherds:
            self.pastures[shepherd.id] = PastureState(
                shepherd=shepherd,
                sheep={sheep: [] for sheep in self.sheep},
            )

        self.items = [
            ItemId(ULID())
            for _ in range(randint(*self.settings.initial_n_items_bounds))
        ]
        self.simulation_graph.add_items(self.items)
        self.simulation_graph.connect_extremities(
            self.items, self.tags, self.settings.n_item_tags_bounds
        )

        introduction_epoch = Epoch(
            id=EpochId(ULID()), items=[], tags=self.tags
        )
        self.epochs.append(introduction_epoch)

        self.pastures = {
            id: PastureState(
                shepherd=state.shepherd.begin(
                    introduction_epoch
                ).introduce_to(
                    [
                        (sheep, list(self.simulation_graph.graph[sheep]))
                        for sheep in self.sheep
                    ]
                ),
                sheep=state.sheep,
            )
            for id, state in self.pastures.items()
        }

    def simulate_epoch(self) -> None:
        new_tags = [
            TagId(ULID())
            for _ in range(randint(*self.settings.n_tags_bounds))
        ]
        self.tag_groups, tag_orphans = (
            self.simulation_graph.add_to_tag_groups(self.tag_groups, new_tags)
        )
        self.tag_orphans.extend(tag_orphans)
        self.tags.extend(new_tags)

        new_items = [
            ItemId(ULID())
            for _ in range(randint(*self.settings.n_items_bounds))
        ]
        self.simulation_graph.add_items(new_items)
        self.simulation_graph.connect_extremities(
            new_items, self.tags, self.settings.n_item_tags_bounds
        )
        self.items.extend(new_items)

        epoch = Epoch(
            id=EpochId(ULID()),
            items=[
                Item(id=id, tags=list(self.simulation_graph.graph[id]))
                for id in new_items
            ],
            tags=new_tags,
        )
        self.epochs.append(epoch)

        # TODO: alter sheep preferences here by some minute amount

        self.pastures = {
            id: PastureState(
                shepherd=state.shepherd.begin(epoch).introduce_to(
                    [
                        (sheep, list(self.simulation_graph.graph[sheep]))
                        for sheep in self.sheep
                    ]
                ),
                sheep=state.sheep,
            )
            for (id, state) in self.pastures.items()
        }

        new_pastures = {}
        for id, state in self.pastures.items():
            shepherd = state.shepherd
            new_sheep = {}
            for sheep_id, seen in state.sheep.items():
                shepherd, feed = shepherd.build_feed(sheep_id)
                seen.extend(feed.items)
                shepherd = shepherd.incorporate_responses(
                    sheep_id,
                    sheep.process_feed(self.simulation_graph, sheep_id, feed),
                )
                new_sheep[sheep_id] = seen
            new_pastures[id] = PastureState(
                shepherd=shepherd, sheep=new_sheep
            )
        self.pastures = new_pastures
