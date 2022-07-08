# gradle-util-rs

Gradle utility written in Rust.

Note that this project is still in the alpha stage. The functionalities and behaviors may change.

## Install

You can find the pre-built binaries at the [release page](https://github.com/jason5lee/gradle-util-rs/releases). You can build and install it via `cargo install gradle-util-rs --version 0.1.0-alpha.6`.

## Usage

```
USAGE:
    gur <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    chver       Change the gradle wrapper version
    help        Print this message or the help of the given subcommand(s)
    set-new     Watch for the new Gradle project and set the gradle version
    template    Create project from the template
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

### `template`

This subcommand allows you to create a gradle project from a template.

The generated project has the following features:

* Using buildSrc as the central dependency version definition.
* Allow you to customize the Gradle version in the wrapper properties.

Example:

* List all the templates: `gur template list`
* List the information of a specific template: `gur template list <template name>`
* Create a project from the template: `gur template create <template-name> -o <project path> -D<arg>=<value>`

The templates are stored as toml files. It will try to find these files from
* `templates` directory within the binary directory
* `$HOME/.gur/templates`
* All the paths in `GUR_TEMPLATES_PATH`, seperated by `:`

When generating a templates, it first copies all files under `shared` directory to the target directory,
then it generates file by the config file.

An example of the template file:

```toml
[args]
wrapper = { description = "Gradle wrapper version", default = "7.3.3" }
targetJvm = { default = "17" }
kotlin = { description = "Kotlin version", default = "1.6.20" }

[[files]]
path = "gradle/wrapper/gradle-wrapper.properties"
content = """
...
distributionUrl=https\\://services.gradle.org/distributions/gradle-$(wrapper)-bin.zip
...
"""

[[files]]
path = "buildSrc/src/main/kotlin/Versions.kt"
content = """object Versions {
    const val kotlin = "$(kotlin)"
}"""

[[files]]
path = "build.gradle.kts"
content = """
...

group = "$(group)"
version = "$(version)"

...

java {
    sourceCompatibility = JavaVersion.VERSION_$(targetJvmJava)
    targetCompatibility = JavaVersion.VERSION_$(targetJvmJava)
}
tasks.withType<KotlinCompile> {
    kotlinOptions.jvmTarget = "$(targetJvm)"
}
"""

[[files]]
path = "settings.gradle.kts"
content = """rootProject.name = "$(artifact)"

...
"""

[[files]]
path = "src/test/kotlin/$(packagePath)/SimpleTest.kt"
content = """package $(package)
...
"""

[[files]]
path = "src/main/kotlin/$(packagePath)/main.kt"
content = """package $(package)

fun main() {
    println("Hello, world!")
}"""

```

In the `[[files]]` section, we define the generated file by path and the content.
We can use `$(<arg name>)` to use the argument value.

There are four built-in arguments: `group`, `artifact`, `version`, `package`.
`package` has a default value of `<group>.<artifact>` if it is a valid package name or
it can be fixed via the [package naming guideline](https://docs.oracle.com/javase/tutorial/java/package/namingpkgs.html).

You can define other arguments at the `[args]` section.

The argument `targetJvm` is special. You don't need to config the description.
If it exists, another argument `targetJvmJava` can be used in the template files,
which has `.` replaced by `_` from the `targetJvm` value.
It's useful for setting Java's `sourceCompatibility` and `targetCompatibility` options.

You can use `packagePath` in the template, which is the path of the `package`.
E.g. if the `package` is `com.example.gradle`, the `packagePath` is `com/example/gradle`.

You can use `$(dollar)` for the `$` character. It's useful for escaping the `$` character.

## Why Rust?

You might be surprised that a Gradle utility is written in Rust instead of Java or other JVM languages.
The major reason is that Gradle already requires a java instance to run. I don't want yet another java process.
Instead, just keep it as light as possible.

The reason I choose Rust is that its mental model is surprisingly closed to Kotlin, the
major language I used with Gradle. Many building blocks in Kotlin like data class, sealed class and nullable type have their
corresponding in Rust like struct, enum and option.

