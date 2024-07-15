from dataclasses import dataclass
from typing import Self

from shepherd.sheep.base import Sheep
from shepherd.epoch import Epoch
from shepherd.feed import Feed, Responses, Response


@dataclass(frozen=True)
class Dummy(Sheep):
    """
    A dummy Sheep

    This Sheep will only ever rate content served to it in one way, indicated
    to it at initialization
    """

    response: Response = Response.POSITIVE

    def begin(self, epoch: Epoch) -> Self:
        return self

    def process_feed(self, feed: Feed) -> Responses:
        return Responses(items=[Response.POSITIVE for _ in feed.items])
