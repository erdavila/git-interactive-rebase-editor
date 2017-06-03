#!/bin/bash
set -e

NAME=visual-rebase
GLOBAL=''

until [ -z "$1" ] ; do
  case "$1" in
    --global)
      GLOBAL=--global
      ;;

    --name)
      shift
      if [ -z $1 ]; then
        echo "What name?!" >&2
        exit 1
      fi
      NAME=$1
      ;;

    *)
      echo "What is '$1'?!" >&2
      exit 1
  esac
  shift
done

APP=$(readlink -f $(dirname $0)/git_visual_rebase.py)
git config $GLOBAL alias.$NAME "-c sequence.editor='$APP' rebase -i"

echo "Created alias '$NAME' for Git."
echo "See README.md for usage instructions."
