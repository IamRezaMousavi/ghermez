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
    from PySide6.QtCore import QCoreApplication, QLocale, QPoint, QSettings, QSize, Qt, QThread, QTranslator
    from PySide6.QtGui import QCloseEvent, QIcon, QKeyEvent
    from PySide6.QtWidgets import QInputDialog, QLineEdit, QPushButton, QWidget
except ImportError:
    from PyQt5.QtCore import QCoreApplication, QLocale, QPoint, QSettings, QSize, Qt, QThread, QTranslator
    from PyQt5.QtGui import QCloseEvent, QIcon, QKeyEvent
    from PyQt5.QtWidgets import QInputDialog, QLineEdit, QPushButton, QWidget

import platform
import subprocess

import ghermez
from persepolis.constants import OS
from persepolis.constants.status import DownloadStatus
from persepolis.gui.progress_ui import ProgressWindow_Ui
from persepolis.scripts import download
from persepolis.scripts.bubble import notifySend
from persepolis.scripts.shutdown import shutDown

os_type = platform.system()


class ShutDownThread(QThread):
    def __init__(self, parent: QWidget, gid: str, password: str | None = None) -> None:
        super().__init__()
        self.gid = gid
        self.password = password
        self.parent = parent

    def run(self) -> None:
        shutDown(self.parent, gid=self.gid, password=self.password)


