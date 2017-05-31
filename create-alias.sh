#!/bin/bash
set -e

P=$(readlink -f $(dirname $0)/git_reorder.py)
git config --global alias.reorder "!$P"

echo 'Created alias "reorder". Use this way:'
echo '   git reorder [number of commits]'
echo 'OR'
echo '   git reorder --root'
