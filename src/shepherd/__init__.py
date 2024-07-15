from shepherd.simulation import Flock, FlockSettings
from shepherd.feed import Response
from shepherd.shepherd.dummy import Dummy as DummyShepherd
from shepherd.sheep.dummy import Dummy as DummySheep


def main():
    flock = Flock([DummyShepherd()], [DummySheep()], FlockSettings())

    for _ in range(20):
        flock.simulate_epoch()

    print(flock.tags)

    return 0
