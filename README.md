# git-reorder
A visual helper for interactive rebase.

![Screenshot](Screenshot.png)

## Installation

Requires Python 3.

```sh
./create-alias.sh
```

This will create the **global** alias `reorder` for Git.

## Usage

```sh
git reorder <options>
```

Since _git reorder_ is implemented with interactive mode of _git rebase_, most of
_git rebase_ options are available. Exceptions are options used after an interactive
rebase is started (`--continue`, `--skip`, etc.) and options incompatible with
interactive mode of _git rebase_ (`--ignore-date`, `--ignore-whitespace`, etc.).

See [git-rebase](https://git-scm.com/docs/git-rebase).
