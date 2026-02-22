plugins {
    id("java")
    id("org.jetbrains.kotlin.jvm") version "1.9.25"
    id("org.jetbrains.intellij") version "1.17.4"
}

group = "com.volki"
version = "0.1.0"

repositories {
    mavenCentral()
}

intellij {
    version.set("2024.1")
    type.set("IC")
    plugins.set(listOf())
}

java {
    sourceCompatibility = JavaVersion.VERSION_17
    targetCompatibility = JavaVersion.VERSION_17
}

tasks {
    val rootLogo = project.file("../../../assets/logo.svg")

    withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
        kotlinOptions.jvmTarget = "17"
    }

    withType<JavaCompile> {
        sourceCompatibility = "17"
        targetCompatibility = "17"
    }

    processResources {
        from(rootLogo) {
            into("META-INF")
            rename { "pluginIcon.svg" }
        }
        from(rootLogo) {
            into("META-INF")
            rename { "pluginIcon_dark.svg" }
        }
        from(rootLogo) {
            into("icons")
            rename { "volki_file.svg" }
        }
        from(rootLogo) {
            into("assets")
            rename { "logo.svg" }
        }
    }

    patchPluginXml {
        sinceBuild.set("241")
        untilBuild.set("253.*")
    }

    signPlugin {
        certificateChain.set(System.getenv("CERTIFICATE_CHAIN"))
        privateKey.set(System.getenv("PRIVATE_KEY"))
        password.set(System.getenv("PRIVATE_KEY_PASSWORD"))
    }

    publishPlugin {
        token.set(System.getenv("PUBLISH_TOKEN"))
    }
}
