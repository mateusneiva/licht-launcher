# CLI

`licht` lists, installs, and launches game versions from the command line. The
binary lives in `crates/licht-core` (`src/bin/licht.rs`) and calls the core
directly. It has no Tauri window and no Microsoft login. Accounts are offline
names only.

This project is unofficial and is not affiliated with Mojang Studios or
Microsoft. Game files are downloaded from Mojang at install time. The CLI does
not redistribute them.

The desktop app in `src-tauri` is also named `licht`. Both packages write
`target/debug/licht` (`licht.exe` on Windows), so a later build replaces the
other binary. Run the CLI through Cargo with the package name:

```sh
cargo run -p licht-core --bin licht --locked -- <command>
```

There is no `--help`. A missing command, an unknown flag, or a flag without a
value exits with status 1 and prints `command arguments are incomplete`.

## Shared rules

Flags may appear in any order. Each flag takes one value, and that value cannot
start with `--`. Repeating a flag keeps the last value. Paths are passed as
written; the core joins cache segments with `std::path`.

`--cache <dir>` selects the data directory for `install` and `launch`. Without
it, the directory comes from the `directories` crate
(`ProjectDirs::from("app.licht", "Licht", "Licht Launcher")`, method
`data_dir`):

| System | Default data directory |
| --- | --- |
| Windows | `%APPDATA%\Licht\Licht Launcher\data` |
| Linux | `$XDG_DATA_HOME/lichtlauncher`, or `~/.local/share/lichtlauncher` |

This is Licht's own tree. It is not the official `.minecraft` folder.

The host must be Windows, Linux, or macOS, and the architecture must be
`x86_64` or `x86`. Anything else stops with `this system is not supported for
launch`. The supported product targets are Windows and Linux.

## `licht versions`

Prints one line per entry in Mojang's version manifest
(`https://piston-meta.mojang.com/mc/game/version_manifest_v2.json`). The line is
the version id and its type: `release`, `snapshot`, `old_beta`, or `old_alpha`.
Order is the order of the manifest. Nothing is downloaded into the cache.

```sh
cargo run -p licht-core --bin licht --locked -- versions
```

```text
1.21.11 release
24w14a snapshot
```

`--cache` is accepted and ignored. The list always comes from the network.

## `licht install`

Downloads one version and a Java runtime into the cache.

```sh
cargo run -p licht-core --bin licht --locked -- install --version 1.21.11
```

| Flag | Required | Meaning |
| --- | --- | --- |
| `--version <id>` | yes | Id from `licht versions`. An unknown id exits with `version was not found`. |
| `--cache <dir>` | no | Data directory. Default is the system path above. |
| `--java <path>` | no | Existing `java` or `java.exe`. Skips the runtime download. |

The command fetches the manifest, downloads that version's JSON, and checks its
SHA1 against the manifest. It writes `versions/<id>/<id>.json`, then the client
jar, libraries, and assets. A file already on disk is skipped when its SHA1
matches. Assets come from `https://resources.download.minecraft.net`.

Without `--java`, the runtime is an Eclipse Temurin JRE for the major version
the version JSON requests (`javaVersion.majorVersion`, or a fallback from the
release id). It is downloaded from `https://api.adoptium.net` and checked with
SHA256. A matching Temurin folder already in `runtime/` is reused. `--java`
must point at a file; a missing path exits with `custom Java executable was not
found`.

Downloads run up to 8 at a time, with 3 attempts and a 200 ms initial backoff.
On a terminal, one progress line is redrawn about every 100 ms:

```text
[████████████████████████] 100.0%  80.2MB/80.2MB
```

The bar is 24 cells. The numbers are decimal megabytes (1 MB = 1 000 000
bytes) with one decimal place. A new phase prints on the next line. When stdout
is not a terminal, a line is printed when the phase changes, every 100 finished
files, and when the phase completes. After a successful install the Java
executable path is printed:

```text
java C:\Users\Alice\AppData\Roaming\Licht\Licht Launcher\data\runtime\temurin21-jre21.0.2-win_x64\bin\java.exe
```

## `licht launch`

Starts a version that is already in the cache. It does not download the client,
libraries, assets, or a runtime.

```sh
cargo run -p licht-core --bin licht --locked -- launch --version 1.21.11 --username Alice
```

| Flag | Required | Meaning |
| --- | --- | --- |
| `--version <id>` | yes | Installed version. The JSON must already be at `versions/<id>/<id>.json`. |
| `--username <name>` | yes | Offline player name. Empty is rejected (`offline username is empty`). |
| `--game-dir <dir>` | no | Instance root. Without it, the CLI creates `instances/<id>` next to the data directory. |
| `--java <path>` | no | Java executable. Without it, the Temurin runtime already in the cache is used. |
| `--cache <dir>` | no | Data directory. Default is the system path above. |

On Windows that default is `%APPDATA%\Licht\Licht Launcher\instances\1.21.11`. On Linux, with the data directory at `~/.local/share/lichtlauncher`, it is `~/.local/share/instances/1.21.11`. `--game-dir` replaces that root and is not created by the CLI.

The offline id is the UUID version 3 of `OfflinePlayer:<name>`, without
hyphens. The access token is `0`. See [COMPATIBILITY.md](COMPATIBILITY.md) for
how older versions substitute that account.

Natives are extracted into `natives/<id>` for this run. The process is the Java
executable with the assembled arguments. No shell is used. Stdin is closed.
Stdout and stderr from the game are forwarded. The process exit code becomes
the CLI exit code. If the process ends without a code, the CLI exits with 1.

A missing runtime exits with `Java runtime was not found for this system`.

## Cache layout

Under the data directory:

| Path | Contents |
| --- | --- |
| `versions/<id>/<id>.json` | Version JSON from Mojang |
| `versions/<id>/<id>.jar` | Client jar |
| `libraries/` | Library artifacts |
| `assets/indexes/` | Asset index JSON |
| `assets/objects/<hh>/<sha1>` | Asset objects, keyed by SHA1 |
| `natives/<id>/` | Natives extracted for a launch |
| `runtime/temurin<major>-<image><version>-<os>_<arch>/` | Eclipse Temurin |

`<os>_<arch>` is `win_x64`, `win_x86`, `linux_x64`, `linux_x86`, `mac_x64`, or
`mac_x86`. Version ids and other single path segments cannot be empty, `.`,
`..`, or contain a slash.

## Exit status and errors

| Status | When |
| --- | --- |
| `0` | `versions` or `install` finished, or the game exited 0 |
| game code | `launch`, when the process reported a code |
| `1` | Argument, network, I/O, hash, Java, or host failure, or a game process with no exit code |

Errors go to stderr: the core message, then each cause indented by two spaces.
Logs must not contain tokens or personal data. This CLI does not configure
`tracing`.
