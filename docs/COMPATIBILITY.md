# Compatibility

Licht Launcher has to launch every version the Mojang manifest publishes. This
file records the differences that change how a launch is built. It does not
redistribute game files.

Sources: the [client.json](https://minecraft.wiki/w/Client.json) page on
minecraft.wiki, the [Java update tutorial](https://minecraft.wiki/w/Tutorial:Update_Java)
on the same wiki, and local launches of 26.3, 1.21.11, 1.8.9, and 1.5.2.

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
snapshot `13w16b`. Later versions use `net.minecraft.client.main.Main`.

## Offline account

The offline id is the UUID version 3 of the UTF-8 string `OfflinePlayer:<name>`,
written without hyphens. The access token is `0`.

1.5.2's game arguments are
`${auth_player_name} ${auth_session} --gameDir ${game_directory} --assetsDir ${game_assets}`.
`${auth_session}` is `0:<uuid>`.

From 1.7 onward the JSON uses `${auth_access_token}`, `${auth_uuid}`,
`${user_type}`, and `${user_properties}`. Current versions also mention
`${auth_xuid}` and `${clientid}`; both are empty for an offline account.
`${user_type}` is `legacy` and `${user_properties}` is `{}`.

The core fills the modern names, `auth_player_name`, and `auth_uuid`. It does
not fill `auth_session` yet (see the gaps).

## Assets

The `pre-1.6` index sets `map_to_resources`. The game then opens files under
`--assetsDir ${game_assets}`, which is `<game directory>/resources/<logical path>`
copied from the hashed objects. An index with `virtual` as well also copies the
same objects to `assets/virtual/<index id>`, and that directory is
`--assetsDir`. A logical path of empty, `.`, or `..` is rejected.

From 1.6 on, objects stay hashed under `assets/objects` and the game receives
`${assets_root}` and `${assets_index_name}`. Those indexes do not ask for a
copy. 1.8.9 and 1.21.11 are in this group.

The core fills `assets_root` and `assets_index_name`. It parses the two flags
and does not copy the objects yet, so `${game_assets}` is still unsubstituted
on a pre-1.6 launch.

## Natives

Legacy libraries name the jar in `natives`, often `natives-windows-${arch}`.
`${arch}` is `64` or `32`. The classifier jar is downloaded and extracted. The
plain library jar stays on the classpath.

Current libraries put `natives-windows` in the Maven name. That jar is extracted
and left off the classpath. The plain name is 64-bit;
`natives-windows-x86` and `natives-windows-arm64` are other machines.

26.x (LWJGL 3.4) stores `lwjgl.dll` under `windows/x64` inside the jar, and the
JVM flag is `-Djava.library.path=${natives_directory}/java`. The core copies the
shared library onto that directory. 1.8.9 and 1.21 keep the library at the
natives root, because their flag is `${natives_directory}` with no suffix.

Extracted natives stay in the data directory, not in a temporary folder:

`natives/<version id>/<platform>`

`<platform>` is the same name the Java runtime uses: `windows-x64`,
`windows-x86`, `linux`, `linux-i386`, `mac-os`. Linux x64 is `linux`, matching
Mojang's runtime index. A version id of empty, `.`, or `..` is rejected. A
second launch extracts over the same folder.

## Java

The version JSON decides the runtime through `javaVersion.component` and
`javaVersion.majorVersion`. The runtime is the one Mojang publishes for that
component. A Java already installed on the machine is not selected. Passing
`--java` skips the Mojang runtime.

The wiki's minimums, for checking a JSON that looks wrong:

- Java 8 through 1.16.5 (`jre-legacy` on 1.8.9 and 1.5.2)
- Java 16 on 1.17
- Java 17 from 1.18 through 1.20.4
- Java 21 from 1.20.5 through 1.21.11 (`java-runtime-delta` on 1.21.11)
- Java 25 from 26.1 onward (`java-runtime-epsilon` on 26.3)

Java 8 reports Windows 11 as "Windows 8.1 (6.3)". That string is not a launch
failure.

## Log

Old clients print bytes that are not UTF-8 (Windows-1252). A line such as the
Portuguese word for "information" shows up in the 1.5.2 log. The launcher must
keep those bytes in the line and must not treat them as a failed launch.

The reader still stops on the first non-UTF-8 byte. That abort is a gap.

## Version ids

Classic releases are `1.x`. Since 2026 the id is `year.week` (`26.1`, `26.3`).
Both are ordinary manifest ids. Nothing in the launch path special-cases the
shape.

## Known gaps

These are not fixed in the documentation change.

**1.5.2 reads the wrong options file.** LaunchWrapper 1.5 sets the game
directory by rewriting `Minecraft.main` so the assignment runs on the `return`.
`applet.start()` has already started the client thread. That thread can call
the data-directory lookup first. On Windows the fallback is
`%APPDATA%\.minecraft`. On Linux it is `~/.minecraft`. A modern `options.txt`
there has `lang:en_us` and key lines such as `key_key.attack:key.mouse.left`.
1.5.2 skips the key lines (`Skipping bad option`) and then asks the jar for
`/lang/en_us.lang`. The jar entry is `lang/en_US.lang`. The zip name is
case-sensitive, `getResourceAsStream` returns null, and
`InputStreamReader` throws `NullPointerException` (`bp.a` line 64, called from
line 100). The crash report is "Failed to start game".

`Is Modded: Jar signature invalidated` is LaunchWrapper rewriting
`Minecraft.class`. It is not this crash.

**Legacy session and assets.** `${auth_session}` and `${game_assets}` are not
filled, and a `map_to_resources` or `virtual` index is not copied into the game
directory.

**Non-UTF-8 log lines.** The pipe reader requires UTF-8, so a 1.5.2 launch can
be reported as a launcher I/O failure after the game has already started.
