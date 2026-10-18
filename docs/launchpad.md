# Launchpad

How packs are released, listed, installed and loaded. `xsa-launchpad` is the
instance and pack manager: it packages packs for release, creates instances
and installs packs into them. The game uses the same library, so an instance
the game creates on its own is identical to one created by `xsa-launchpad`.
The pack format itself is described in `docs/packs.md`.

## Terms

- **Pack:** a directory with a `pack.kdl` manifest (`docs/packs.md`).
- **Release:** one version of a pack, published as archives.
- **Archive:** a `.tar.zst` file holding part of a release.
- **Repository:** `afrigon/xsa-repository`, a GitHub repository whose
  `index.kdl` lists every pack and where each release's archives are hosted.
- **Instance:** a directory the game runs from: its packs, its lockfile, its
  saves.
- **Lockfile:** the exact release of every pack an instance uses, resolved from
  its manifest.

## Releases

Authors host their own archives anywhere reachable over HTTPS. First-party
packs are hosted on the xsa repository's GitHub Releases, one release per pack
version, tagged `<pack>-v<version>` (`base-v0.0.1`, `system-solar-v0.0.1`);
game releases are tagged `xsa-v<version>`.

A release is two archives, so a dedicated server downloads only what it reads:

| Archive | Holds |
| --- | --- |
| `<pack>-v<version>-data.tar.zst` | `pack.kdl`, `LICENSE`, every `<namespace>/data/` |
| `<pack>-v<version>-resources.tar.zst` | `pack.kdl`, `LICENSE`, every `<namespace>/resources/` |

Each archive holds exactly one top-level directory named after the pack, laid
out as in `docs/packs.md`, so extracting both into the same directory gives the
whole pack: `base-v0.0.1-data.tar.zst` contains `base/pack.kdl`,
`base/base/data/…`. Both carry the manifest, so either archive alone is a valid
pack.

Large packs are split by hand into packs that depend on each other
(`system-solar` and its per-system packs). There are no variants of a pack: a
different set of content is a different pack.

## Packaging

```
xsa-launchpad package <pack directory> [--bump major|minor|patch] [--output <directory>]
```

1. Reads and validates `pack.kdl`; the pack id is the directory's name.
2. With `--bump`, increments that part of `version` in `pack.kdl` and saves it
   with its comments and layout kept. Without it, the current version is
   packaged.
3. Lists the pack's files, leaving out what `.xsaignore` excludes.
4. Converts authored sources into their shipped form, such as terrain cube maps
   into streamed pages.
5. Writes both archives to the output directory and prints, for each, its path,
   size and SHA-256.

Archives are written in Rust (the `tar` and `zstd` crates), so packaging needs
nothing installed on the system. Compression uses zstd level 19 on every core:
slow to package, as fast as any level to extract. Entries are sorted, with a
fixed modification time, owner and permissions, so the same files always
produce the same archive.

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
but belong in its archives. `.git/` and `.xsaignore` itself are always left
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
        archive "data" url="https://github.com/afrigon/xsa/releases/download/system-solar-v0.0.1/system-solar-v0.0.1-data.tar.zst" size=1048576 sha256="…"
        archive "resources" url="https://github.com/afrigon/xsa/releases/download/system-solar-v0.0.1/system-solar-v0.0.1-resources.tar.zst" size=268435456 sha256="…"
    }
}
```

- **`pack`:** written by hand once per pack: its description, homepage and
  license.
- **`release`:** written only by the repository's tool. `xsa-repository
  add-release <url>…` downloads the archives, checks that each holds one
  top-level directory named after the pack, reads the manifest, and writes the
  entry: the version, the dependencies and `provides` copied from the manifest
  (so resolution never downloads an archive), and each archive's size and
  SHA-256 computed from the download. Hashes are never taken from an author.
- **`yanked=#true`** on a release: no longer chosen by resolution, still
  installed by lockfiles that already name it.

## Instances

Instances live in `$HOME/xsa/instances/`; the default is `main`.

