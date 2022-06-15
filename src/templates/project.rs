use crate::templates::{index_map_with_capacity, ArgInfo, IndexMapString, Template, TemplateFile};

pub(super) fn create_template() -> Template {
    Template {
        extra_args_info: {
            let mut info: IndexMapString<ArgInfo> = index_map_with_capacity(1);
            info.insert(
                "junitJupiter".into(),
                ArgInfo {
                    description: "junitJupiter version, default to 5.8.1",
                    default: Some("5.8.1"),
                },
            );
            info
        },
        files: || {
            vec![
                super::git_ignore::GIT_IGNORE,
                super::gitattributes::GITATTRIBUTES,
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
                        let junit_jupiter = &args.extras["junitJupiter"];
                        write!(
                            w,
                            r#"object Versions {{
    const val junitJupiter = "{junit_jupiter}"
}}"#
                        )
                    },
                },
                TemplateFile {
                    path: |_| "build.gradle.kts".into(),
                    write_content: |args, w| {
                        let group = &args.group;
                        let version = &args.version;
                        let java_jvm = &args.java_jvm;

                        write!(
                            w,
                            r#"plugins {{
    id("java")
}}

group = "{group}"
version = "{version}"

repositories {{
    mavenCentral()
}}

java {{
    sourceCompatibility = JavaVersion.VERSION_{java_jvm}
    targetCompatibility = JavaVersion.VERSION_{java_jvm}
}}

dependencies {{
    testImplementation("org.junit.jupiter:junit-jupiter-api:${{Versions.junitJupiter}}")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:${{Versions.junitJupiter}}")
}}

tasks.getByName<Test>("test") {{
    useJUnitPlatform()
}}"#
                        )
                    },
                },
                TemplateFile {
                    path: |_| "gradle.properties".into(),
                    write_content: |_, _| Ok(()),
                },
                TemplateFile {
                    path: |_| "settings.gradle.kts".into(),
                    write_content: |args, w| {
                        let name = &args.artifact;

                        write!(
                            w,
                            r#"rootProject.name = "{name}"

// https://twitter.com/Louis_CAD/status/1498270951175299080?s=20&t=uv0XxtYQzbktJTcpvnJ6Wg
rootDir.resolve("gradle.properties").copyTo(
    target = rootDir.resolve("buildSrc/gradle.properties"),
    overwrite = true,
)
"#
                        )
                    },
                },
                TemplateFile {
                    path: |args| {
                        format!("src/test/java/{}/TestExample.java", args.package_path).into()
                    },
                    write_content: |args, w| {
                        let package = &args.package;
                        write!(
                            w,
                            r"package {package};
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.*;

public class TestExample {{
    @Test
    public void test() {{
        assertEquals(1, 1);
    }}
}}
"
                        )
                    },
                },
                TemplateFile {
                    path: |args| format!("src/main/java/{}/Main.java", args.package_path).into(),
                    write_content: |args, w| {
                        let package = &args.package;
                        write!(
                            w,
                            r#"package {package};

public class Main {{
    public static void main(String[] args) {{
        System.out.println("Hello World!");
    }}
}}
"#
                        )
                    },
                },
            ]
        },
    }
}
