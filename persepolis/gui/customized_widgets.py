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
from __future__ import annotations

try:
    from PySide6.QtCore import QSettings, Qt
    from PySide6.QtWidgets import QDateTimeEdit, QWidget
except ImportError:
    from PyQt5.QtCore import QSettings, Qt
    from PyQt5.QtWidgets import QDateTimeEdit, QWidget

from persepolis.constants import APP_NAME, ORG_NAME

# load persepolis_setting
persepolis_setting = QSettings(ORG_NAME, APP_NAME)

# check ui_direction RTL or LTR
ui_direction = persepolis_setting.value('ui_direction')


class MyQDateTimeEdit(QDateTimeEdit):
    def __init__(self, parent: QWidget | None=None) -> None:
        super().__init__(parent)

        # change ui direction from rtl to ltr
        if ui_direction == 'rtl':
            self.setLayoutDirection(Qt.LeftToRight)

