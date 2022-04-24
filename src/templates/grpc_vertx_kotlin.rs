use std::borrow::Cow;
use crate::templates::{ArgInfo, index_map_with_capacity, IndexMapString, Template, TemplateFile};

pub(super) fn create_template() -> Template {
    Template {
        args: {
            let mut args: IndexMapString<ArgInfo> = index_map_with_capacity(5);
            args.insert("group".into(), ArgInfo::with_description("Project group"));
            args.insert("name".into(), ArgInfo::with_description("Project name"));
            args.insert("package".into(), ArgInfo {
                description: "Package name, default to <group>.<name>".into(),
                default: Some("".into()),
            });
            args.insert("jvm".into(), ArgInfo {
                description: "Target JVM version, default to 17",
                default: Some("17")
            });
            args
        },
        files: || vec![
            TemplateFile {
                path: "protos/build.gradle.kts",
                write_content: |_, w| {
                    write!(w, r#"plugins {{
    `java-library`
}}

java {{
    sourceSets.getByName("main").resources.srcDir("src/main/proto")
}}
"#)
                },
            },
            TemplateFile {
                path: "protos/src/main/proto/greeting.proto",
                write_content: |args, w| {
                    let pkg = get_package(args);
                    write!(w, r#"syntax = "proto3";

option java_multiple_files = true;
option java_package = "{pkg}";
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
"#)
                },
            },
            TemplateFile {
                path: "server/build.gradle.kts",
                write_content: |args, w| {
                    let jvm = &args["jvm"];
                    let jvm_java = jvm.replace('.', "_");
                    let pkg = get_package(args);
                    write!(w, r#"import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar
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

val mainVerticleName = "{pkg}.MainVerticle"
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
    sourceCompatibility = JavaVersion.VERSION_{jvm_java}
    targetCompatibility = JavaVersion.VERSION_{jvm_java}
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
"#)
                },
            },
            TemplateFile {
                path: "server/src/test/kotlin/the/pkg/TestMainVerticle.kt",
                write_content: |_, w| {
                    write!(w, r"package the.pkg

import the.pkg.MainVerticle
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
")
                },
            },
            TemplateFile {
                path: "server/src/main/kotlin/the/pkg/GreetingServiceImpl.kt",
                write_content: |_, w| {
                    write!(w, r#"package the.pkg

class GreetingServiceImpl : GreetingServiceGrpcKt.GreetingServiceCoroutineImplBase() {{
    override suspend fun greet(request: GreetRequest): GreetResponse =
        GreetResponse.newBuilder()
            .setMessage("Hello, ${{request.name}}")
            .build()
}}"#)
                },
            },
            TemplateFile {
                path: "server/src/main/kotlin/the/pkg/MainVerticle.kt",
                write_content: |_, w| {
                    write!(w, r#"package the.pkg

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
"#)
                },
            },
            TemplateFile {
                path: "gradle/wrapper/gradle-wrapper.properties",
                write_content: |_, w| {
                    write!(w, r"distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-7.3.3-bin.zip
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
")
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
                    write!(w, r"kotlin.code.style=official")
                },
            },
            TemplateFile {
                path: "buildSrc/src/main/kotlin/Versions.kt",
                write_content: |_, w| {
                    write!(w, r#"object Versions {{
    const val kotlin = "1.6.20"
    // Here it uses a different version of stdlib to match the Vert.x version.
    const val kotlinStdlib = "1.5.31"
    const val versionsPlugin = "0.42.0"
    const val protobufPlugin = "0.8.18"
    const val shadow = "7.0.0"
    const val vertx = "4.2.7"
    const val junitJupiter = "5.7.0"
    const val kotlinxCoroutines = "1.5.2"
    const val grpc = "1.44.0"
    const val grpcKotlin = "1.2.1"
    const val protobuf = "3.19.2"
}}"#)
                },
            },
            TemplateFile {
                path: "build.gradle.kts",
                write_content: |_, w| {
                    write!(w, r#"plugins {{
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
"#)
                },
            },
            TemplateFile {
                path: "stub/build.gradle.kts",
                write_content: |_, w| {
                    write!(w, r#"import com.google.protobuf.gradle.generateProtoTasks
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

//    api("org.jetbrains.kotlinx:kotlinx-coroutines-core:${{Versions.kotlinxCoroutines}}")

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

// TODO: set JVM target version
java {{
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
}}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile>().all {{
    kotlinOptions {{
        freeCompilerArgs = listOf("-Xopt-in=kotlin.RequiresOptIn")
        jvmTarget = "11"
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
}}"#)
                },
            },
            TemplateFile {
                path: "settings.gradle.kts",
                write_content: |_, w| {
                    write!(w, r#"rootProject.name = "transcode-limit"

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
"#)
                },
            },
            TemplateFile {
                path: "gradle.properties",
                write_content: |_, w| {
                    write!(w, r"kotlin.code.style=official")
                },
            },
            TemplateFile {
                path: "client/build.gradle.kts",
                write_content: |_, w| {
                    write!(w, r#"import com.github.jengelman.gradle.plugins.shadow.tasks.ShadowJar
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

val mainVerticleName = "the.pkg.MainVerticle" // TODO: point to your vericle path
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

// TODO: set target kotlin version
val compileKotlin: KotlinCompile by tasks
compileKotlin.kotlinOptions.jvmTarget = "11"

java {{
    sourceCompatibility = JavaVersion.VERSION_11
    targetCompatibility = JavaVersion.VERSION_11
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
"#)
                },
            },
            TemplateFile {
                path: "client/src/test/kotlin/the/pkg/TestMainVerticle.kt",
                write_content: |_, w| {
                    write!(w, r"package the.pkg

import io.vertx.core.Vertx
import io.vertx.junit5.VertxExtension
import io.vertx.junit5.VertxTestContext
import org.junit.jupiter.api.BeforeEach
import org.junit.jupiter.api.Test
import org.junit.jupiter.api.extension.ExtendWith
import the.pkg.MainVerticle

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
")
                },
            },
            TemplateFile {
                path: "client/src/main/kotlin/the/pkg/MainVerticle.kt",
                write_content: |_, w| {
                    write!(w, r#"package the.pkg

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
"#)
                },
            },
        ],
    }
}

fn get_package(args: &fxhash::FxHashMap<String, String>) -> Cow<str> {
    let pkg = &args["package"];
    if pkg.is_empty()  {
        pkg.into()
    } else {
        format!("{}.{}", args["group"], args["name"]).into()
    }
}
