# gradle-util-rs

Gradle utility written in Rust.

Note that this project is still at the alpha stage. The functionalities and behaviors may change.

## Subcommands

### `set-new`

This is made as a workaround of [IDEA-177325](https://youtrack.jetbrains.com/issue/IDEA-177325). It watches for the creation of the file named `gradle.properties`, and it creates the gradle wrapper properties using a certain gradle version at the corresponding path. In this way, once IntelliJ IDEA creates a new project, the gradle version will be set to the version you want.

Example:

`gur set-new --watch-dir path1 --watch-dir path2 --gradle-version 7.3` : watch the gradle project creation under `path1` and `path2` recursively, and create the gradle wrapper properties for the new projects using gradle version `7.3`.
