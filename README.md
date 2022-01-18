# gradle-util-rs

Gradle utility written in Rust.

Note that this project is still at the alpha stage. The functionalities and behaviors may change.

## Install

You can find the pre-built binaries at the [release page](https://github.com/jason5lee/gradle-util-rs/releases). You can build and install it via `cargo install gradle-util-rs --version 0.1.0-alpha.1`.

## Usage

```
USAGE:
    gur <SUBCOMMAND>

SUBCOMMANDS:
    set-new    Watch for the new Gradle project and set the gradle version
```

### `set-new`

This is made as a workaround of [IDEA-177325](https://youtrack.jetbrains.com/issue/IDEA-177325). It watches for the creation of the file named `gradle.properties`, and it creates the gradle wrapper properties using a certain gradle version at the corresponding path. In this way, once IntelliJ IDEA creates a new project, the gradle version will be set to the version you want.

Example:

`gur set-new --watch-dir path1 --watch-dir path2 --gradle-version 7.3` : watch the gradle project creation under `path1` and `path2` recursively, and create the gradle wrapper properties for the new projects using gradle version `7.3`.
