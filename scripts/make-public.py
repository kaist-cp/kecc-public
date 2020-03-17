#!/usr/bin/env python3

"""Makes public skeleton.
"""

import os
import subprocess
import itertools
import argparse
import sys
import re

if __name__ == "__main__":
    for fullname in os.listdir("src"):
        (filename, ext) = os.path.splitext(fullname)

        if ext == ".public":
            os.rename(os.path.join("src", fullname), os.path.join("src", filename))