```
$HOME/xsa/instances/main/
  launchpad.kdl     the instance's manifest
  launchpad.lock    resolved releases
  config.kdl        overrides of the global game config, optional
  packs/<pack>/     installed packs
  saves/
```

`launchpad.kdl` lists the packs the player asked for:

```kdl
format 1

packs {
    system-solar "^0.0.1"
}
```

`base` is an implicit dependency of every pack and of every instance:
manifests never list it, and an instance without it is completed before
anything else runs.

### Lockfile

`launchpad.lock` records, for every pack in the resolved set, the exact
version and each archive's URL and SHA-256. It works like Cargo's:

- It is written by resolution and only read otherwise.
- Resolution runs when `launchpad.kdl` changes, keeping every locked release
  that still satisfies it, or on `update`, which picks the latest matching
  releases.
- Each pack id resolves to one release: requirements are Cargo-style semantic
  version requirements, and two requirements no release satisfies together
  fail the resolution. A dependency on a provided name (`provides`, see
  `docs/packs.md`) is met by the one pack providing it.

### Sync

Syncing makes `packs/` match the lockfile. A missing or outdated pack is
downloaded into the archive cache (`$XDG_CACHE_HOME/xsa/archives/`), verified
against its SHA-256, extracted into a staging directory inside the instance,
then moved in place of the old `packs/<pack>/` as a whole. A failed download or
mismatched hash leaves the installed pack untouched. Installing replaces a
pack's directory entirely; packs are never edited in place.

## Loading packs

The game loads packs from its search path, then from its instance's `packs/`.
`--packs <directory>` adds directories to the search path, separated by the
platform's path separator (`:` on Linux, `;` on Windows), like `$PATH`; for each
pack id, the first directory containing that pack wins. Development builds run
with `--packs data`, so the repository's packs take precedence over installed
ones.

When the instance does not exist, the game creates it as `xsa-launchpad`
would, with the first-party packs in its manifest, and syncs it before loading.
Once synced, an instance runs without a network connection.

## Config

The game's config (`$XDG_CONFIG_HOME/xsa/config.kdl`) applies to every
instance; an instance's `config.kdl` overrides individual keys.

## Commands

`xsa-launchpad` acts on the instance in the current directory, or on
`--instance <name>`.

| Command | Does |
| --- | --- |
| `instance create <name>` | Creates an instance with `base` and syncs it |
| `instance list` | Lists the instances |
| `install <pack>[@<requirement>]` | Adds the pack to `launchpad.kdl`, resolves and syncs |
| `uninstall <pack>` | Removes the pack from `launchpad.kdl`, resolves and syncs |
| `update [<pack>]` | Resolves to the latest matching releases and syncs |
| `sync` | Makes `packs/` match the lockfile |
| `package <pack directory>` | Packages a pack for release (see Packaging) |

Publishing is not a launchpad command: an author uploads archives to their own
host and opens a pull request on the repository.

## Crates

| Crate | Responsibility |
| --- | --- |
| `packs` | The manifest (`PackManifest`), with `base` implicit |
| `repository` | Reads the index, downloads archives and verifies them, the archive cache |
| `instance` | Instance layout, `launchpad.kdl`, the lockfile and resolution, sync |
| `launchpad` | The `xsa-launchpad` binary: commands and packaging |

Dependencies point one way: `packs` ← `repository` ← `instance` ← `launchpad`,
`xsa`. `launchpad` follows the architecture of
[mc](https://github.com/afrigon/mc): command handlers stay thin and call
operations that hold the logic, a context carries the output shell and the HTTP
client, and manifest edits keep the document's layout.

## Open questions

- Whether the dedicated server runs from an instance, and whether its identity
  moves into it.
- `xsa client --instance` already names a running client for `xsa ipc`; the
  game's instance needs another flag, or the two concepts merge.
- Where launchpad's own settings live (extra repositories, the instances
  directory).
- The default output directory of `package`.
- Which first-party packs a new instance gets besides `base`.
