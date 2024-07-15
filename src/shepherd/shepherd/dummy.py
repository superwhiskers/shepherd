# necessary until 3.13
from __future__ import annotations
from dataclasses import dataclass, field
from typing import Self
from collections.abc import Iterable
from random import choices


from shepherd.shepherd.base import Shepherd
from shepherd.sheep.base import PackagedSheep
from shepherd.epoch import Epoch
from shepherd.feed import Item, Feed, Responses
from shepherd.ids import SheepId


@dataclass(frozen=True)
class Dummy(Shepherd):
    """
    A dummy Shepherd

    This Shepherd will only ever select random links to give to Sheep. It does
    not take into account any of the Sheep's preferences
    """

    known_items: list[Item] = field(default_factory=lambda: [])

    def introduce_to(self, sheep: Iterable[PackagedSheep]) -> Self:
        return self

    def begin(self, epoch: Epoch) -> Dummy:
        return Dummy(id=self.id, known_items=self.known_items + epoch.items)

    def build_feed(self, sheep: SheepId) -> tuple[Self, Feed]:
        return (
            self,
            Feed(items=choices(list(map(lambda item: item.id, self.known_items)), k=5)),
        )

    def incorporate_responses(self, sheep: SheepId, responses: Responses) -> Self:
        return self
