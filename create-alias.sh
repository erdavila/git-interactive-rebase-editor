#!/bin/bash
set -e

P=$(readlink -f $(dirname $0)/git_reorder.py)
git config --global alias.reorder "!$P"

echo 'Created alias "reorder" for Git.'
echo 'See README.md for usage instructions.'
