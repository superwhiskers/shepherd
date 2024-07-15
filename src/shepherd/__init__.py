from typing import Optional
from textual.app import App, ComposeResult
from textual.widgets import Header, Footer


from shepherd.simulation import Flock, FlockSettings
from shepherd.feed import Response
from shepherd.shepherd.dummy import Dummy as DummyShepherd
from shepherd.sheep.dummy import Dummy as DummySheep


class ShepherdApp(App[None]):
    """The shepherd flock simulator application"""

    flock: Optional[Flock] = None

    def compose(self) -> ComposeResult:
        """Builds child widgets for the application"""

        yield Header()
        yield Footer()


def main():
    """
    flock = Flock([DummyShepherd()], [DummySheep()], FlockSettings())

    for _ in range(20):
        flock.simulate_epoch()

    print(flock.tags)
    """

    app = ShepherdApp()
    app.run()

    return 0
