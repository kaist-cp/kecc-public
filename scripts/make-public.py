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
    for root, dir, files in os.walk("src"):
        for f in files:
            (filename, ext) = os.path.splitext(f)

            if ext == ".public":
                os.rename(os.path.join(root, f), os.path.join(root, filename))
