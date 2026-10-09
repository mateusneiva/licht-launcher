# LaunchWrapper game directory

Versions through 1.5.2, except snapshot `13w16b`, start with
`net.minecraft.launchwrapper.Launch`. From 1.6 the main class is
`net.minecraft.client.main.Main`, and `--gameDir` applies from the start.

Licht treats a version as LaunchWrapper when its main class is
`net.minecraft.launchwrapper.Launch`, or when any library name starts with
`net.minecraft:launchwrapper:`.

## The race

The LaunchWrapper injector writes `--gameDir` on the `return` of `Minecraft.main`.
The client thread can read options before that write.

On Windows the default folder is `%APPDATA%\.minecraft`. On Linux it is
`user.home` plus `/.minecraft`. If that path is the official `.minecraft`, 1.5.2
reads `lang:en_us` from its `options.txt` and crashes: the client jar contains
`lang/en_US.lang`, and `en_us.lang` is absent.

Passing `--gameDir` alone does not close the race. Both readers have to land on
the instance.

## How other launchers handle it

The old client always appends the literal name `.minecraft`. A separate instance
folder has to use that name, or the launcher has to set the folder before the
client reads it.

- **Official launcher.** The game directory is `%APPDATA%\.minecraft` (or
  `--workDir`, which is still a `.minecraft` folder). The default path and
  `--gameDir` are the same place, so the race does not show up. There is no
  separate instance.
- **HMCL.** Sets `APPDATA` to the parent of the game directory on every launch.
  Off Windows it also sets `-Duser.home` to that parent. The default folder is
  already named `.minecraft`, so `parent/.minecraft` is the game directory.
  Saves, versions, and libraries share that folder. A per-version directory
  named `versions/<id>` does not satisfy the hardcoded name by itself.
- **Prism and MultiMC.** They do not change `APPDATA`. The process starts in
  their own jar (`LegacyLauncher` in Prism, `OneSixLauncher.legacyLaunch` in
  MultiMC) when the version has the `legacyLaunch` or `alphaLaunch` trait.
  Before `main`, that jar takes the first `private static File` field and sets
  it to the working directory, and it sets `minecraft.applet.TargetDirectory`.
  Prism's published 1.5.2 metadata has no `mainClass` and no `launchwrapper`
  library, so vanilla 1.5.2 does not go through Mojang's injector. The game
  folder is `instances/<name>/minecraft` in Prism, unless only `.minecraft`
  already exists. MultiMC prefers `.minecraft`, including when neither folder
  exists yet. They do not rewrite `APPDATA` or `user.home`. Alpha versions
  without that field, from `inf-20100327` through `a1.0.9`, still write to the
  official `.minecraft` (MultiMC issue 2548). Scanning fields has no stable
  name and has already aborted a Prism launch with `NoClassDefFoundError`.
- **minecraft-launcher-lib and portablemc.** They only fill `--gameDir`. They
  do not change `APPDATA` or `user.home`. portablemc defaults to the official
  `.minecraft`, so the race stays hidden. A separate work directory hits the
  same 1.5.2 failure.

## Decision

Licht does not ship a Java launcher and does not create a junction.

The instance stays `<launcher>/instances/<id>`, the sibling of the data
directory. `--game-dir` replaces that root.

A LaunchWrapper version stores the game in `instances/<id>/.minecraft`. The
process working directory and `${game_directory}` are that folder. `APPDATA`
and `-Duser.home` are the instance, its parent. Windows then resolves
`%APPDATA%\.minecraft` and Linux resolves `user.home/.minecraft` to the same
folder, whichever side of the race runs first. The official `.minecraft` is
not modified.

Other versions use `instances/<id>` itself and do not change `APPDATA` or
`user.home`.

On Windows Explorer the `.minecraft` folder is hidden because of the leading
dot.

This is not the phase 6 instance model. There is no profile name, memory
setting, or instance file. One folder per version id.
