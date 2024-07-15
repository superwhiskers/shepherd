from dataclasses import dataclass, field
from abc import ABC, abstractmethod
from typing import NamedTuple, Self
from ulid import ULID


from shepherd.ids import SheepId, ItemId, TagId
from shepherd.epoch import Epoch
from shepherd.feed import Responses, Feed


@dataclass(frozen=True)
class AssociatedTag(NamedTuple):
    """A Tag associated with a Sheep"""

    id: TagId
    negated: bool


@dataclass(frozen=True)
class PackagedSheep(NamedTuple):
    """The packaged form of a Sheep, ready for a Shepherd to be introduced to it"""

    id: SheepId
    tags: list[AssociatedTag]


@dataclass(frozen=True)
class Sheep(ABC):
    """
    A sheep within the flock simulation

    Holds the Sheep's preferred tag combination, and its liked Items (the items
    the Shepherd is supposed to try to put into a Feed)
    """

    id: SheepId = SheepId(ULID())
    tags: list[AssociatedTag] = field(default_factory=lambda: [])
    liked: list[ItemId] = field(default_factory=lambda: [])

    def package(self) -> PackagedSheep:
        """Package this SheepState for consumption by a Shepherd"""
        return PackagedSheep(id=self.id, tags=self.tags)

    @abstractmethod
    def begin(self, epoch: Epoch) -> Self:
        """Begin a new Epoch"""
        pass

    @abstractmethod
    def process_feed(self, feed: Feed) -> Responses:
        """Process a Feed and yield Responses"""
        pass
