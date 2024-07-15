from dataclasses import dataclass
from typing import Any, Self
from collections.abc import Iterable
from abc import ABC, abstractmethod
from ulid import ULID


from shepherd.ids import ShepherdId, SheepId
from shepherd.epoch import Epoch
from shepherd.sheep.base import PackagedSheep
from shepherd.feed import Feed, Responses


@dataclass(frozen=True)
class Shepherd(ABC):
    """A shepherd within the flock simulation"""

    id: ShepherdId = ShepherdId(ULID())

    @abstractmethod
    def introduce_to(self, sheep: Iterable[PackagedSheep]) -> Self:
        """
        Introduce a Shepherd to an iterable of Sheep

        If this Shepherd is already aware of any of the Sheep referenced by the
        provided sequence of PackagedSheep, its associated information is
        updated
        """
        pass

    @abstractmethod
    def begin(self, epoch: Epoch) -> Self:
        """
        Begin a new Epoch

        This must be able to handle being called without subsequent build_feed
        and incorporate_responses calls being made
        """
        pass

    @abstractmethod
    def build_feed(self, sheep: SheepId) -> tuple[Self, Feed]:
        """Build a Feed for the specified Sheep"""
        pass

    @abstractmethod
    def incorporate_responses(self, sheep: SheepId, responses: Responses) -> Self:
        """Incorporate responses to a Feed from a Sheep into the Shepherd"""
        pass
