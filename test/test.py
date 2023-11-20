#!/usr/bin/env python3  # noqa: INP001

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

import os
import platform
import sys

# finding os platform
os_type = platform.system()

# Don't run persepolis as root!
if os_type in ('Linux', 'FreeBSD', 'OpenBSD', 'Darwin'):
    uid = os.getuid()
    if uid == 0:
        print('Do not run persepolis as root.')  # noqa: T201
        sys.exit(1)


cwd = os.path.abspath(__file__)
run_dir = os.path.dirname(cwd)
# if ghermez run in test folder
print('ghermez is running from test folder')  # noqa: T201
parent_dir = os.path.dirname(run_dir)

sys.path.insert(0, parent_dir)

from persepolis import __main__  # noqa: E402, F401
