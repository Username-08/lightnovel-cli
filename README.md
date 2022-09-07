<h1 align="center">LightNovel-CLI</h1>

A simple program to read lightnovels in your terminal. It scrapes [freewebnovel](https://freewebnovel.com)


https://user-images.githubusercontent.com/83852973/159120682-1d0bfcd3-d42f-4d4a-893a-122e4c9159e5.mp4




# Table Of Contents :toc:

- [Keybinds](#keybinds)
- [Installation](#installation)
  - [Arch Linux](#arch-linux)
  - [Linux](#linux)
- [Uninstalling](#uninstalling)

# Keybinds

| Bind             | Action                  |
| ---------------- | ----------------------- |
| s                | search for a novel      |
| j or down_arrow  | scroll down             |
| d                | scroll down half a page |
| k or up_arrow    | scroll up               |
| u                | scroll up half a page   |
| q                | quit                    |
| h or left_arrow  | go to previous chapter  |
| l or right_arrow | go to next chapter      |
| enter            | select option under cursor |

# Installation

## Arch Linux

```sh
yay -S lightnovel-cli-git

```

## Linux

### Dependencies

Make sure you have these installed before compiling from source

```sh
rust
ncurses
openssl
```

### Compiling

```sh
git clone https://github.com/Username-08/lightnovel-cli.git
cd lightnovel-cli
cargo build --release
```

# Uninstalling

```sh
yay -Rns lightnovel-cli-git
```
