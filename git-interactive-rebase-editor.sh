#!/bin/bash
exec "$(dirname $0)/git-interactive-rebase-editor" "$(cygpath -w "$1")"
