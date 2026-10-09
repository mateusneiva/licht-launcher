# Compatibility

Licht Launcher has to launch every version the Mojang manifest publishes. This
file records the differences that change how a launch is built. It does not
redistribute game files. The mechanism for each difference is in `docs/core`.

Sources: the [client.json](https://minecraft.wiki/w/Client.json) page on
minecraft.wiki, the [Java update tutorial](https://minecraft.wiki/w/Tutorial:Update_Java)
on the same wiki, and Windows launches of 26.3, 1.21.11, 1.20.1, 1.12.2,
1.8.9, 1.7.2, and 1.5.2.

A gap listed at the end is not implemented yet. Each one is a later change, one
at a time.

## Arguments

Until 1.12.2 the version JSON stores game arguments in `minecraftArguments`, one
string split on ASCII whitespace. Since 1.13 (`17w43a`) the same data is
`arguments.game` and `arguments.jvm`: each entry is a string, or an object with
`rules` and `value`.

Before 1.13 the JSON does not carry JVM arguments. The core inserts
`-Djava.library.path=${natives_directory}`, `-cp`, and the classpath, then the
`mainClass`, then the game arguments. From 1.13 on, those JVM arguments come
from the JSON and the core evaluates them, including rule-gated entries.

`mainClass` is `net.minecraft.launchwrapper.Launch` through 1.5.2, except the
snapshot `13w16b`. Later versions use `net.minecraft.client.main.Main`. The
`.minecraft` folder for LaunchWrapper is in
[LAUNCHWRAPPER.md](core/LAUNCHWRAPPER.md).

## Offline account

The offline id is the UUID version 3 of the UTF-8 string `OfflinePlayer:<name>`,
written without hyphens. The access token is `0`.

1.5.2's game arguments are
`${auth_player_name} ${auth_session} --gameDir ${game_directory} --assetsDir ${game_assets}`.
`${auth_session}` is `0:<uuid>`.

From 1.7 onward the JSON uses `${auth_access_token}`, `${auth_uuid}`,
`${user_type}`, and `${user_properties}`. The installed 1.7.2 arguments are
`--username ${auth_player_name} --version ${version_name} --gameDir ${game_directory} --assetsDir ${game_assets} --uuid ${auth_uuid} --accessToken ${auth_access_token}`.
Current versions also mention `${auth_xuid}` and `${clientid}`; both are empty
for an offline account. `${user_type}` is `legacy` and `${user_properties}` is
`{}`.

The core fills the modern names, `auth_player_name`, and `auth_uuid`. It leaves
`${auth_session}` literal (see the gaps).

## Assets

The `pre-1.6` index sets `map_to_resources`. That covers classic, alpha, beta,
and releases through 1.5.2, plus snapshots through `13w23b`. Launch copies the
hashed objects to `<game directory>/resources` and sets `${game_assets}` there.

The `legacy` index sets `virtual`. That covers `13w24a` through 1.7.2. The copy
goes to `<cache>/assets/virtual/legacy`, and that folder is `${game_assets}`.

From 1.7.3 on, objects stay hashed under `assets/objects` and the game receives
`${assets_root}` and `${assets_index_name}`. 1.8.9, 1.12.2, 1.20.1, and 26.3
are in this group.

The missing-sound failure and the copy rules are in
[LEGACY-ASSETS.md](core/LEGACY-ASSETS.md).

## Natives

Legacy libraries name the jar in `natives`, often `natives-windows-${arch}`.
`${arch}` is `64` or `32`. The classifier jar is extracted. The plain library
jar stays on the classpath.

Current libraries put `natives-windows` in the Maven name. That jar is extracted
and left off the classpath. The plain name is 64-bit. `natives-windows-x86` is
32-bit. Other classifiers belong to other machines.

26.x stores `lwjgl.dll` under `windows/x64` inside the jar and asks for
`${natives_directory}/java`. Extraction flattens the DLL into
`natives/<version id>`, and the launch passes that folder. 1.8.9 and 1.21
store the library at the jar root.

Which jar is downloaded, and what the extract keeps, is in
[NATIVES.md](core/NATIVES.md).

## Java

`javaVersion.majorVersion` wins. Without that field, the id decides: 1.17 is
16, 1.18 through 1.20.4 is 17, 1.20.5 and newer is 21, and every other id is 8.
The installed runtime is Eclipse Temurin for that major, a JRE when Adoptium
publishes one. `--java` skips Temurin. A Java already installed on the machine
is not selected.

Installed JSONs from the Windows checks:

| Version                     | Major | Component in the JSON  |
| --------------------------- | ----- | ---------------------- |
| 1.5.2, 1.7.2, 1.8.9, 1.12.2 | 8     | `jre-legacy`           |
| 1.20.1                      | 17    | `java-runtime-gamma`   |
| 1.21.11                     | 21    | `java-runtime-delta`   |
| 26.3                        | 25    | `java-runtime-epsilon` |

The wiki's minimums, for checking a JSON that looks wrong:

- Java 8 through 1.16.5
- Java 16 on 1.17
- Java 17 from 1.18 through 1.20.4
- Java 21 from 1.20.5 through 1.21.11
- Java 25 from 26.1 onward

Java 8 reports Windows 11 as "Windows 8.1 (6.3)". That string is not a launch
failure. Download, folder names, and `--java` are in
[RUNTIME.md](core/RUNTIME.md).

## Log

Java 18 and newer are read as UTF-8. An older Java on this Windows machine is
read as Windows-1252, so `INFORMAÇÕES` in the 1.5.2 log stays intact. A strict
UTF-8 read used to fail the launch on that line. Page 65001 stays UTF-8. Any
other page, and Linux, is lossy UTF-8. A byte that does not fit becomes
U+FFFD, and the process keeps running.

The failure and the reader are in [LOG.md](core/LOG.md).

## Version ids

Classic releases are `1.x`. Since 2026 the id is `year.week` (`26.1`, `26.3`).
Both are ordinary manifest ids, along with `snapshot`, `old_beta`, and
`old_alpha`. Nothing in install or launch inspects the shape. The manifest,
the SHA1 check, and `versions/<id>/<id>.json` are in
[VERSIONS.md](core/VERSIONS.md).

## Known gaps

These are not implemented.

**Legacy session.** Where the arguments still name `${auth_session}`, Licht
leaves that token literal. The installed 1.5.2 JSON is one of those. The value
the client expects is `0:<uuid>`.

**One codec for Java 17 and older.** A UTF-8 accent in that log can be
misread. A code page other than 1252 or 65001 is not decoded as that page.
Those bytes become U+FFFD when they are not valid UTF-8. See
[LOG.md](core/LOG.md).
