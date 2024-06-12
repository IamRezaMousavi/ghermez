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
    from PySide6.QtCore import QSettings, QSize, Qt
    from PySide6.QtGui import QIcon, QKeyEvent
    from PySide6.QtWidgets import QHBoxLayout, QLabel, QPushButton, QTextEdit, QVBoxLayout, QWidget
except ImportError:
    from PyQt5.QtCore import QSettings, QSize, Qt
    from PyQt5.QtGui import QIcon, QKeyEvent
    from PyQt5.QtWidgets import QHBoxLayout, QLabel, QPushButton, QTextEdit, QVBoxLayout, QWidget

import ghermez
from persepolis.constants import APP_NAME, LONG_NAME, ORG_NAME, REPO_LINK
from persepolis.gui import resources  # noqa: F401


class ErrorWindow(QWidget):
    def __init__(self, text: str) -> None:
        super().__init__()
        # finding windows_size
        self.setMinimumSize(QSize(363, 300))
        self.setWindowIcon(QIcon.fromTheme(APP_NAME, QIcon(':/ghermez.png')))
        self.setWindowTitle(LONG_NAME)

        verticalLayout = QVBoxLayout(self)
        horizontalLayout = QHBoxLayout()
        horizontalLayout.addStretch(1)

        self.text_edit = QTextEdit(self)
        self.text_edit.setReadOnly(True)
        self.text_edit.insertPlainText(text)

        verticalLayout.addWidget(self.text_edit)

        self.label2 = QLabel(self)
        self.label2.setText(
            'Reseting persepolis may solving problem.\nDo not panic!If you add your download links again,\npersepolis will resume your downloads\nPlease copy this error message and press "Report Issue" button\nand open a new issue in Github for it.\nWe answer you as soon as possible. \nreporting this issue help us to improve persepolis.\nThank you!', # noqa: E501
        )
        verticalLayout.addWidget(self.label2)

        self.report_pushButton = QPushButton(self)
        self.report_pushButton.setText('Report Issue')
        horizontalLayout.addWidget(self.report_pushButton)

        self.reset_persepolis_pushButton = QPushButton(self)
        self.reset_persepolis_pushButton.clicked.connect(self.resetPushButtonPressed)
        self.reset_persepolis_pushButton.setText(f'Reset {APP_NAME.capitalize()}')
        horizontalLayout.addWidget(self.reset_persepolis_pushButton)

        self.close_pushButton = QPushButton(self)
        self.close_pushButton.setText('close')
        horizontalLayout.addWidget(self.close_pushButton)

        verticalLayout.addLayout(horizontalLayout)

        self.report_pushButton.clicked.connect(self.reportPushButtonPressed)
        self.close_pushButton.clicked.connect(self.closePushButtonPressed)

    def reportPushButtonPressed(self, _button: QPushButton) -> None:
        ghermez.xdgOpen(f'{REPO_LINK}/issues')

    # close window with ESC key
    def keyPressEvent(self, event: QKeyEvent) -> None:
        if event.key() == Qt.Key_Escape:
            self.close()

    def closePushButtonPressed(self, _button: QPushButton) -> None:
        self.close()

    def resetPushButtonPressed(self, _button: QPushButton) -> None:
        # create an object for DataBase
        persepolis_db = ghermez.DataBase()

        # Reset data base
        persepolis_db.resetDataBase()

        # close connections
        del persepolis_db

        # Reset persepolis_setting
        persepolis_setting = QSettings(ORG_NAME, APP_NAME)
        persepolis_setting.clear()
        persepolis_setting.sync()
