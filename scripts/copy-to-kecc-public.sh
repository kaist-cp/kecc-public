#!/usr/bin/env bash

rsync --exclude=".git" --delete --archive ./ ../kecc-public
