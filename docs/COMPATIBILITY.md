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

The `pre-1.6` index sets `map_to_resources`. That covers classic, alpha, beta,
and releases through 1.5.2, plus snapshots through `13w23b`. At launch the
hashed objects are copied to `<game directory>/resources/<logical path>`, and
`${game_assets}` is that `resources` folder. A file that already has the size
from the index is left in place. A logical path of empty, `.`, or `..` is
rejected.

The `legacy` index sets `virtual`. That covers `13w24a` through 1.7.2. The same
copy goes to `<cache>/assets/virtual/<index id>`, and `${game_assets}` is that
folder. An index with both flags gets both copies, and the virtual folder is
the one passed to the game.

From 1.7.3 on, objects stay hashed under `assets/objects` and the game receives
`${assets_root}` and `${assets_index_name}`. Those indexes do not ask for a
copy. 1.8.9 and 1.21.11 are in this group.

## Natives

Legacy libraries name the jar in `natives`, often `natives-windows-${arch}`.
`${arch}` is `64` or `32`. The classifier jar is downloaded and extracted. The
plain library jar stays on the classpath.

Current libraries put `natives-windows` in the Maven name. That jar is extracted
and left off the classpath. The plain name is 64-bit;
`natives-windows-x86` and `natives-windows-arm64` are other machines.

26.x (LWJGL 3.4) stores `lwjgl.dll` under `windows/x64` inside the jar. Extraction
puts that DLL in the version folder. The JVM flag `${natives_directory}/java`
is launched as the version folder, where the DLL sits. 1.8.9 and 1.21 already
store the library at the jar root.

Extracted natives stay in the data directory, not in a temporary folder:

`natives/<version id>`

The binaries for this machine sit in that folder. A version id of empty, `.`,
or `..` is rejected. A second launch extracts over the same folder. Only the
current operating system and architecture are downloaded.

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

Old clients print the system code page. On this Windows machine that is
Windows-1252, so a line such as the Portuguese word for "information" shows up
in the 1.5.2 log. The launcher keeps that line and does not treat the byte as
a failed launch.

Java 18 and newer are read as UTF-8. An older Java is read as the system code
page: Windows-1252 on code page 1252, UTF-8 on code page 65001, and lossy UTF-8
for any other page. Linux uses lossy UTF-8 for the older Java. A byte that
does not fit the chosen codec becomes U+FFFD, and the process keeps running.
One codec covers the whole process, so a UTF-8 line from Java 17 can show a
wrong accent.

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

**Legacy session.** `${auth_session}` is not filled. Pre-1.6 and 1.6 through
1.7.2 arguments still contain that name.

**Non-UTF-8 log lines on a code page other than 1252 or 65001.** Those bytes
are kept with U+FFFD. Java 17 and older still use one codec for the whole
process, so a UTF-8 accent in that log can be misread.
