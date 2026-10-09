# Natives

LWJGL ships Windows `.dll`, Linux `.so`, and macOS `.dylib` files inside jars.
The JVM does not load them from the jar. `licht install` downloads the jar for
this machine into `libraries/`. `licht launch` extracts the binaries into
`natives/<version id>` and passes that folder as `-Djava.library.path`.

The folder stays in the data directory. A second launch extracts into the same
place and overwrites a file that has the same name. A version id of empty,
`.`, `..`, or one that contains a slash is rejected.

## Which jar

Two shapes appear in the version JSON. Only the jar for this operating system
and architecture is downloaded.

Old libraries, through the 1.12 era, keep a `natives` map.
`natives-windows-${arch}` becomes `natives-windows-64` or `natives-windows-32`.
The classifier jar is the one extracted. The plain library jar stays on the
classpath.

Current libraries put the classifier in the Maven name, as in
`org.lwjgl:lwjgl:3.3.3:natives-windows`. That jar is extracted and left off the
classpath. The plain name is 64-bit. `natives-windows-x86` is 32-bit.
`natives-linux` and `natives-windows-arm64` are other machines, so a 64-bit
Windows install skips them. macOS accepts `natives-osx` and `natives-macos`.

Licht knows `x86` and `x86_64`. It does not select an arm64 jar.

## What is extracted

The extract keeps `.dll`, `.so`, and `.dylib`, and drops the rest of the jar,
including `META-INF`. A prefix in `extract.exclude` is dropped too. A nested
path is flattened: `windows/x64/org/lwjgl/lwjgl.dll` lands as `lwjgl.dll` in
the version folder. An entry that leaves the destination fails the launch.

26.x (LWJGL 3.4) stores the DLL under `windows/x64` inside the jar and asks
for `-Djava.library.path=${natives_directory}/java`. The DLL is already in
`natives/<version id>`, so that flag is launched as the version folder.
1.8.9 and 1.21 store the binary at the jar root and pass
`${natives_directory}` with no `/java` suffix, so the flag is left as written.

Before 1.13 the JSON has no JVM arguments. The core inserts
`-Djava.library.path=${natives_directory}` itself.
