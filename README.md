# gradle-util-rs

Gradle utility written in Rust.

Note that this project is still in the alpha stage. The functionalities and behaviors may change.

## Install

You can find the pre-built binaries at the [release page](https://github.com/jason5lee/gradle-util-rs/releases). You can build and install it via `cargo install gradle-util-rs --version 0.1.0-alpha.3`.

## Usage

```
USAGE:
    gur <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    chver      Change the gradle wrapper version
    help       Print this message or the help of the given subcommand(s)
    set-new    Watch for the new Gradle project and set the gradle version
```

### `set-new`

This is made as a workaround of [IDEA-177325](https://youtrack.jetbrains.com/issue/IDEA-177325). It watches for the creation of the file named `gradle.properties`, and it creates the gradle wrapper properties using a certain gradle version at the corresponding path. In this way, once IntelliJ IDEA creates a new project, the gradle version will be set to the version you want.

Example:

`gur set-new 7.3.3 path1 path2` : watch the gradle project creation under `path1` and `path2` recursively, and create the gradle wrapper properties for the new projects using gradle version `7.3.3`.

### `chver`

This subcommand is to update the gradle wrapper version of the current project.
It essentially calls the `./gradlew wrapper --gradle-version <version>` .

When the `yolo` flag is enabled, it will first update the content of `gradle-wrapper.properties` to using the new version,
then run the wrapper task. In this way, the gradle distribution of the old version won't be downloaded. But it may have potential problems.

Example: `gur chver 7.3.3 --yolo`

## Why Rust?

You might be surprised that a Gradle utility is written in Rust instead of Java or other JVM languages.
The major reason is that Gradle already requires a java instance to run. I don't want yet another java process.
Instead, just keep it as light as possible.

The reason I choose Rust is that its mental model is surprisingly closed to Kotlin, the
major language I used with Gradle. Many building blocks in Kotlin like data class, sealed class and nullable type have their
corresponding in Rust like struct, enum and option.

