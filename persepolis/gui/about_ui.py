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
    from PySide6.QtCore import QCoreApplication, QLocale, QSettings, QSize, Qt, QTranslator
    from PySide6.QtGui import QFont, QIcon
    from PySide6.QtWidgets import QHBoxLayout, QLabel, QPushButton, QTabWidget, QTextEdit, QVBoxLayout, QWidget

    pyside6_is_installed = True
except ImportError:
    from PyQt5.QtCore import QCoreApplication, QLocale, QSettings, QSize, Qt, QTranslator
    from PyQt5.QtGui import QFont, QIcon
    from PyQt5.QtWidgets import QHBoxLayout, QLabel, QPushButton, QTabWidget, QTextEdit, QVBoxLayout, QWidget

    pyside6_is_installed = False

from persepolis.constants import APP_NAME, LONG_NAME, REPO_LINK, VERSION
from persepolis.gui import resources  # noqa: F401

if pyside6_is_installed:
    try:
        from PySide6 import QtSvgWidget

        qtsvg_available = True
    except ImportError:
        qtsvg_available = False
else:
    try:
        from PyQt5 import QtSvg as QtSvgWidget

        qtsvg_available = True
    except ImportError:
        qtsvg_available = False
qtsvg_available = False


class AboutWindow_Ui(QWidget):  # noqa: N801
    def __init__(self, persepolis_setting: QSettings) -> None:
        super().__init__()

        self.persepolis_setting = persepolis_setting

        # add support for other languages
        locale = str(self.persepolis_setting.value('settings/locale'))
        QLocale.setDefault(QLocale(locale))
        self.translator = QTranslator()
        if self.translator.load(':/translations/locales/ui_' + locale, 'ts'):
            QCoreApplication.installTranslator(self.translator)

        # set ui direction
        ui_direction = self.persepolis_setting.value('ui_direction')

        if ui_direction == 'rtl':
            self.setLayoutDirection(Qt.RightToLeft)

        elif ui_direction in 'ltr':
            self.setLayoutDirection(Qt.LeftToRight)

        icons = ':/' + str(self.persepolis_setting.value('settings/icons')) + '/'

        self.setMinimumSize(QSize(545, 375))
        self.setWindowIcon(QIcon.fromTheme(APP_NAME, QIcon(':/ghermez.png')))

        verticalLayout = QVBoxLayout(self)

        self.about_tabWidget = QTabWidget(self)

        # about tab
        self.about_tab = QWidget(self)

        about_tab_horizontalLayout = QHBoxLayout(self.about_tab)

        about_tab_verticalLayout = QVBoxLayout()

        # persepolis icon
        if qtsvg_available:
            persepolis_icon_verticalLayout = QVBoxLayout()
            self.persepolis_icon = QtSvgWidget.QSvgWidget(':/persepolis.svg')
            self.persepolis_icon.setFixedSize(QSize(64, 64))

            persepolis_icon_verticalLayout.addWidget(self.persepolis_icon)
            persepolis_icon_verticalLayout.addStretch(1)

            about_tab_horizontalLayout.addLayout(persepolis_icon_verticalLayout)

        self.title_label = QLabel(self.about_tab)
        font = QFont()
        font.setBold(True)
        font.setWeight(QFont.Weight.Bold)
        self.title_label.setFont(font)
        self.title_label.setAlignment(Qt.AlignCenter)
        about_tab_verticalLayout.addWidget(self.title_label)

        self.version_label = QLabel(self.about_tab)
        self.version_label.setAlignment(Qt.AlignCenter)

        about_tab_verticalLayout.addWidget(self.version_label)

        self.site2_label = QLabel(self.about_tab)
        self.site2_label.setTextFormat(Qt.RichText)
        self.site2_label.setAlignment(Qt.AlignCenter)
        self.site2_label.setOpenExternalLinks(True)
        self.site2_label.setTextInteractionFlags(Qt.TextBrowserInteraction)
        about_tab_verticalLayout.addWidget(self.site2_label)

        about_tab_verticalLayout.addStretch(1)

        about_tab_horizontalLayout.addLayout(about_tab_verticalLayout)

        # developers_tab
        # developers
        self.developers_tab = QWidget(self)
        developers_verticalLayout = QVBoxLayout(self.developers_tab)

        self.developers_title_label = QLabel(self.developers_tab)
        font.setBold(True)
        font.setWeight(QFont.Weight.Bold)
        self.developers_title_label.setFont(font)
        self.developers_title_label.setAlignment(Qt.AlignCenter)
        developers_verticalLayout.addWidget(self.developers_title_label)

        self.name_label = QLabel(self.developers_tab)
        self.name_label.setAlignment(Qt.AlignCenter)

        developers_verticalLayout.addWidget(self.name_label)

        # contributors
        self.contributors_thank_label = QLabel(self.developers_tab)
        self.contributors_thank_label.setFont(font)
        self.contributors_thank_label.setAlignment(Qt.AlignCenter)

        developers_verticalLayout.addWidget(self.contributors_thank_label)

        self.contributors_link_label = QLabel(self.developers_tab)
        self.contributors_link_label.setTextFormat(Qt.RichText)
        self.contributors_link_label.setAlignment(Qt.AlignCenter)
        self.contributors_link_label.setOpenExternalLinks(True)
        self.contributors_link_label.setTextInteractionFlags(Qt.TextBrowserInteraction)
        developers_verticalLayout.addWidget(self.contributors_link_label)

        developers_verticalLayout.addStretch(1)

        # translators tab
        self.translators_tab = QWidget(self)
        translators_tab_verticalLayout = QVBoxLayout(self.translators_tab)

        # translators
        self.translators_textEdit = QTextEdit(self.translators_tab)
        self.translators_textEdit.setReadOnly(True)
        translators_tab_verticalLayout.addWidget(self.translators_textEdit)

        # License tab
        self.license_tab = QWidget(self)
        license_tab_verticalLayout = QVBoxLayout(self.license_tab)

        self.license_text = QTextEdit(self.license_tab)
        self.license_text.setReadOnly(True)

        license_tab_verticalLayout.addWidget(self.license_text)

        verticalLayout.addWidget(self.about_tabWidget)

        # buttons
        button_horizontalLayout = QHBoxLayout()
        button_horizontalLayout.addStretch(1)

        self.pushButton = QPushButton(self)
        self.pushButton.setIcon(QIcon(icons + 'ok'))
        self.pushButton.clicked.connect(self.close)

        button_horizontalLayout.addWidget(self.pushButton)

        verticalLayout.addLayout(button_horizontalLayout)

        self.setWindowTitle(QCoreApplication.translate('about_ui_tr', f'About {APP_NAME.capitalize()}'))

        # about_tab
        self.title_label.setText(QCoreApplication.translate('about_ui_tr', LONG_NAME))
        self.version_label.setText(QCoreApplication.translate('about_ui_tr', f'Version {VERSION}'))
        self.site2_label.setText(
            QCoreApplication.translate(
                'about_ui_tr',
                f'<a href={REPO_LINK}>{REPO_LINK}</a>',
                "TRANSLATORS NOTE: YOU REALLY DON'T NEED TO TRANSLATE THIS PART!",
            ),
        )

        # developers_tab
        self.developers_title_label.setText(QCoreApplication.translate('about_ui_tr', 'Developers:'))

        self.name_label.setText(
            QCoreApplication.translate(
                'about_ui_tr',
                'Reza Mousavi\nAliReza AmirSamimi\nMohammadreza Abdollahzadeh\nSadegh Alirezaie\nMostafa Asadi\nMohammadAmin Vahedinia\nJafar Akhondali\nH.Rostami\nEhsan Titish',  # noqa: E501
                "TRANSLATORS NOTE: YOU REALLY DON'T NEED TO TRANSLATE THIS PART!",
            ),
        )

        self.contributors_thank_label.setText(QCoreApplication.translate('about_ui_tr', 'Special thanks to:'))
        self.contributors_link_label.setText(f'<a href={REPO_LINK}/graphs/contributors>our contributors</a>')

        # License
        self.license_text.setPlainText('''
            This program is free software: you can redistribute it and/or modify
            it under the terms of the GNU General Public License as published by
            the Free Software Foundation, either version 3 of the License, or
            (at your option) any later version.

            This program is distributed in the hope that it will be useful,
            but WITHOUT ANY WARRANTY; without even the implied warranty of
            MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
            GNU General Public License for more details.

            You should have received a copy of the GNU General Public License
            along with this program.  If not, see http://www.gnu.org/licenses/.
            ''')

        # tabs
        self.about_tabWidget.addTab(
            self.about_tab,
            QCoreApplication.translate('about_ui_tr', f'About {APP_NAME.capitalize()}'),
        )
        self.about_tabWidget.addTab(self.developers_tab, QCoreApplication.translate('about_ui_tr', 'Developers'))
        self.about_tabWidget.addTab(self.translators_tab, QCoreApplication.translate('about_ui_tr', 'Translators'))
        self.about_tabWidget.addTab(self.license_tab, QCoreApplication.translate('about_ui_tr', 'License'))

        # button
        self.pushButton.setText(QCoreApplication.translate('about_ui_tr', 'OK'))
