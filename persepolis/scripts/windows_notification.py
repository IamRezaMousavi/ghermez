#    This program is free software: you can redistribute it and/or modify
#    it under the terms of the GNU General Public License as published by
#    the Free Software Foundation, either version 3 of the License, or
#    (at your option) any later version.
#
#    This program is distributed in the hope that it will be useful,
#    but WITHOUT ANY WARRANTY; without even the implied warranty of
#    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#    GNU General Public License for more details.
#
#    You should have received a copy of the GNU General Public License
#    along with this program.  If not, see <http://www.gnu.org/licenses/>.

try:
    from PySide6.QtCore import QSettings, Qt, QThread, Signal
    from PySide6.QtGui import QCloseEvent, QKeyEvent, QMouseEvent
    from PySide6.QtWidgets import QWidget
except ImportError:
    from PyQt5.QtCore import QSettings, Qt, QThread
    from PyQt5.QtCore import pyqtSignal as Signal
    from PyQt5.QtGui import QCloseEvent, QKeyEvent, QMouseEvent
    from PyQt5.QtWidgets import QWidget

from time import sleep

from persepolis.gui.windows_notification_ui import WindowsNotification_UI


class TimerThread(QThread):
    TIMEISUP = Signal()

    def __init__(self, time: str) -> None:
        super().__init__()
        self.time = float(int(time) / 1000)

    def run(self) -> None:
        sleep(self.time)
        self.TIMEISUP.emit()


class WindowsNotification(WindowsNotification_UI):
    def __init__(self, parent: QWidget, time: str, text1: str, text2: str, persepolis_setting: QSettings) -> None:
        super().__init__(parent, persepolis_setting)

        # run timer and close notification after time is up.
        timer = TimerThread(time)
        parent.threadPool.append(timer)
        parent.threadPool[-1].start()
        parent.threadPool[-1].TIMEISUP.connect(self.close)

        # set text to the labels
        self.label1.setText(str(text1))
        self.label2.setText(str(text2))

    def mousePressEvent(self, _event: QMouseEvent) -> None:
        self.close()

    # close window with ESC key
    def keyPressEvent(self, event: QKeyEvent) -> None:
        if event.key() == Qt.Key_Escape:
            self.close()

    def closeEvent(self, event: QCloseEvent) -> None:
        event.accept()
