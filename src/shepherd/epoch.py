from dataclasses import dataclass


from shepherd.ids import EpochId, TagId
from shepherd.feed import Item


@dataclass(frozen=True)
class Epoch:
    """
    An epoch within the simulation

    Records the ID of the epoch, in addition to any items or tags introduced
    within it
    """

    id: EpochId
    items: list[Item]
    tags: list[TagId]
