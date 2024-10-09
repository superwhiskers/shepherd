from collections.abc import Iterable
from ulid import ULID
from random import randrange, choices
from dataclasses import dataclass
from typing import Optional
import networkx as nx


from shepherd.shepherd.base import Shepherd
from shepherd.sheep.base import Sheep
from shepherd.ids import ShepherdId, SheepId, ItemId, EpochId, TagId
from shepherd.epoch import Epoch
from shepherd.feed import Item
from shepherd.graph import SimulationGraph


@dataclass(frozen=True)
class PastureState:
    """State contained within a Pasture value"""

    shepherd: Shepherd
    sheep: dict[SheepId, list[ItemId]]


@dataclass(frozen=True)
class FlockSettings:
    """Settings for the simulation"""

    """Bounds on the number of tags added at the start of each epoch"""
    n_tags_bounds: tuple[Optional[int], int] = (None, 5)

    """Bounds on the number of items added at the start of each epoch"""
    n_items_bounds: tuple[Optional[int], int] = (None, 50)

    """Bounds on the number of tags assigned to a new Item"""
    n_item_tags_bounds: tuple[int, int] = (1, 10)

    """The bounds on the initial number of tags used to seed the simulation"""
    initial_n_tags_bounds: tuple[Optional[int], int] = (20, 40)

    """
    The bounds on the initial number of items used to seed the simulation
    """
    initial_n_items_bounds: tuple[Optional[int], int] = (40, 60)

    """
    An approximate measure of how many tags belong in a group

    This is used to determine the upper limit on how many groups should be
    added when there is a sufficient amount of tags orphaned from a group
    """
    average_tags_per_group: int = 7

    """The threshold of orphaned tags at which new groups will be formed"""
    orphaned_tag_threshold: int = 20


class Flock:
    """A flock simulation object"""

    simulation_graph: SimulationGraph
    sheep: dict[SheepId, Sheep]
    pasture: dict[ShepherdId, PastureState] = {}
    settings: FlockSettings

    def __init__(
        self,
        shepherds: Iterable[Shepherd],
        sheep: Iterable[Sheep],
        settings: FlockSettings,
    ) -> None:
        self.settings = settings
        self.sheep = {sheep.id: sheep for sheep in sheep}

        for shepherd in shepherds:
            self.pasture[shepherd.id] = PastureState(
                shepherd=shepherd, sheep={sheep.id: [] for sheep in self.sheep.values()}
            )

        # it's probably a good idea to fill the tag list with some initial
        # amount to counteract there not being many filled in subsequent epochs
        self.tags = [
            TagId(ULID())
            for _ in range(
                randrange(self.settings.n_tags_bounds[1])
                if self.settings.n_tags_bounds[0] is None
                else randrange(
                    self.settings.n_tags_bounds[0], self.settings.n_tags_bounds[1]
                )
            )
        ]

        temporary_epoch = Epoch(id=EpochId(ULID()), items=[], tags=self.tags)

        self.sheep = {
            id: sheep.begin(temporary_epoch) for (id, sheep) in self.sheep.items()
        }
        self.pasture = {
            id: PastureState(
                shepherd=state.shepherd.begin(temporary_epoch).introduce_to(
                    map(lambda s: s.package(), self.sheep.values())
                ),
                sheep=state.sheep,
            )
            for (id, state) in self.pasture.items()
        }

    def simulate_epoch(self) -> None:
        # TODO: generate a graph of tag associations with probabilities linking
        #       them so we can more accurately model how tags actually work in
        #       practice. right now, it's completely random which isn't
        #       reflective of how content is actually tagged in real life

        new_tags = [
            TagId(ULID())
            for _ in range(
                randrange(self.settings.n_tags_bounds[1])
                if self.settings.n_tags_bounds[0] is None
                else randrange(
                    self.settings.n_tags_bounds[0], self.settings.n_tags_bounds[1]
                )
            )
        ]
        self.tags.extend(new_tags)

        new_item_ids = [
            ItemId(ULID())
            for _ in range(
                randrange(self.settings.n_items_bounds[1])
                if self.settings.n_items_bounds[0] is None
                else randrange(
                    self.settings.n_items_bounds[0], self.settings.n_items_bounds[1]
                )
            )
        ]

        new_items = [
            Item(
                id=id,
                tags=choices(
                    self.tags,
                    k=randrange(
                        self.settings.n_item_tags_bounds[0],
                        self.settings.n_item_tags_bounds[1],
                    ),
                ),
            )
            for id in new_item_ids
        ]

        epoch = Epoch(id=EpochId(ULID()), items=new_items, tags=new_tags)

        self.sheep = {id: sheep.begin(epoch) for (id, sheep) in self.sheep.items()}
        self.pasture = {
            id: PastureState(
                shepherd=state.shepherd.begin(epoch).introduce_to(
                    map(lambda s: s.package(), self.sheep.values())
                ),
                sheep=state.sheep,
            )
            for (id, state) in self.pasture.items()
        }

        new_pasture = {}
        for id, state in self.pasture.items():
            shepherd: Shepherd
            new_sheep = {}
            for sheep_id, seen in state.sheep.items():
                shepherd, feed = state.shepherd.build_feed(sheep_id)
                seen.extend(feed.items)
                shepherd = shepherd.incorporate_responses(
                    sheep_id, self.sheep[sheep_id].process_feed(feed)
                )
                new_sheep[sheep_id] = seen

            new_pasture[id] = PastureState(shepherd=shepherd, sheep=new_sheep)
        self.pasture = new_pasture
