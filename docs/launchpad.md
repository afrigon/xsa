# Launchpad

How packs are released, listed, installed and loaded. `launchpad` is the
instance and pack manager: it packages packs for release, creates instances
and installs packs into them. The game uses the same library, so an instance
the game creates on its own is identical to one created by `launchpad`.
The pack format itself is described in `docs/packs.md`.

## Terms

- **Pack:** a directory with a `pack.kdl` manifest (`docs/packs.md`).
- **Release:** one version of a pack, published as one archive.
- **Archive:** a `.tar.zst` file holding part of a release.
- **Repository:** `afrigon/xsa-repository`, a GitHub repository whose
  `index.kdl` lists every pack and where each release's archive is hosted.
- **Instance:** a directory the game runs from: its packs, its lockfile, its
  saves.
- **Lockfile:** the exact release of every pack an instance uses, resolved from
  its manifest.

## Releases

Authors host their own archives anywhere reachable over HTTPS. First-party
packs are hosted on the xsa repository's GitHub Releases, one release per pack
version, tagged `<pack>-v<version>` (`base-v0.0.1`, `system-solar-v0.0.1`);
game releases are tagged `xsa-v<version>`.

A release is one archive, `<pack>-v<version>.tar.zst`, holding the whole pack:
data and resources together. It holds exactly one top-level directory named
after the pack, laid out as in `docs/packs.md`: `base-v0.0.1.tar.zst` contains
`base/pack.kdl`, `base/base/data/…`, `base/base/resources/…`. Which files an
instance needs is `launchpad`'s concern at install time (see Sync), never the
author's.

Large packs are split by hand into packs that depend on each other
(`system-solar` and its per-system packs). There are no variants of a pack: a
different set of content is a different pack.

## Packaging

```
launchpad pack package <pack directory> [--bump major|minor|patch] [--output <directory>]
```

1. Reads and validates `pack.kdl`; the pack id is the directory's name.
2. With `--bump`, increments that part of `version` in `pack.kdl` and saves it
   with its comments and layout kept. Without it, the current version is
   packaged.
3. Lists the pack's files, leaving out what `.xsaignore` excludes.
4. Converts every terrain layer the pack's terrain definitions name from its
   cube map into a `.terrain` file (see Terrain conversion); the archive ships
   the `.terrain` files in place of the cube maps.
5. Writes the archive, ready to upload, to the output directory
   (`<pack directory>/target/` by default) and prints its path, size and
   SHA-256.

`target/` is packaging's own directory, like Cargo's: it holds the archive and
the converted `.terrain` files, which a later run reuses when their cube map has
not changed. Authors git-ignore it.

Archives are written in Rust (the `tar` and `zstd` crates), so packaging needs
nothing installed on the system. Compression uses zstd level 19 on every core:
slow to package, as fast as any level to extract. Entries are sorted, with a
fixed modification time, owner and permissions, so the same files always
produce the same archive.

### Terrain conversion

```
launchpad terrain create [--height] [--output <path>] <input>
```

Streamed terrain ships as `.terrain` files, one per terrain layer (heights,
colour, normals, each Hapke map), holding every page of the layer at every
level. Creators author each layer as a KTX2 cube map and never write a
`.terrain` file by hand; the format is specified in #299 (Convert terrain cube
maps into streamed pages). Packaging runs this conversion on every layer; the
command also runs on its own, so a creator can check their data before
packaging. system-solar's terrain layers are produced by it.

The input is a KTX2 cube map:

- six square faces, +X, −X, +Y, −Y, +Z, −Z of the body's frame (+X at longitude
  0, +Z the north pole), in Vulkan's cube-map face order and orientation;
- a face size that is a power of two, at least 128;
- its full mip chain, each level the average of the one above;
- uncompressed texels, since page borders are rotated across face edges and
  compressed blocks cannot be.

The output is 136×136-texel pages (a 128×128 interior and a 4-texel border
copied from the neighbouring pages), each compressed with zstd. Without
`--output`, it is written next to the input, with the same name and the
`.terrain` extension.

| Layer | Input texels | Pages |
| --- | --- | --- |
| Heights (`--height`) | `R32_SFLOAT`, metres relative to the body's radius | `R16_UNORM` over each page's own height range, stored in the table of contents |
| Any other layer | `R8G8B8A8_SRGB` | `BC7_SRGB_BLOCK` |
| | `R8G8_UNORM` | `BC5_UNORM_BLOCK` |
| | `R16G16B16A16_SFLOAT` | `R16G16B16A16_SFLOAT` |

`--height` selects the height encoding, the only one whose pages differ in
structure; every other layer keeps its texel format, block-compressed when it
has 8 bits per channel, whatever the layer means. An input that matches no row
is refused. The page size and border are constants of the format: they match
4-tap cubic filtering and 4×4 compression blocks, and the game's page
addressing relies on them.

`launchpad terrain create --help` lists these requirements in full, for the
artists producing the cube maps.

### `.xsaignore`

A pack directory can hold files that are not content: the scripts that produce
its assets, their downloads, a `.gitignore`, a `mise.toml`. `.xsaignore` at the
pack's root lists what packaging leaves out, in `.gitignore` syntax.

```
scripts/
cache/
mise.toml
```

`.gitignore` is never consulted: a pack's generated binaries are git-ignored
but belong in its archive. `.git/`, `target/` and `.xsaignore` itself are always left
out.

## Repository

`afrigon/xsa-repository` lists packs and their releases in one `index.kdl`,
served from its `main` branch over `raw.githubusercontent.com`. Changes arrive
as pull requests, merged only by a maintainer. The repository's tooling lives
in that repository, in Rust, never in the game's.

