# git-visual-rebase
A visual helper for Git rebase interactive mode.

![Screenshot](screenshot.png)

## Installing and building
1. Clone this repository
2. Build it with `cargo build --release`
3. Copy `./target/release/git-visual-rebase` to your desired location.

Alternativelly, use `cargo` to build and install at once:

```sh
cargo install --path <LOCAL_REPOSITORY_PATH>
```

or:

```sh
cargo install --git https://github.com/erdavila/git-visual-rebase.git
```

See the options for [`cargo install`](https://doc.rust-lang.org/cargo/commands/cargo-install.html).

## Using it

### Single use
Use the `sequence.editor` configuration when rebasing:

```sh
git -c sequence.editor=<PATH_TO_BINARY> rebase -i ...
```

### With an alias

Define an alias:
```sh
git config --global alias.vrebase "-c sequence.editor=<PATH_TO_BINARY> rebase -i"
```

then rebase with:

```sh
git vrebase ...
```

### Permanent use

Set the editor:

```sh
git config --global sequence.editor=<PATH_TO_BINARY>
```

then rebase with:

```sh
git rebase -i ...
```

### Platform Specifics

#### Git Bash
Start the `<PATH_TO_BINARY>` without `/` and with drive name.

E.g.: use `C:/...` instead of `/c/...`.

#### Cygwin
Cygwin requires the [`cygwin.sh`](./cygwin.sh) adapter script to ensure that the editor receives a path in Windows format.

In the commands above, instead of `sequence.editor=<PATH_TO_BINARY>`, use `sequence.editor='<PATH_TO_cygwin.sh> <PATH_TO_BINARY>'`.
