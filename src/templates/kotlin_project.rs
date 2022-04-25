use crate::templates::{index_map_with_capacity, ArgInfo, IndexMapString, Template, TemplateFile};

pub(super) fn create_template() -> Template {
    Template {
        extra_args_info: {
            let mut info: IndexMapString<ArgInfo> = index_map_with_capacity(1);
            info.insert(
                "kotlin".into(),
                ArgInfo {
                    description: "kotlin version, default to 1.6.20",
                    default: Some("1.6.20"),
                },
            );
            info
        },
        files: || {
            vec![
                TemplateFile {
                    path: |_| "gradle/wrapper/gradle-wrapper.properties".into(),
                    write_content: |args, w| {
                        let gradle = &args.gradle;
                        write!(
                            w,
                            r"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-{gradle}-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists"
                        )
                    },
                },
                TemplateFile {
                    path: |_| "buildSrc/build.gradle.kts".into(),
                    write_content: |_, w| {
                        write!(
                            w,
                            r"plugins {{
    `kotlin-dsl`
}}

repositories {{
    mavenCentral()
}}
"
                        )
                    },
                },
                TemplateFile {
                    path: |_| "buildSrc/settings.gradle.kts".into(),
                    write_content: |_, w| {
                        write!(
                            w,
                            r#"rootProject.name = "buildSrc"
"#
                        )
                    },
                },
                TemplateFile {
                    path: |_| "buildSrc/src/main/kotlin/Versions.kt".into(),
                    write_content: |args, w| {
                        let kotlin = &args.extras["kotlin"];
                        write!(
                            w,
                            r#"object Versions {{
    const val kotlin = "{kotlin}"
}}"#
                        )
                    },
                },
                TemplateFile {
                    path: |_| "build.gradle.kts".into(),
                    write_content: |args, w| {
                        let group = &args.group;
                        let version = &args.version;
                        let jvm = &args.jvm;
                        let java_jvm = &args.java_jvm;

                        write!(
                            w,
                            r#"import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {{
    kotlin("jvm") version Versions.kotlin
}}

group = "{group}"
version = "{version}"

repositories {{
    mavenCentral()
}}

dependencies {{
    testImplementation(kotlin("test"))
}}

tasks.test {{
    useJUnitPlatform()
}}

java {{
    sourceCompatibility = JavaVersion.VERSION_{java_jvm}
    targetCompatibility = JavaVersion.VERSION_{java_jvm}
}}
tasks.withType<KotlinCompile> {{
    kotlinOptions.jvmTarget = "{jvm}"
}}
"#
                        )
                    },
                },
                TemplateFile {
                    path: |_| "settings.gradle.kts".into(),
                    write_content: |args, w| {
                        let name = &args.name;

                        write!(
                            w,
                            r#"rootProject.name = "{name}"

// https://twitter.com/Louis_CAD/status/1498270951175299080?s=20&t=uv0XxtYQzbktJTcpvnJ6Wg
try {{
    rootDir.resolve("gradle.properties").copyTo(
        target = rootDir.resolve("buildSrc/gradle.properties"),
        overwrite = true,
    )
}} catch (e:  NoSuchFileException) {{
    // ignore
}}
"#
                        )
                    },
                },
                TemplateFile {
                    path: |_| "gradle.properties".into(),
                    write_content: |_, w| {
                        write!(
                            w,
                            r"kotlin.code.style=official
"
                        )
                    },
                },
                TemplateFile {
                    path: |args| {
                        format!("src/test/kotlin/{}/SimpleTest.kt", args.package_path).into()
                    },
                    write_content: |args, w| {
                        let package = &args.package;
                        write!(
                            w,
                            r"package {package}

import kotlin.test.*

class SimpleTest {{
    @Test
    fun test() {{
        assertEquals(1, 1)
    }}
}}"
                        )
                    },
                },
                TemplateFile {
                    path: |args| format!("src/main/kotlin/{}/main.kt", args.package_path).into(),
                    write_content: |args, w| {
                        let package = &args.package;
                        write!(
                            w,
                            r#"package {package}

fun main() {{
    println("Hello, world!")
}}"#
                        )
                    },
                },
            ]
        },
    }
}
