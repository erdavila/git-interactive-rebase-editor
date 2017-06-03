#!/bin/bash
set -e

GLOBAL=''

until [ -z "$1" ] ; do
  case "$1" in
    --global)
      GLOBAL=--global
      ;;

    *)
      echo "What is '$1'?!" >&2
      exit 1
  esac
  shift
done

APP=$(readlink -f $(dirname $0)/git_reorder.py)
git config $GLOBAL sequence.editor "$APP"

echo "Configured interactive git rebase editor."
echo "See README.md for usage instructions."
