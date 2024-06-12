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

import ast


# This function is writting a list in file_path in dictionary format
def writeList(file_path: str, dir_list: str) -> None:
    dictionary = {'list': dir_list}
    with open(file_path, 'w') as f:
        f.writelines(str(dictionary))


# This function is reading file_path and return content of file in list format
def readList(file_path: str, mode: str = 'dictionary') -> list[str]:
    with open(file_path) as f:
        f_string = f.readline()
    dictionary = ast.literal_eval(f_string.strip())
    dir_list = dictionary['list']

    if mode == 'string':
        dir_list[9] = str(dir_list[9])

    return dir_list


# this function is reading a file that contains dictionary , and extracts
# dictionary from it.
def readDict(file_path: str) -> dict[str, str]:
    with open(file_path) as f:
        f_lines = f.readlines()
    dict_str = str(f_lines[0].strip())
    return ast.literal_eval(dict_str)
