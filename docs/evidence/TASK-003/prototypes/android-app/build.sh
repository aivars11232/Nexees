#!/bin/sh
# Builds the TASK-003 Android feasibility APK. Everything it downloads, builds
# or caches stays in the isolated toolchain folder; it reads the owner's LCL
# checkout and changes nothing there.
set -eu
. /mnt/F/Nexees-toolchains/env.sh
here=$(cd "$(dirname "$0")" && pwd)

cargo build --manifest-path "$here/../feasibility-core/Cargo.toml" --offline --release \
    --target x86_64-linux-android --lib

# Gradle refuses to snapshot the 1901 directory timestamps in the canonical
# Core 0.1.0 package, so the APK takes a copy with fresh timestamps. diff -r
# proves the copy is byte-identical; the engine verifies it again on the phone.
stage=$T/stage/lcl
rm -rf "$stage"
mkdir -p "$stage"
for core in LCL_Core_0.1.0 LCL_Core_0.3.0; do
    cp -r "/mnt/F/LCL/canonical/$core" "$stage/$core"
    diff -r "/mnt/F/LCL/canonical/$core" "$stage/$core"
done

cd "$here"
HOME=$T/home gradle --no-daemon --project-cache-dir "$T/gradle-project-cache" \
    -Pkotlin.project.persistent.dir="$T/kotlin-project" \
    -Dorg.gradle.jvmargs="-Xmx2g -Dfile.encoding=UTF-8 -Duser.home=$T/home" \
    assembleDebug
