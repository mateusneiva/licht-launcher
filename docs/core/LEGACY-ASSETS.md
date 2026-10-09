# Sound on old versions

## The problem

`licht install` stores every asset as `assets/objects/<first two hash characters>/<hash>`.
Clients through 1.7.2 do not open that path. They open the name in the asset
index, such as `sounds/random/click.ogg`.

With only the hash on disk, the game starts silent. It then tries
`http://s3.amazonaws.com/MinecraftResources/`, which no longer serves the
files. On 1.5.2, before the copy, the log showed that failed fetch and the
menu had no sound. The same gap hides the rest of the pack, including the
window icons.

The bytes are already Mojang's. Each object was downloaded from
`https://resources.download.minecraft.net/<first two hash characters>/<hash>`.
What is missing is the name the old client knows.

## The solution

`licht launch` copies those objects to the logical names when the index asks
for it, and sets `${game_assets}` to that folder. The copy does not change
the contents. From 1.7.3 on, the client reads the hash index itself, so the
launch does not copy.

### `pre-1.6` (`map_to_resources`)

Classic, alpha, beta, releases through 1.5.2, and snapshots through `13w23b`.
The installed 1.5.2 JSON points at this index: 749 objects, `totalSize` 49505710.

```text
https://launchermeta.mojang.com/v1/packages/3d8e55480977e32acd9844e545177e69a52f594b/pre-1.6.json
```

The copy goes to `<game directory>/resources/<logical path>`, and
`${game_assets}` is that `resources` folder. 1.5.2 is LaunchWrapper, so the
sounds land in `instances/1.5.2/.minecraft/resources`. Its arguments are
`${auth_player_name} ${auth_session} --gameDir ${game_directory} --assetsDir ${game_assets}`.
See [LAUNCHWRAPPER.md](LAUNCHWRAPPER.md) for why that folder is `.minecraft`.

### `legacy` (`"virtual": true`)

`13w24a` through 1.7.2, including 1.6, 1.6.1, 1.6.2, 1.6.4, and 1.7.2. The
installed 1.7.2 JSON points at this index: 1120 objects, `totalSize` 153475165.
The flag sits at the end of the JSON.

```text
https://launchermeta.mojang.com/v1/packages/770572e819335b6c0a053f8378ad88eda189fc14/legacy.json
```

The copy goes to `<cache>/assets/virtual/legacy/`. Every instance that uses
this index shares that folder, and `${game_assets}` points there. 1.7.2's main
class is `net.minecraft.client.main.Main`, so the instance has no `.minecraft`
subfolder and no `resources` directory. Its arguments pass
`--assetsDir ${game_assets}`.

That folder is the default resource pack of the era: 1047 `.ogg` files, 67
`.lang` files, icons, `sounds.json`, `pack.mcmeta` (`pack_format` 1, "The
default look of Minecraft"), and `READ_ME_I_AM_VERY_IMPORTANT.txt` from
Dinnerbone. There is no `.mrpack`.

### 1.7.3 and newer

Each version has its own index. 1.8.9's arguments are
`--assetsDir ${assets_root} --assetIndex ${assets_index_name}`. Objects stay
hashed. 1.8.9, 1.12.2, 1.20.1, and 26.3 are in this group.

## Copy rules

A destination file whose length already equals `size` in the index is left in
place. The download verified the SHA1. The skip does not hash the file again,
so a same-size replacement would be kept.

A logical path that is empty, or that contains an empty segment, `.`, `..`,
`\`, `:`, or an absolute segment, fails the launch. A missing object file fails
the launch too.

An index with both flags gets both copies, and `${game_assets}` is the virtual
folder. The published `pre-1.6` and `legacy` indexes each set one flag.

## Checked on Windows

1.5.2 reached the menu with sound. 1.7.2 printed `Sound engine started`,
entered a world, and exited 0. Linux has not been launched for this path.