```kdl
format 1

pack "system-solar" {
    description "The real solar system"
    homepage "https://github.com/afrigon/xsa"
    license "MIT"

    release "0.0.1" {
        dependencies {
            system-solar-earth "^0.0.1"
        }
        archive url="https://github.com/afrigon/xsa/releases/download/system-solar-v0.0.1/system-solar-v0.0.1.tar.zst" size=268435456 sha256="…"
    }
}
```

- **`pack`:** written by hand once per pack: its description, homepage and
  license.
- **`release`:** written only by the repository's tool. `xsa-repository
  add-release <url>` downloads the archive, checks that it holds one
  top-level directory named after the pack, reads the manifest, and writes the
  entry: the version, the dependencies and `provides` copied from the manifest
  (so resolution never downloads an archive), and the archive's size and
  SHA-256 computed from the download. Hashes are never taken from an author.
- **`yanked=#true`** on a release: no longer chosen by resolution, still
  installed by lockfiles that already name it.

## Instances

Instances live in `$HOME/xsa/instances/`. The client and the dedicated server
use instances the same way; the client's default instance is `main`, the
server's is `server`, and `--instance <name>` selects another.

```
$HOME/xsa/instances/main/
  launchpad.kdl     the instance's manifest
  launchpad.lock    resolved releases
  config.kdl        overrides of the global game config, optional
  packs/<pack>/     installed packs
  saves/
  identity/         the dedicated server's certificate and key
```

Everything the game writes for an instance stays inside it: the dedicated
server generates its identity in `identity/` on first run, so each server
instance has its own fingerprint.

`launchpad.kdl` declares what the instance runs, a client or a dedicated
server, and lists the packs the player asked for:

```kdl
format 1
role "client"

packs {
    system-solar "^0.0.1"
}
```

- **`role`:** `client` or `server`. A new `main` instance is a client, a new
  `server` instance a server. How the instance is started, and with which
  options, belongs here too, so one `launchpad run` starts any instance.

`base` is an implicit dependency of every pack and of every instance:
manifests never list it, and an instance without it is completed before
anything else runs.

### Lockfile

`launchpad.lock` records, for every pack in the resolved set, the exact
version and its archive's URL and SHA-256. It works like Cargo's:

- It is written by resolution and only read otherwise.
- Resolution runs when `launchpad.kdl` changes, keeping every locked release
  that still satisfies it, or on `upgrade`, which picks the latest matching
  releases.
- Each pack id resolves to one release: requirements are Cargo-style semantic
  version requirements, and two requirements no release satisfies together
  fail the resolution. A dependency on a provided name (`provides`, see
  `docs/packs.md`) is met by the one pack providing it.

### Sync

Syncing makes `packs/` match the lockfile. A
missing or outdated pack is downloaded into the archive cache
(`$XDG_CACHE_HOME/xsa/archives/`), verified against its SHA-256, extracted into
a staging directory inside the instance, then moved in place of the old
`packs/<pack>/` as a whole. The instance's role decides what is extracted: a
server gets `pack.kdl`, `LICENSE` and every `<namespace>/data/`, the only files
it reads; a client gets the whole pack. A failed download or
mismatched hash leaves the installed pack untouched. Installing replaces a
pack's directory entirely; packs are never edited in place.

## Loading packs

The game loads packs from its search path, then from its instance's `packs/`.
`--packs <directory>` adds directories to the search path, separated by the
platform's path separator (`:` on Linux, `;` on Windows), like `$PATH`; for each
pack id, the first directory containing that pack wins. Development builds run
with `--packs data`, so the repository's packs take precedence over installed
ones.

When the instance does not exist, the game creates it as `launchpad`
would, with `system-solar` in its manifest, and syncs it before loading.
Once synced, an instance runs without a network connection.

## Config

The game's config (`$XDG_CONFIG_HOME/xsa/config.kdl`) applies to every
instance; an instance's `config.kdl` overrides individual keys.

`launchpad`'s own settings live in `$XDG_CONFIG_HOME/xsa/launchpad.kdl` (falling
back to `~/.config/xsa/launchpad.kdl`): the instances directory and repositories
besides `afrigon/xsa-repository`. The game reads the instances directory from
it too.

## Commands

`launchpad` acts on the instance in the current directory, or on
`--instance <name>`.

| Command | Does |
| --- | --- |
| `init <name>` | Creates an instance with `system-solar` in its manifest and syncs it |
| `add <pack>[@<requirement>]` | Adds the pack to `launchpad.kdl`, resolves and syncs |
| `remove <pack>` | Removes the pack from `launchpad.kdl`, resolves and syncs |
| `upgrade [<pack>]` | Resolves to the latest matching releases and syncs |
| `pack package <pack directory>` | Packages a pack for release (see Packaging) |
| `terrain create <input>` | Converts a terrain layer's cube map into a `.terrain` file (see Terrain conversion) |

Publishing is not a launchpad command: an author uploads archives to their own
host and opens a pull request on the repository.

## Crates

| Crate | Responsibility |
| --- | --- |
| `packs` | The manifest (`PackManifest`), with `base` implicit |
| `repository` | Reads the index, downloads archives and verifies them, the archive cache |
| `instance` | Instance layout, `launchpad.kdl`, the lockfile and resolution, sync |
| `launchpad` | `xsa-launchpad`, building the `launchpad` binary: commands, packaging, terrain conversion |

Dependencies point one way: `packs` ← `repository` ← `instance` ← `launchpad`,
`xsa`. `launchpad` follows the architecture of
[mc](https://github.com/afrigon/mc): command handlers stay thin and call
operations that hold the logic, a context carries the output shell and the HTTP
client, and manifest edits keep the document's layout.
