
plugins {
    alias(libs.plugins.kotlin.jvm)
    alias(ktorLibs.plugins.ktor)
    alias(libs.plugins.kotlin.serialization)
}

group = "dev.stephyu"
version = "1.0.0-SNAPSHOT"

application {
    mainClass = "io.ktor.server.netty.EngineMain"
}

kotlin {
    jvmToolchain(21)
}
dependencies {
    implementation(ktorLibs.serialization.kotlinx.json)
    implementation(ktorLibs.server.config.yaml)
    implementation(ktorLibs.server.contentNegotiation)
    implementation(ktorLibs.server.core)
    implementation(ktorLibs.server.cors)
    implementation(ktorLibs.server.netty)
    implementation(ktorLibs.server.openapi)
    implementation(ktorLibs.server.routingOpenapi)
    implementation(libs.koin.ktor)
    implementation(libs.koin.loggerSlf4j)
    implementation(libs.logback.classic)

    testImplementation(kotlin("test"))
    testImplementation(ktorLibs.server.testHost)
}

tasks.register<JavaExec>("chatCli") {
    group = "application"
    description = "Starts a terminal chat client against the running chatbot backend."
    classpath = sourceSets["main"].runtimeClasspath
    mainClass.set("dev.stephyu.cli.ChatCliKt")
    standardInput = System.`in`

    val endpoint = providers.gradleProperty("chatbotApiUrl")
        .orElse(providers.environmentVariable("CHATBOT_API_URL"))
    if (endpoint.isPresent) {
        args(endpoint.get())
    }
}
