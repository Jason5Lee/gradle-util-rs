use crate::templates::{ArgInfo, index_map_with_capacity, IndexMapString, Template, TemplateFile};

pub(super) fn create_template() -> Template {
    Template {
        args: {
            let mut args: IndexMapString<ArgInfo> = index_map_with_capacity(6);
            args.insert("gradle".into(), ArgInfo::with_description("Gradle wrapper version"));
            args.insert("group".into(), ArgInfo::with_description("Project group"));
            args.insert("name".into(), ArgInfo::with_description("Project name"));
            args.insert("version".into(), ArgInfo::with_description("Project version"));
            args.insert("jvm".into(), ArgInfo {
                description: "Target JVM version, default to 17",
                default: Some("17")
            });
            args.insert("kotlin".into(), ArgInfo {
                description: "Kotlin language version, default to 1.6.20",
                default: Some("1.6.20")
            });
            
            args
        },
        files: || vec![
            TemplateFile {
                path: "gradle/wrapper/gradle-wrapper.properties",
                write_content: |args, w| {
                    let gradle = &args["gradle"];
                    write!(w, r"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-{gradle}-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists")
                },
            },
            TemplateFile {
                path: "buildSrc/build.gradle.kts",
                write_content: |_, w| {
                    write!(w, r"plugins {{
    `kotlin-dsl`
}}

repositories {{
    mavenCentral()
}}
")
                },
            },
            TemplateFile {
                path: "buildSrc/settings.gradle.kts",
                write_content: |_, w| {
                    write!(w, r#"rootProject.name = "buildSrc"
"#)
                },
            },
            TemplateFile {
                path: "buildSrc/gradle.properties",
                write_content: |_, w| {
                    write!(w, r"kotlin.code.style=official
")
                },
            },
            TemplateFile {
                path: "buildSrc/src/main/kotlin/Versions.kt",
                write_content: |args, w| {
                    let kotlin = &args["kotlin"];
                    write!(w, r#"object Versions {{
    const val kotlin = "{kotlin}"
}}"#)
                },
            },
            TemplateFile {
                path: "build.gradle.kts",
                write_content: |args, w| {
                    let group = &args["group"];
                    let version = &args["version"];
                    
                    let jvm = &args["jvm"];
                    let jvm_java = jvm.replace(".", "_");
                    write!(w, r#"import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {{
    kotlin("jvm") version Versions.kotlin
}}

group = {group}
version = {version}

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
    sourceCompatibility = JavaVersion.VERSION_{jvm_java}
    targetCompatibility = JavaVersion.VERSION_{jvm_java}
}}
tasks.withType<KotlinCompile> {{
    kotlinOptions.jvmTarget = "{jvm}"
}}
"#)
                },
            },
            TemplateFile {
                path: "settings.gradle.kts",
                write_content: |args, w| {
                    let name = &args["name"];
                    write!(w, r#"rootProject.name = {name}

// https://twitter.com/Louis_CAD/status/1498270951175299080?s=20&t=uv0XxtYQzbktJTcpvnJ6Wg
try {{
    rootDir.resolve("gradle.properties").copyTo(
        target = rootDir.resolve("buildSrc/gradle.properties"),
        overwrite = true,
    )
}} catch (e:  NoSuchFileException) {{
    // ignore
}}
"#)
                },
            },
            TemplateFile {
                path: "gradle.properties",
                write_content: |_, w| {
                    write!(w, r"kotlin.code.style=official
")
                },
            },
            TemplateFile {
                path: "src/test/kotlin/SimpleTest.kt",
                write_content: |_, w| {
                    write!(w, r"import kotlin.test.*

class SimpleTest {{
    @Test
    fun test() {{
        assertEquals(1, 1)
    }}
}}")
                },
            },
            TemplateFile {
                path: "src/main/kotlin/main.kt",
                write_content: |_, w| {
                    write!(w, r#"fun main() {{
    println("Hello, world!")
}}"#)
                },
            },
        ],
    }
}
