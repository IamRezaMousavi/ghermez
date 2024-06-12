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


# THIS FILE CONTAINING SOME VARIABLES , ... THAT USING FOR INITIALIZING PERSEPOLIS

import os

import ghermez
from persepolis.constants import APP_NAME, ORG_NAME
from persepolis.scripts.browser_integration import browserIntegration
from persepolis.scripts.useful_tools import returnDefaultSettings

try:
    from PySide6.QtCore import QSettings
except ImportError:
    from PyQt5.QtCore import QSettings

# initialization

# create folders
ghermez.init_create_folders()

# persepolisdm.log file contains persepolis log.
ghermez.initLogger()

# refresh logs!
ghermez.init_log_file()

# create an object for DataBase
persepolis_db = ghermez.DataBase()

# create tables
persepolis_db.createTables()

# close connections
del persepolis_db

# create an object for PluginsDB
plugins_db = ghermez.PluginsDB()

# create tables
plugins_db.createTables()

# delete old links
plugins_db.deleteOldLinks()

# close connections
del plugins_db


# load persepolis_setting
# persepolis is using QSettings for saving windows size and windows
# position and program settings.

persepolis_setting = QSettings(ORG_NAME, APP_NAME)

persepolis_setting.beginGroup('settings')

default_setting_dict = returnDefaultSettings()
# this loop is checking values in persepolis_setting . if value is not
# valid then value replaced by default_setting_dict value
for key in default_setting_dict:
    setting_value = persepolis_setting.value(key, default_setting_dict[key])
    persepolis_setting.setValue(key, setting_value)

# download files is downloading in temporary folder(download_path_temp)
# and then they will be moved to user download folder(download_path) after completion.
# Check that mount point is available of not!
if not (os.path.exists(persepolis_setting.value('download_path_temp'))):
    persepolis_setting.setValue('download_path_temp', default_setting_dict['download_path_temp'])

if not (os.path.exists(persepolis_setting.value('download_path'))):
    persepolis_setting.setValue('download_path', default_setting_dict['download_path'])


persepolis_setting.sync()

# this section  creates temporary download folder and download folder and
# download sub folders if they did not existed.
download_path_temp = persepolis_setting.value('download_path_temp')
download_path = persepolis_setting.value('download_path')


folder_list = [download_path_temp, download_path]

# add subfolders to folder_list if user checked subfolders check box in setting window.
if persepolis_setting.value('subfolder') == 'yes':
    for folder in ['Audios', 'Videos', 'Others', 'Documents', 'Compressed']:
        folder_list.append(os.path.join(download_path, folder))  # noqa: PERF401

# create folders in folder_list
for folder in folder_list:
    ghermez.makeDirs(folder)

persepolis_setting.endGroup()

# Browser integration for Firefox and chromium and google chrome
for browser in ['chrome', 'chromium', 'opera', 'vivaldi', 'firefox', 'brave']:
    json_done, native_done = browserIntegration(browser)

    log_message = browser

    if json_done:
        log_message = log_message + ': ' + 'Json file is created successfully.\n'

    else:
        log_message = log_message + ': ' + 'Json ERROR!\n'

    if native_done:
        log_message = log_message + 'persepolis executer file is created successfully.\n'

    elif native_done is False:
        log_message = log_message + ': ' + 'persepolis executer file ERROR!\n'

    ghermez.sendToLog(log_message)

# get locale and set ui direction
locale = str(persepolis_setting.value('settings/locale'))

# right to left languages
rtl_locale_list = ['fa_IR', 'ar']

# left to right languages
ltr_locale_list = ['en_US', 'zh_CN', 'fr_FR', 'pl_PL', 'nl_NL', 'pt_BR', 'es_ES', 'hu', 'tr', 'tr_TR']

if locale in rtl_locale_list:
    persepolis_setting.setValue('ui_direction', 'rtl')
else:
    persepolis_setting.setValue('ui_direction', 'ltr')

# compatibility
persepolis_version = float(persepolis_setting.value('version/version', 2.5))
if persepolis_version < 2.6:  # noqa: PLR2004
    from persepolis.scripts.compatibility import compatibility

    try:
        compatibility()
    except Exception as e:
        # create an object for DataBase
        persepolis_db = ghermez.DataBase()

        # create tables
        persepolis_db.resetDataBase()

        # close connections
        del persepolis_db

        # write error in log
        ghermez.sendToLog('compatibility ERROR!', 'ERROR')
        ghermez.sendToLog(str(e), 'ERROR')

    persepolis_version = 2.6

if persepolis_version < 3.1:  # noqa: PLR2004
    # create an object for DataBase
    persepolis_db = ghermez.DataBase()

    # correct data base
    persepolis_db.correctDataBase()

    # close connections
    del persepolis_db

    persepolis_version = 3.1

if persepolis_version < 3.2:  # noqa: PLR2004
    persepolis_setting.beginGroup('settings')

    for key in default_setting_dict:
        setting_value = default_setting_dict[key]
        persepolis_setting.setValue(key, setting_value)

    persepolis_setting.endGroup()

    persepolis_setting.setValue('version/version', 3.2)

persepolis_setting.sync()
