import logging
import sys
from collections.abc import Callable
from typing import Optional

import pandas as pd

from shepherd.shepherd.base import Shepherd
from shepherd.shepherd.dummy import Dummy as DummyShepherd
from shepherd.simulation import Flock, FlockSettings

logger = logging.getLogger(__name__)


def main() -> int:
    flock = Flock([DummyShepherd()], FlockSettings())

    for _ in range(20):
        flock.simulate_epoch()
        print(flock.tags)

    print(flock.tags)

    logging.basicConfig(filename="shepherd.log", level=logging.INFO)

    return 0


if __name__ == "__main__":
    sys.exit(main())
