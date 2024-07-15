from dataclasses import dataclass
from typing import NamedTuple
from enum import Enum, auto


from shepherd.ids import TagId, ItemId


class Response(Enum):
    """An enum indicating a Sheep's response to a Feed Item"""

    POSITIVE = auto()
    NEUTRAL = auto()
    NEGATIVE = auto()


@dataclass(frozen=True)
class Responses:
    """
    The responses returned from a Sheep after evaluating a Feed

    These must be in the same order as those provided in the original Feed,
    to prevent an incorrect assignment of Sheep sentiment toward the Feed's
    Items
    """

    items: list[Response]


@dataclass(frozen=True)
class Item:
    """
    An item within the simulation

    This can represent anything. All it has is an ID and associated tags
    """

    id: ItemId
    tags: list[TagId]


@dataclass(frozen=True)
class Feed:
    """The feed returned from a Shepherd, prepared for specific Sheep"""

    items: list[ItemId]
