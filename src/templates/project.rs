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
            args.insert("junitJupiter".into(), ArgInfo {
                description: "junitJupiter version, default to 5.8.1",
                default: Some("5.8.1")
            });
            args
        },
        files: || vec![
            TemplateFile {
                path: "gradle/wrapper/gradle-wrapper.properties",
                write_content: |args, w| write!(w, r"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-{}-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists", args["gradle"]),
            },
            TemplateFile {
                path: "buildSrc/build.gradle.kts",
                write_content: |_, w| write!(w, r"plugins {{
    `kotlin-dsl`
}}

repositories {{
    mavenCentral()
}}
"),
            },
            TemplateFile {
                path: "buildSrc/settings.gradle.kts",
                write_content: |_, w| write!(w, r#"rootProject.name = "buildSrc"
"#),
            },
            TemplateFile {
                path: "buildSrc/src/main/kotlin/Versions.kt",
                write_content: |_, w| write!(w, r#"object Versions {{
    const val junitJupiter = "5.8.1"
}}"#),
            },
            TemplateFile {
                path: "build.gradle.kts",
                write_content: |args, w| {
                    let group = &args["group"];
                    let version = &args["version"];
                    let jvm = (&args["jvm"])
                        .replace('.', "_");
                    write!(w, r#"plugins {{
    id("java")
}}

group = {group}
version = {version}

repositories {{
    mavenCentral()
}}

java {{
    sourceCompatibility = JavaVersion.VERSION_{jvm}
    targetCompatibility = JavaVersion.VERSION_{jvm}
}}

dependencies {{
    testImplementation("org.junit.jupiter:junit-jupiter-api:${{Versions.junitJupiter}}")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:${{Versions.junitJupiter}}")
}}

tasks.getByName<Test>("test") {{
    useJUnitPlatform()
}}"#)
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
                path: "src/test/java/TestExample.java",
                write_content: |_, w| write!(w, r"import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

public class TestExample {{
    @Test
    public void test() {{
        assertEquals(1, 1);
    }}
}}
"),
            },
            TemplateFile {
                path: "src/main/java/Main.java",
                write_content: |_, w| write!(w, r#"public class Main {{
    public static void main(String[] args) {{
        System.out.println("Hello World!");
    }}
}}
"#),
            },
        ],
    }
}
