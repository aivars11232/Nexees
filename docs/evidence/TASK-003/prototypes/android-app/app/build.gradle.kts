plugins {
    id("com.android.application")
}

// Inputs that stay outside this repository, prepared by ../build.sh: the shared
// Rust library built by cargo from ../feasibility-core, a byte-identical copy of
// the canonical LCL Core packages, and the owner's LCL user manual (read only).
val toolchains = "/mnt/F/Nexees-toolchains"
val rustLibrary = "$toolchains/target/x86_64-linux-android/release/libnexees_feasibility.so"
val lclCore = "$toolchains/stage/lcl"
val lclManual = "/mnt/F/LCL/users_manual"

layout.buildDirectory.set(file("$toolchains/android-build/app"))
val generatedJni: File = layout.buildDirectory.dir("generated/jniLibs").get().asFile
val generatedAssets: File = layout.buildDirectory.dir("generated/assets").get().asFile

val syncNativeLibrary = tasks.register<Sync>("syncNativeLibrary") {
    from(rustLibrary) { into("x86_64") }
    into(generatedJni)
}

val syncAssets = tasks.register<Sync>("syncAssets") {
    from("$lclCore/LCL_Core_0.1.0") { into("nexees/LCL_Core_0.1.0") }
    from("$lclCore/LCL_Core_0.3.0") { into("nexees/LCL_Core_0.3.0") }
    from(lclManual) {
        include("*.md")
        into("nexees/users_manual")
    }
    from(rootProject.file("../feasibility-core/fixtures")) { into("nexees/fixtures") }
    into(generatedAssets)
}

android {
    namespace = "dev.nexees.feasibility"
    compileSdk = 36

    defaultConfig {
        applicationId = "dev.nexees.feasibility"
        minSdk = 29
        targetSdk = 36
        versionCode = 1
        versionName = "0.0.0-task003"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets.getByName("main") {
        jniLibs.directories.add(generatedJni.path)
        assets.directories.add(generatedAssets.path)
    }
}

tasks.named("preBuild") { dependsOn(syncNativeLibrary, syncAssets) }
