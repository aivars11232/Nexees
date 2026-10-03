plugins {
    id("com.android.application") version "9.4.1" apply false
}

// Build output stays in the isolated toolchain folder, not in the repository.
layout.buildDirectory.set(file("/mnt/F/Nexees-toolchains/android-build/root"))
