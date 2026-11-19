import os  # $ import=imports-forms-import-os
import os.path as osp  # $ import=imports-forms-import-os-path
from json import dumps, loads as parse  # $ import=imports-forms-import-json-dumps import=imports-forms-import-json-loads
from . import sibling  # $ import=imports-forms-import-sibling
from ..pkg.mod import name  # $ import=imports-forms-import-pkg-mod-name
from csv import *  # $ import=imports-forms-import-csv
from typing import (
    Any,  # $ import=imports-forms-import-typing-any
    Optional,  # $ import=imports-forms-import-typing-optional
)


def local():
    import re  # $ import=imports-forms-local-import-re