class ProgressWindow(ProgressWindow_Ui):
    def __init__(self, parent: QWidget, gid: str, persepolis_setting: QSettings) -> None:
        super().__init__(persepolis_setting)
        self.persepolis_setting = persepolis_setting
        self.parent = parent
        self.gid = gid
        self.status = None
        self.resume_pushButton.clicked.connect(self.resumePushButtonPressed)
        self.stop_pushButton.clicked.connect(self.stopPushButtonPressed)
        self.pause_pushButton.clicked.connect(self.pausePushButtonPressed)
        self.download_progressBar.setValue(0)
        self.limit_pushButton.clicked.connect(self.limitPushButtonPressed)

        self.limit_frame.setEnabled(False)
        self.limit_checkBox.toggled.connect(self.limitCheckBoxToggled)

        self.after_frame.setEnabled(False)
        self.after_checkBox.toggled.connect(self.afterCheckBoxToggled)

        self.after_pushButton.clicked.connect(self.afterPushButtonPressed)

        # add support for other languages
        locale = str(self.persepolis_setting.value('settings/locale'))
        QLocale.setDefault(QLocale(locale))
        self.translator = QTranslator()
        if self.translator.load(':/translations/locales/ui_' + locale, 'ts'):
            QCoreApplication.installTranslator(self.translator)

        # check if limit speed activated by user or not
        add_link_dictionary = self.parent.persepolis_db.searchGidInAddLinkTable(gid)

        limit = str(add_link_dictionary['limit_value'])
        if limit != '0':
            limit_number = limit[:-1]
            limit_unit = limit[-1]
            self.limit_spinBox.setValue(float(limit_number))
            if limit_unit == 'K':
                self.after_comboBox.setCurrentIndex(0)
            else:
                self.after_comboBox.setCurrentIndex(1)
            self.limit_checkBox.setChecked(True)

        self.after_comboBox.currentIndexChanged.connect(self.afterComboBoxChanged)

        self.limit_comboBox.currentIndexChanged.connect(self.limitComboBoxChanged)

        self.limit_spinBox.valueChanged.connect(self.limitComboBoxChanged)

        # set window size and position
        size = self.persepolis_setting.value('ProgressWindow/size', QSize(595, 274))
        position = self.persepolis_setting.value('ProgressWindow/position', QPoint(300, 300))
        self.resize(size)
        self.move(position)

    # close window with ESC key
    def keyPressEvent(self, event: QKeyEvent) -> None:
        if event.key() == Qt.Key_Escape:
            self.close()

    def closeEvent(self, _event: QCloseEvent) -> None:
        # save window size and position
        self.persepolis_setting.setValue('ProgressWindow/size', self.size())
        self.persepolis_setting.setValue('ProgressWindow/position', self.pos())
        self.persepolis_setting.sync()

        self.hide()

    def resumePushButtonPressed(self, _button: QPushButton) -> None:
        if self.status == DownloadStatus.Paused:
            answer = ghermez.downloadUnpause(self.gid)

            # if aria2 did not respond , then this function is checking for aria2
            # availability , and if aria2 disconnected then aria2Disconnected is
            # executed
            if not (answer):
                version_answer = ghermez.aria2Version()
                if version_answer == 'did not respond':
                    self.parent.aria2Disconnected()
                    notifySend(
                        QCoreApplication.translate('progress_src_ui_tr', 'Aria2 disconnected!'),
                        QCoreApplication.translate(
                            'progress_src_ui_tr',
                            'Persepolis is trying to connect! be patient!',
                        ),
                        10000,
                        'warning',
                        parent=self.parent,
                    )
                else:
                    notifySend(
                        QCoreApplication.translate('progress_src_ui_tr', 'Aria2 did not respond!'),
                        QCoreApplication.translate('progress_src_ui_tr', 'Please try again.'),
                        10000,
                        'warning',
                        parent=self.parent,
                    )

    def pausePushButtonPressed(self, _button: QPushButton) -> None:
        if self.status == DownloadStatus.Downloading:
            answer = ghermez.downloadPause(self.gid)

            # if aria2 did not respond , then this function is checking for aria2
            # availability , and if aria2 disconnected then aria2Disconnected is
            # executed
            if not (answer):
                version_answer = ghermez.aria2Version()
                if version_answer == 'did not respond':
                    self.parent.aria2Disconnected()
                    download.downloadStop(self.gid, self.parent)
                    notifySend(
                        'Aria2 disconnected!',
                        'Persepolis is trying to connect! be patient!',
                        10000,
                        'warning',
                        parent=self.parent,
                    )
                else:
                    notifySend(
                        QCoreApplication.translate('progress_src_ui_tr', 'Aria2 did not respond!'),
                        QCoreApplication.translate('progress_src_ui_tr', 'Try again!'),
                        10000,
                        'critical',
                        parent=self.parent,
                    )

    def stopPushButtonPressed(self, _button: QPushButton) -> None:
        download_dict = {'gid': self.gid, 'shutdown': 'canceled'}

        self.parent.temp_db.updateSingleTable(download_dict)

        answer = download.downloadStop(self.gid, self.parent)

        # if aria2 did not respond , then this function is checking for aria2
        # availability , and if aria2 disconnected then aria2Disconnected is
        # executed
        if answer == 'None':
            version_answer = ghermez.aria2Version()
            if version_answer == 'did not respond':
                self.parent.aria2Disconnected()
                notifySend(
                    QCoreApplication.translate('progress_src_ui_tr', 'Aria2 disconnected!'),
                    QCoreApplication.translate('progress_src_ui_tr', 'Persepolis is trying to connect! be patient!'),
                    10000,
                    'warning',
                    parent=self.parent,
                )

    def limitCheckBoxToggled(self, _checkBoxes: bool) -> None:
        # user checked limit_checkBox
        if self.limit_checkBox.isChecked():
            self.limit_frame.setEnabled(True)
            self.limit_pushButton.setEnabled(True)

        # user unchecked limit_checkBox
        else:
            self.limit_frame.setEnabled(False)

            # check download status is "scheduled" or not!
            if self.status != DownloadStatus.Scheduled:
                # tell aria2 for unlimited speed
                ghermez.limitSpeed(self.gid, '0')
            else:
                # update limit value in data_base
                add_link_dictionary = {'gid': self.gid, 'limit_value': '0'}
                self.parent.persepolis_db.updateAddLinkTable([add_link_dictionary])

    def limitComboBoxChanged(self, _connect: str) -> None:
        self.limit_pushButton.setEnabled(True)

    def afterComboBoxChanged(self, _connect: int) -> None:
        self.after_pushButton.setEnabled(True)

    def afterCheckBoxToggled(self, _checkBoxes: bool) -> None:
        if self.after_checkBox.isChecked():
            self.after_frame.setEnabled(True)
        else:
            # so user canceled shutdown after download
            # write cancel value in data_base for this gid
            self.after_frame.setEnabled(False)

            download_dict = {'gid': self.gid, 'shutdown': 'canceled'}

            self.parent.temp_db.updateSingleTable(download_dict)

    def afterPushButtonPressed(self, _button: QPushButton) -> None:
        self.after_pushButton.setEnabled(False)

        if os_type != OS.WINDOWS:  # For Linux and Mac OSX and FreeBSD and OpenBSD
            # get root password
            passwd, ok = QInputDialog.getText(self, 'PassWord', 'Please enter root password:', QLineEdit.Password)

            if ok:
                # check password is true or not!
                pipe = subprocess.Popen(
                    ['sudo', '-S', 'echo', 'hello'],
                    stdout=subprocess.DEVNULL,
                    stdin=subprocess.PIPE,
                    stderr=subprocess.DEVNULL,
                    shell=False,
                )

                pipe.communicate(passwd.encode())

                answer = pipe.wait()

                # Wrong password
                while answer != 0:
                    passwd, ok = QInputDialog.getText(
                        self,
                        'PassWord',
                        'Wrong Password!\nPlease try again.',
                        QLineEdit.Password,
                    )

                    if ok:
                        # checking password
                        pipe = subprocess.Popen(
                            ['sudo', '-S', 'echo', 'hello'],
                            stdout=subprocess.DEVNULL,
                            stdin=subprocess.PIPE,
                            stderr=subprocess.DEVNULL,
                            shell=False,
                        )

                        pipe.communicate(passwd.encode())

                        answer = pipe.wait()

                    else:
                        ok = False
                        break

                if ok is not False:
                    # if user selects shutdown option after download progress,
                    # value of 'shutdown' will changed in temp_db for this gid
                    # and "wait" word will be written for this value.
                    # (see ShutDownThread and shutdown.py for more information)
                    # shutDown method will check that value in a loop .
                    # when "wait" changes to "shutdown" then shutdown.py script
                    # will shut down the system.
                    shutdown_enable = ShutDownThread(self.parent, self.gid, passwd)
                    self.parent.threadPool.append(shutdown_enable)
                    self.parent.threadPool[-1].start()

                else:
                    self.after_checkBox.setChecked(False)
            else:
                self.after_checkBox.setChecked(False)

        else:  # for Windows
            shutdown_enable = ShutDownThread(self.parent, self.gid)
            self.parent.threadPool.append(shutdown_enable)
            self.parent.threadPool[-1].start()

    def limitPushButtonPressed(self, _button: QPushButton) -> None:
        self.limit_pushButton.setEnabled(False)
        if self.limit_comboBox.currentText() == 'KiB/s':
            limit_value = str(self.limit_spinBox.value()) + 'K'
        else:
            limit_value = str(self.limit_spinBox.value()) + 'M'

        # if download was started before , send the limit_speed request to aria2 .
        # else save the request in data_base
        if self.status != DownloadStatus.Scheduled:
            ghermez.limitSpeed(self.gid, limit_value)
        else:
            # update limit value in data_base
            add_link_dictionary = {'gid': self.gid, 'limit_value': limit_value}
            self.parent.persepolis_db.updateAddLinkTable([add_link_dictionary])

    def changeIcon(self, icons: str) -> None:
        icons = ':/' + str(icons) + '/'

        self.resume_pushButton.setIcon(QIcon(icons + 'play'))
        self.pause_pushButton.setIcon(QIcon(icons + 'pause'))
        self.stop_pushButton.setIcon(QIcon(icons + 'stop'))
