from typing import Optional
import pandas as pd
import logging
from collections.abc import Callable

from shepherd.simulation import Flock, FlockSettings
from shepherd.feed import Response
from shepherd.sheep.base import Sheep
from shepherd.shepherd.base import Shepherd
from shepherd.shepherd.dummy import Dummy as DummyShepherd
from shepherd.sheep.dummy import Dummy as DummySheep


logger = logging.getLogger(__name__)


class Simulation:
    """
    A wrapper over the simulation which makes handling it easier

    This handles things such as evenly dividing up Sheep and Shepherds for you
    """

    flock: Flock
    epochs: list[pd.Series]

    def __init__(
        self,
        sheep_pool: int,
        sheep_distribution: list[tuple[Callable[[], Sheep], float]],
        shepherds: list[Shepherd],
        settings: FlockSettings,
    ) -> None:
        sheep = []

        total = 0.0
        for make_sheep, percentage in sheep_distribution:
            if percentage < 0.0:
                raise Exception("the sheep distribution has negative percentages")

            for _ in range(round(sheep_pool * percentage)):
                sheep.append(make_sheep())

        if total != 1.0:
            raise Exception("the sheep distribution does not sum to 1.0")

        self.flock = Flock(shepherds, sheep, settings)
        self.epochs = []

        return

    def simulate_epoch(self) -> None:
        self.flock.simulate_epoch()

        self.epochs.append(pd.Series({"avg_satisfaction": None}))


def main():
    """
    flock = Flock([DummyShepherd()], [DummySheep()], FlockSettings())

    for _ in range(20):
        flock.simulate_epoch()

    print(flock.tags)
    """

    logging.basicConfig(filename="shepherd.log", level=logging.INFO)

    flock = Simulation(100, [(DummySheep, 0.5), (DummySheep, 0.4)], [])

    return 0
