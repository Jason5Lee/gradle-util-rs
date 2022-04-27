use crate::templates::{index_map_with_capacity, ArgInfo, IndexMapString, Template, TemplateFile};

pub(super) fn create_template() -> Template {
    Template {
        extra_args_info: {
            let mut info: IndexMapString<ArgInfo> = index_map_with_capacity(8);
            info.insert(
                "kotlin".into(),
                ArgInfo {
                    description: "Kotlin version, default to 1.6.20",
                    default: Some("1.6.20"),
                },
            );
            info.insert(
                "protobufPlugin".into(),
                ArgInfo {
                    description: "Protobuf Plugin (com.google.protobuf) version, default to 0.8.18",
                    default: Some("0.8.18"),
                },
            );
            info.insert(
                "shadow".into(),
                ArgInfo {
                    description:
                        "Shadow Plugin (com.github.johnrengelman.shadow) version, default to 7.0.0",
                    default: Some("7.0.0"),
                },
            );
            info.insert(
                "vertx".into(),
                ArgInfo {
                    description: "Vert.x version, default to 4.2.7",
                    default: Some("4.2.7"),
                },
            );
            info.insert(
                "junitJupiter".into(),
                ArgInfo {
                    description: "junitJupiter version, default to 5.7.0",
                    default: Some("5.7.0"),
                },
            );
            info.insert(
                "grpc".into(),
                ArgInfo {
                    description: "gRPC version, default to 1.44.0",
                    default: Some("1.44.0"),
                },
            );
            info.insert(
                "grpc-kotlin".into(),
                ArgInfo {
                    description: "gRPC Kotlin (io.grpc:grpc-kotlin-stub) version, default to 1.2.1",
                    default: Some("1.2.1"),
                },
            );
            info.insert(
                "protobuf".into(),
                ArgInfo {
                    description: "Protobuf version, default to 3.19.2",
                    default: Some("3.19.2"),
                },
            );
            info
        },
        files: || {
            vec![
                super::git_ignore::GIT_IGNORE,
                TemplateFile {
                    path: |_| "protos/build.gradle.kts".into(),
                    write_content: |_, w| {
                        write!(
                            w,
                            r#"plugins {{
    `java-library`
}}

java {{
    sourceSets.getByName("main").resources.srcDir("src/main/proto")
}}
"#
                        )
                    },
                },
                TemplateFile {
                    path: |args| {
                        format!("protos/src/main/proto/{}/greeting.proto", args.package_path).into()
                    },
                    write_content: |args, w| {
                        let package = &args.package;
                        write!(
                            w,
                            r#"syntax = "proto3";

option java_multiple_files = true;
option java_package = "{package}";
option java_outer_classname = "GreetingProto";

package greeting;

service GreetingService {{
    rpc Greet(GreetRequest) returns (GreetResponse) {{}}
}}

message GreetRequest {{
    string name = 1;
}}

message GreetResponse {{
  string message = 1;
}}
"#
                        )
                    },
                },
                TemplateFile {
                    path: |_| "server/build.gradle.kts".into(),
                    write_content: |args, w| {
                        let package = &args.package;
                        let jvm = &args.jvm;
                        let java_jvm = &args.java_jvm;

                        write!(
                            w,
                            r#"import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar
import org.gradle.api.tasks.testing.logging.TestLogEvent.*
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {{
    kotlin("jvm")
    application
    id("com.github.johnrengelman.shadow")
}}

repositories {{
    mavenCentral()
}}

val mainVerticleName = "{package}.MainVerticle"
val launcherClassName = "io.vertx.core.Launcher"

val watchForChange = "src/**/*"
val doOnChange = "${{projectDir}}/gradlew classes"

application {{
    mainClass.set(launcherClassName)
}}

dependencies {{
    implementation(project(":stub"))
    implementation(platform("io.vertx:vertx-stack-depchain:${{Versions.vertx}}"))
    implementation("io.vertx:vertx-grpc")
    implementation("io.vertx:vertx-lang-kotlin-coroutines")
    testImplementation("io.vertx:vertx-junit5")
    testImplementation("org.junit.jupiter:junit-jupiter:${{Versions.junitJupiter}}")
}}

val compileKotlin: KotlinCompile by tasks
compileKotlin.kotlinOptions.jvmTarget = "{jvm}"

java {{
    sourceCompatibility = JavaVersion.VERSION_{java_jvm}
    targetCompatibility = JavaVersion.VERSION_{java_jvm}
}}

tasks.withType<ShadowJar> {{
    archiveClassifier.set("fat")
    manifest {{
        attributes(mapOf("Main-Verticle" to mainVerticleName))
    }}
    mergeServiceFiles()
}}

tasks.withType<Test> {{
    useJUnitPlatform()
    testLogging {{
        events = setOf(PASSED, SKIPPED, FAILED)
    }}
}}

tasks.withType<JavaExec> {{
    args = listOf(
        "run",
        mainVerticleName,
        "--redeploy=$watchForChange",
        "--launcher-class=$launcherClassName",
        "--on-redeploy=$doOnChange"
    )
}}
"#
                        )
                    },
                },
                TemplateFile {
                    path: |args| {
                        format!(
                            "server/src/test/kotlin/{}/TestMainVerticle.kt",
                            args.package_path
                        )
                        .into()
                    },
                    write_content: |args, w| {
                        let package = &args.package;
                        write!(
                            w,
                            r"package {package}

import {package}.MainVerticle
import io.vertx.core.Vertx
import io.vertx.junit5.VertxExtension
import io.vertx.junit5.VertxTestContext
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith

@ExtendWith(VertxExtension::class)
class TestMainVerticle {{

    @BeforeEach
    fun deployVerticle(vertx: Vertx, testContext: VertxTestContext) {{
        vertx.deployVerticle(MainVerticle(), testContext.succeeding<String> {{ _ -> testContext.completeNow() }})
    }}

    @Test
    fun verticleDeployed(vertx: Vertx, testContext: VertxTestContext) {{
        testContext.completeNow()
    }}
}}
"
                        )
                    },
                },
                TemplateFile {
                    path: |args| {
                        format!(
                            "server/src/main/kotlin/{}/GreetingServiceImpl.kt",
                            args.package_path
                        )
                        .into()
                    },
                    write_content: |args, w| {
                        let package = &args.package;
                        write!(
                            w,
                            r#"package {package}

class GreetingServiceImpl : GreetingServiceGrpcKt.GreetingServiceCoroutineImplBase() {{
    override suspend fun greet(request: GreetRequest): GreetResponse =
        GreetResponse.newBuilder()
            .setMessage("Hello, ${{request.name}}")
            .build()
}}"#
                        )
                    },
                },
                TemplateFile {
                    path: |args| {
                        format!(
                            "server/src/main/kotlin/{}/MainVerticle.kt",
                            args.package_path
                        )
                        .into()
                    },
                    write_content: |args, w| {
                        let package = &args.package;
                        write!(
                            w,
                            r#"package {package}

import io.vertx.core.AbstractVerticle
import io.vertx.core.Promise
import io.vertx.grpc.VertxServerBuilder

class MainVerticle : AbstractVerticle() {{

    override fun start(startPromise: Promise<Void>) {{
        val rpcServer = VertxServerBuilder
            .forAddress(vertx, "localhost", 8080)
            .addService(GreetingServiceImpl())
            .build()
        rpcServer.start(startPromise)
    }}
}}
"#
                        )
                    },
                },
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
zipStorePath=wrapper/dists
"
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
                        let protobuf_plugin = &args.extras["protobufPlugin"];
                        let shadow = &args.extras["shadow"];
                        let vertx = &args.extras["vertx"];
                        let junit_jupiter = &args.extras["junitJupiter"];
                        let grpc = &args.extras["grpc"];
                        let grpc_kotlin = &args.extras["grpc-kotlin"];
                        let protobuf = &args.extras["protobuf"];

                        write!(
                            w,
                            r#"object Versions {{
    const val kotlin = "{kotlin}"
    const val protobufPlugin = "{protobuf_plugin}"
    const val shadow = "{shadow}"
    const val vertx = "{vertx}"
    const val junitJupiter = "{junit_jupiter}"
    const val grpc = "{grpc}"
    const val grpcKotlin = "{grpc_kotlin}"
    const val protobuf = "{protobuf}"
}}"#
                        )
                    },
                },
                TemplateFile {
                    path: |_| "build.gradle.kts".into(),
                    write_content: |_, w| {
                        write!(
                            w,
                            r#"plugins {{
    id("com.google.protobuf") version Versions.protobufPlugin apply false
    kotlin("jvm") version Versions.kotlin apply false
    id("com.github.johnrengelman.shadow") version Versions.shadow apply false
}}

allprojects {{
    repositories {{
        mavenLocal()
        mavenCentral()
        google()
    }}
}}
"#
                        )
                    },
                },
                TemplateFile {
                    path: |_| "stub/build.gradle.kts".into(),
                    write_content: |args, w| {
                        let jvm = &args.jvm;
                        let java_jvm = &args.java_jvm;

                        write!(
                            w,
                            r#"import com.google.protobuf.gradle.generateProtoTasks
import com.google.protobuf.gradle.id
import com.google.protobuf.gradle.plugins
import com.google.protobuf.gradle.protobuf
import com.google.protobuf.gradle.protoc

plugins {{
    kotlin("jvm")
    id("com.google.protobuf")
}}

dependencies {{
    protobuf(project(":protos"))

    api("io.grpc:grpc-stub:${{Versions.grpc}}")
    api("io.grpc:grpc-protobuf:${{Versions.grpc}}")
    api("io.grpc:grpc-kotlin-stub:${{Versions.grpcKotlin}}")
    api("com.google.protobuf:protobuf-kotlin:${{Versions.protobuf}}")
    api("com.google.protobuf:protobuf-java-util:${{Versions.protobuf}}")
}}

// Makes IntelliJ IDEA happy.
sourceSets {{
    val main by getting {{ }}
    main.java.srcDirs("build/generated/source/proto/main/java")
    main.java.srcDirs("build/generated/source/proto/main/grpc")
    main.java.srcDirs("build/generated/source/proto/main/kotlin")
    main.java.srcDirs("build/generated/source/proto/main/grpckt")
}}

java {{
    sourceCompatibility = JavaVersion.VERSION_{java_jvm}
    targetCompatibility = JavaVersion.VERSION_{java_jvm}
}}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().all {{
    kotlinOptions {{
        freeCompilerArgs = listOf("-Xopt-in=kotlin.RequiresOptIn")
        jvmTarget = "{jvm}"
    }}
}}

protobuf {{
    protoc {{
        artifact = "com.google.protobuf:protoc:${{Versions.protobuf}}"
    }}
    plugins {{
        id("grpc") {{
            artifact = "io.grpc:protoc-gen-grpc-java:${{Versions.grpc}}"
        }}
        id("grpckt") {{
            artifact = "io.grpc:protoc-gen-grpc-kotlin:${{Versions.grpcKotlin}}:jdk7@jar"
        }}
    }}
    generateProtoTasks {{
        all().forEach {{
            it.builtins {{
                id("kotlin")
            }}
            it.plugins {{
                id("grpc")
                id("grpckt")
            }}
        }}
    }}
}}"#
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

pluginManagement {{
    repositories {{
        gradlePluginPortal()
        google()
    }}
}}

include("stub", "protos", "client", "server")

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
                    write_content: |_, w| write!(w, r"kotlin.code.style=official"),
                },
                TemplateFile {
                    path: |_| "client/build.gradle.kts".into(),
                    write_content: |args, w| {
                        let package = &args.package;
                        let jvm = &args.jvm;
                        let java_jvm = &args.java_jvm;

                        write!(
                            w,
                            r#"import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar
import org.gradle.api.tasks.testing.logging.TestLogEvent.*
import org.jetbrains.kotlin.gradle.tasks.KotlinCompile

plugins {{
    kotlin("jvm")
    application
    id("com.github.johnrengelman.shadow")
}}

repositories {{
    mavenCentral()
}}

val mainVerticleName = "{package}.MainVerticle" // TODO: point to your vericle path
val launcherClassName = "io.vertx.core.Launcher"

val watchForChange = "src/**/*"
val doOnChange = "$projectDir/gradlew classes"

application {{
    mainClass.set(launcherClassName)
}}

dependencies {{
    implementation(platform("io.vertx:vertx-stack-depchain:${{Versions.vertx}}"))
    implementation("io.vertx:vertx-grpc")
    implementation("io.vertx:vertx-lang-kotlin-coroutines")
    implementation(project(":stub"))
    testImplementation("io.vertx:vertx-junit5")
    testImplementation("org.junit.jupiter:junit-jupiter:${{Versions.junitJupiter}}")
}}

val compileKotlin: KotlinCompile by tasks
compileKotlin.kotlinOptions.jvmTarget = "{jvm}"

java {{
    sourceCompatibility = JavaVersion.VERSION_{java_jvm}
    targetCompatibility = JavaVersion.VERSION_{java_jvm}
}}

tasks.withType<ShadowJar> {{
    archiveClassifier.set("fat")
    manifest {{
        attributes(mapOf("Main-Verticle" to mainVerticleName))
    }}
    mergeServiceFiles()
}}

tasks.withType<Test> {{
    useJUnitPlatform()
    testLogging {{
        events = setOf(PASSED, SKIPPED, FAILED)
    }}
}}

tasks.withType<JavaExec> {{
    args = listOf(
        "run",
        mainVerticleName,
        "--redeploy=$watchForChange",
        "--launcher-class=$launcherClassName",
        "--on-redeploy=$doOnChange"
    )
}}
"#
                        )
                    },
                },
                TemplateFile {
                    path: |args| {
                        format!(
                            "client/src/test/kotlin/{}/TestMainVerticle.kt",
                            args.package_path
                        )
                        .into()
                    },
                    write_content: |args, w| {
                        let package = &args.package;

                        write!(
                            w,
                            r"package {package}

import io.vertx.core.Vertx
import io.vertx.junit5.VertxExtension
import io.vertx.junit5.VertxTestContext
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith
import {package}.MainVerticle

@ExtendWith(VertxExtension::class)
class TestMainVerticle {{

    @BeforeEach
    fun deployVerticle(vertx: Vertx, testContext: VertxTestContext) {{
        vertx.deployVerticle(MainVerticle(), testContext.succeeding<String> {{ _ -> testContext.completeNow() }})
    }}

    @Test
    fun verticleDeployed(vertx: Vertx, testContext: VertxTestContext) {{
        testContext.completeNow()
    }}
}}
"
                        )
                    },
                },
                TemplateFile {
                    path: |args| {
                        format!(
                            "client/src/main/kotlin/{}/MainVerticle.kt",
                            args.package_path
                        )
                        .into()
                    },
                    write_content: |args, w| {
                        let package = &args.package;

                        write!(
                            w,
                            r#"package {package}

import io.vertx.grpc.VertxChannelBuilder
import io.vertx.kotlin.coroutines.CoroutineVerticle

class MainVerticle : CoroutineVerticle() {{
    override suspend fun start() {{
        val channel = VertxChannelBuilder
            .forAddress(vertx, "localhost", 8080)
            .usePlaintext()
            .build()

        val stub = GreetingServiceGrpcKt.GreetingServiceCoroutineStub(channel)
        val name = "World"

        val resp = stub.greet(GreetRequest.newBuilder().setName(name).build())
        println(resp.message)
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
