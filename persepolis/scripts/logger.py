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

import logging
import os

import ghermez
from persepolis.constants import APP_NAME

# config_folder
config_folder = ghermez.determineConfigFolder()

# create a directory if it does not exist
if not os.path.exists(config_folder):
    os.makedirs(config_folder)

# log file address
log_file = os.path.join(str(config_folder), f'{APP_NAME}dm.log')

if not os.path.isfile(log_file):
    ghermez.touch(log_file)

# define logging object
LOG_OBJ = logging.getLogger(APP_NAME.capitalize())
LOG_OBJ.setLevel(logging.INFO)

# don't show log in console
LOG_OBJ.propagate = False

# create a file handler
handler = logging.FileHandler(log_file)
handler.setLevel(logging.INFO)
# create a logging format
formatter = logging.Formatter('%(asctime)s - %(name)s - %(levelname)s - %(message)s')
handler.setFormatter(formatter)

# add the handlers to the logger
LOG_OBJ.addHandler(handler)


def sendToLog(text: str = '', level: str = 'INFO') -> None:
    if level == 'INFO':
        LOG_OBJ.info(text)
    elif level == 'ERROR':
        LOG_OBJ.error(text)
    else:
        LOG_OBJ.warning(text)
