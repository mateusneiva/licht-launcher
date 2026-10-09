# Versions

## The problem

Mojang publishes one list. It mixes four types, and the id is not a single
shape: `1.5.2` and `26.3` are both releases. A launcher that parses the id as
a version number, or that keeps only `release`, drops snapshots, old beta,
old alpha, and the year-week ids.

## The solution

The id is the string in the manifest. Nothing in install or launch inspects
its shape. `licht versions` prints every entry and does not install.
`licht install --version <id>` finds that id, downloads its JSON, checks the
SHA1 published beside it, and writes `versions/<id>/<id>.json`. `licht launch`
reads that file. It does not download the list again. An id that is empty,
`.`, `..`, or that contains a slash is rejected.

The list is:

```text
https://piston-meta.mojang.com/mc/game/version_manifest_v2.json
```

Each line from `licht versions` is `<id> <type>`. The types are `release`,
`snapshot`, `old_beta`, and `old_alpha`. The manifest also names
`latest.release` and `latest.snapshot`. Those ids appear in the same list.
There is no separate latest line. `--cache` is accepted and does not change
the list. The list always comes from that URL.

An id that is not in the list fails the install. A SHA1 that does not match
the manifest fails the install before the JSON is written. Launch of an id
that was never installed fails because `versions/<id>/<id>.json` is missing.

`complianceLevel` is on each manifest entry. Install and launch do not branch
on it.

The JSON is the source for the client, the libraries, the asset index, the
main class, and `javaVersion`. What Licht does with those fields is in
[LAUNCHWRAPPER.md](LAUNCHWRAPPER.md), [LEGACY-ASSETS.md](LEGACY-ASSETS.md),
[RUNTIME.md](RUNTIME.md), [NATIVES.md](NATIVES.md), and [LOG.md](LOG.md).
