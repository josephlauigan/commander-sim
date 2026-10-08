"""The iPad app: the practice table from commander_sim.play, full screen in a web view. The practice server runs
inside the app on a background thread and listens on this device only, so the table works with no network."""
import toga
from toga.style import Pack

from commander_ipad import bootstrap


class Practice(toga.App):
    def startup(self):
        self.server, url = bootstrap.start(str(self.paths.app), str(self.paths.data))
        self.main_window = toga.MainWindow(title=self.formal_name)
        self.main_window.content = toga.WebView(url=url, style=Pack(flex=1))
        self.main_window.show()


def main():
    return Practice()
