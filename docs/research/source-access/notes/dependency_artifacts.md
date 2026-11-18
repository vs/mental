# Dependency artifacts as file sources

Scope: how analysis tools get, check and mount the source files of package-level dependencies, and what that means for one read-only file layer in `mentalc` that also serves git commits and zip archives. As of 2026-11-16.

Evidence labels: no label = read in the primary source (docs, spec or source code; "local" = seen on this machine's caches). **snippet-only** = known only from a search summary. **unverified** = prior knowledge, not checked this session.

Fit with `docs/resolution.md`: R3 (execute nothing), the rungs in section 6, and E4 (how much library surface to read). Where this note would change those rules, it says **[rule change]**.

## Summary: one line per format or tool

Columns: what it is / can its files be read without running anything? / what identifies it / take for `mentalc`.

### Formats

- **Python wheel (`.whl`)**: a zip holding one distribution laid out as installed, with `.dist-info/{METADATA,WHEEL,RECORD}` / **yes** / the sha256 of the archive in the lockfile, plus sha256 for every file in `RECORD` / first-class source; show `*.data/purelib|platlib` merged at the root.
- **Python sdist (`.tar.gz`)**: a source tree plus `PKG-INFO` / **files yes; installed layout no**: which modules a build produces is up to the build backend; `PKG-INFO` fields can be trusted only at Metadata ≥ 2.2 and when not `Dynamic` / sha256 of the archive / read the files, but a module map inferred from an sdist is a weaker claim, and generated files are a gap.
- **Installed `site-packages`**: a directory tree with `*.dist-info/RECORD` per distribution / **yes** (as data; `.pth` path lines can be read as data, see ty) / per-file `RECORD` hashes; **no archive hash** / rung 2 as today; build the file set from `RECORD`, not by walking the directory.
- **npm tarball (`.tgz`)**: a gzip tar whose files sit under one top directory, usually `package/` / **files yes**; install scripts may add files / SRI `integrity` (sha512) of the tarball / strip the first path component by position, not by name.
- **Rust `.crate`**: a gzip tar with a normalized `Cargo.toml` and `Cargo.toml.orig` / **files yes**; `build.rs` and proc-macro output are missing / `cksum` = sha256 of the `.crate` / good source; generated code is a gap.
- **Go module zip**: a zip with the prefix `module@version/` and strict path rules / **yes**; Go has no install hooks / `h1:` = sha256 over a sorted list of file hashes (a **tree hash**) / the best-specified format; verify directly against `go.sum`.
- **Maven jar and `-sources.jar`**: zips of class files, and of source files / the sources jar yes; Central requires one but **allows placeholders** / `.sha1` and `.md5` required, `.sha256` optional / use only the sources jar; the class jar holds bytecode.
- **RubyGems `.gem`**: a plain tar of `metadata.gz`, `data.tar.gz` and `checksums.yaml.gz` / files yes; native extensions build at install / sha256 and sha512 of the inner members (snippet-only) / mount as a tar nested in a tar.

### Caches and stores

- **uv cache**: versioned buckets (`wheels-v8`, `archive-v0`, `files-v0`, ...); unpacked wheels; hardlinks / yes / an internal random id, plus BLAKE3 per file / readable, but the format is internal: verify by hash and never rely on its paths.
- **pip cache**: HTTP cache (`http-v2`) and built wheels / yes / none usable / "an implementation detail" that "may change between any two versions": do not read it.
- **Cargo `registry/cache` and `registry/src`**: `.crate` files and their unpacked trees / yes / `.crate` sha256 = the `Cargo.lock` checksum / readable if verified; Cargo says the layout is "not stabilized".
- **Go `GOMODCACHE`**: `cache/download/.../@v/*.zip|.mod|.info|.ziphash` plus read-only extracted trees / yes / `go.sum` `h1:` / **the only documented cache that is also a protocol** (it can serve as a `file://` GOPROXY): safe to read.
- **npm `_cacache`**: content-addressed by SRI (`content-v2`, `index-v5`) / yes / the SRI digest / addressable by the lockfile's `integrity`; the on-disk format is not documented.
- **pnpm store and `node_modules/.pnpm`**: hardlinks from a content-addressed store / yes / per-file hashes / read through `node_modules/.pnpm`; do not read the store.
- **Nix store**: fixed-output paths with NAR or flat hashes / yes / the NAR hash = a tree hash / the reference model for tree identity.
- **Bazel repository cache**: keyed by sha256, shared, never cleaned / yes / the archive sha256 / the reference for "archive hash plus `strip_prefix`".
- **typst package cache**: `{cache-dir}/typst/packages/preview/...` / yes / name and version only / a counter-example: no hash.

### Analysers

- **ty**: typeshed is a zip compiled into the binary; it reads `site-packages` found through `pyvenv.cfg` / does not execute for search paths / zip CRC32 as the file revision / the closest model for `mentalc`: a path enum over {system, vendored zip}.
- **rust-analyzer**: library source roots taken from `cargo metadata` / **runs cargo** / `is_local = source.is_none()` / copy the split into "local" and "library" roots, not the way it gets them.
- **CodeQL `build-mode: none`**: Java runs Gradle or Maven; C# restores NuGet; Rust runs build scripts / **executes and needs network** / not applicable / a counter-example under R3.
- **Sourcegraph package repos and SCIP**: a package becomes a repository; SCIP symbols carry `manager name version` / indexers run builds / `(manager, name, version)` / SCIP's package triple is a reasonable external key, but it carries no hash.

---

## Q1. Which artifacts can be read as files without executing anything?

### Takeaway

Every format listed can be *unpacked and read* without running code. The real question is whether **the unpacked files are the files a user's program sees**. That holds for wheels, Go module zips and Maven `-sources.jar`. It holds for installed `site-packages`, except for import hooks. It holds only partly for sdists, crates, npm tarballs and gems, because build backends, `build.rs`, install scripts and native extensions add or rearrange files at build or install time.

### Cited Findings

#### Python

- A wheel is "a ZIP archive with a `.whl` extension" laid out as it would be installed. `.dist-info` must hold METADATA, WHEEL and RECORD. Every file except RECORD must have a hash of sha256 or better in RECORD, and installers "verify all RECORD hashes" during extraction and fail on any unlisted or mis-hashed file — [Binary distribution format](https://packaging.python.org/en/latest/specifications/binary-distribution-format/)
- Installing a wheel means *unpacking* it (into purelib if `Root-Is-Purelib`, otherwise platlib) and then *spreading* it: `.data` subtrees move to their sysconfig paths, `#!python` scripts are rewritten, RECORD is updated and `.pyc` files are compiled. Installers must fail when the `Wheel-Version` major is higher than they support — [Binary distribution format](https://packaging.python.org/en/latest/specifications/binary-distribution-format/)
- Archivers are encouraged to put `.dist-info` at the end of the archive. Since Dec 2024 `scripts` may hold only regular files, and symlinks there are undefined behaviour. PEP 815 deprecated RECORD.jws and RECORD.p7s in Jan 2026 — [Binary distribution format](https://packaging.python.org/en/latest/specifications/binary-distribution-format/)
- `Dynamic` (Metadata 2.2, PEP 643): `Name`, `Version` and `Metadata-Version` may never be dynamic. A field in an sdist that is not marked `Dynamic` "MUST match" in any wheel built from that sdist. An sdist with metadata older than 2.2 must be treated as if every field were `Dynamic`. Outside sdists, `Dynamic` is only informational — [Core metadata](https://packaging.python.org/en/latest/specifications/core-metadata/)
- Core metadata is at version 2.6 (PEP 808, approved May 2026). In 2.6 a multiple-use `Dynamic` field "may only be appended to" in a wheel. Metadata 2.5 (PEP 794) added `Import-Name` (an import name the project provides exclusively) and `Import-Namespace` (one it provides but not exclusively). A name in both fields is an error — [Core metadata](https://packaging.python.org/en/latest/specifications/core-metadata/)
- Python resolvers build sdists to read their metadata (resolution.md F-U12) — [docs/resolution.md §12.2](../../../resolution.md)
- ty reads the `site-packages` of an environment found through `pyvenv.cfg`. It treats each line of a `.pth` file in `site-packages` as an extra search path for editable installs, in alphabetical order, as `site` does. Nothing is executed — [ty_site_packages/src/lib.rs](https://github.com/astral-sh/ruff/blob/main/crates/ty_site_packages/src/lib.rs); [ty_module_resolver/src/resolve.rs](https://github.com/astral-sh/ruff/blob/main/crates/ty_module_resolver/src/resolve.rs)
- ty's search path kinds are `Extra`, `FirstParty`, `StandardLibraryCustom`, `StandardLibraryVendored`, `StandardLibraryReal`, `SitePackages` and `Editable` — [ty_module_resolver/src/path.rs](https://github.com/astral-sh/ruff/blob/main/crates/ty_module_resolver/src/path.rs)

#### Rust

- `cargo package` writes a "compressed `.crate` file". It rewrites and normalizes `Cargo.toml`, dropping `[patch]`, `[replace]` and `[workspace]`, and keeps the original as `Cargo.toml.orig`. `Cargo.lock` "is always included". "Symlinks are flattened to their target files". `.cargo_vcs_info.json` records a git sha1, with the warning that "the provenance of the package is not verified" — [cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html)
- CodeQL's Rust none mode "uses `rust-analyzer` to compile and run build scripts (`build.rs` files) and compile macro code". So even a no-build analyser executes code to see generated Rust — [CodeQL build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages)

#### Go

- Symbolic links and other irregular files are not allowed in a module zip. Vendor directories and nested modules are excluded. Modes and timestamps are ignored — [Go modules reference: zip files](https://go.dev/ref/mod#zip-files); [golang.org/x/mod/zip](https://pkg.go.dev/golang.org/x/mod/zip)

#### Java

- Maven Central requires `<artifactId>-<version>-sources.jar` and `-javadoc.jar` for every jar, but "we allow uploading placeholder `.jar` files in order to pass validation". A sources jar can therefore be empty — [Sonatype Central requirements](https://central.sonatype.org/publish/requirements/)
- CodeQL Java none mode "attempts to run Gradle or Maven to extract accurate dependency information (but not to invoke a build)" and needs internet or a private artifact repository. C# none mode "restores dependencies" from NuGet — [CodeQL build options](https://docs.github.com/en/code-security/reference/code-scanning/codeql/build-options-for-compiled-languages). A 2024 issue reports that CodeQL 2.17 still ran the Maven build in none mode (snippet-only) — [github/codeql#16202](https://github.com/github/codeql/issues/16202)

#### JavaScript

- npm `package-lock.json` `integrity` is "a `sha512` or `sha1` Standard Subresource Integrity" string. `resolved` is the tarball URL, or for git dependencies "the full git url with commit sha" — [package-lock.json](https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json)
- npm tarballs normally put their files under `package/`, but some (e.g. an `@types/react` tarball) use another top directory, so robust extractors strip the first component by position (snippet-only) — [npm-utils install module](https://docs.rs/npm-utils/0.5.0/src/npm_utils/install/mod.rs.html)
- pnpm's `node_modules/.pnpm/<name>@<version>/node_modules/<name>` holds hard links into a content-addressable store, and dependencies are linked in as symlinks — [pnpm: symlinked node_modules structure](https://pnpm.io/symlinked-node-modules-structure)
- Deno keeps npm packages in a global cache by default and creates no `node_modules` (`nodeModulesDir: none`). In isolated layout each package sits in a content-addressed `.deno/` directory reached through a symlink — [Deno: Node and npm](https://docs.deno.com/runtime/fundamentals/node/)

#### Ruby

- A `.gem` is a plain tar of `data.tar.gz` (the files), `metadata.gz` (a YAML `Gem::Specification`) and `checksums.yaml.gz` (SHA-256 and SHA-512 of the other members), with optional signatures (snippet-only) — [Gem::Package](https://docs.ruby-lang.org/en/3.4/Gem/Package.html)

### Inferences

- **Readable without execution, faithful to what runs:** wheels (once the `.data` merge is applied), Go module zips, real `-sources.jar` files, and `site-packages` minus import-hook editables. Rung 4 (E4 option C) can rely on these.
- **Readable, but incomplete or rearranged:**
  - sdist: which modules exist and where they sit (`src/` or flat, `packages=` lists) is up to the backend (unverified). `_version.py` from setuptools-scm and Cython output are missing (unverified).
  - crate: `build.rs` output is missing.
  - npm: `postinstall` output is missing (unverified).
  - gem: native extensions are missing.

  `mentalc` can read these files, but each such tree should carry a "pre-build" mark, and imports into missing generated modules become gaps with a reason. This fits R3 and R7.
- For sdists, use `PKG-INFO` only when `Metadata-Version` ≥ 2.2 and the field is not in `Dynamic`. Name and version are always safe. `Import-Name` (2.5+) is the first standard way to map an import name to a distribution without running anything. It is better than `top_level.txt` and RECORD heuristics where present (rung 0 / §5.3 of resolution.md).
- **[rule change, small]** resolution.md §6.3 says "`.pth` files are not interpreted". ty shows a middle way: read the path lines of `.pth` as data and ignore the `import` lines. This adds no execution and recovers ordinary path-based editable installs. Proposal: path lines are read and ranked `observed`; an `import` line is a gap.

### Gaps

- No primary source was read for the npm tarball layout or RubyGems format; the npm `package/` convention and the `.gem` layout are snippet-only.
- How often PyPI sdists carry Metadata ≥ 2.2 with no `Dynamic` dependencies: no figure found.
- `deno_npm` and `deno_graph` internals, Glean's cross-repo package model: not reached.

---

## Q2. What minimal interface makes a wheel, a site-packages directory, a Go module zip and a git commit all look like one read-only tree of files?

### Takeaway

Existing tools converge on a small *read-only VFS*: list a directory, stat a path (file or dir only), read bytes, plus an identity per tree. It is backed either by a real directory or by an in-memory archive index; ty does exactly this for typeshed. Archive-specific work belongs in one *normalisation* step at mount time: strip the prefix, validate paths, reject or flag links, merge wheel `.data`, filter site-packages by RECORD. That way every backend presents the same clean relative-path namespace.

### Cited Findings

- ty packs typeshed into a zip at build time (`build.rs`, zstd or deflate, every entry mode 0o644) and embeds it with `include_bytes!`. It serves the zip through `VendoredFileSystem` (`exists`, `metadata`, `is_directory`, `is_file`, `read_to_string`, `read_directory`). Each file's revision is the zip entry's CRC32. The source commit of typeshed is pinned in `source_commit.txt` — [ty_vendored/build.rs](https://github.com/astral-sh/ruff/blob/main/crates/ty_vendored/build.rs); [ty_vendored/src/lib.rs](https://github.com/astral-sh/ruff/blob/main/crates/ty_vendored/src/lib.rs); [ruff_db/src/vendored.rs](https://github.com/astral-sh/ruff/blob/main/crates/ruff_db/src/vendored.rs)
- ruff_db's `FilePath` is an enum `System | SystemVirtual | Vendored`. Reads dispatch per variant (`FilePath::Vendored(v) => db.vendored().read_to_string(v)`) — [ruff_db/src/files.rs](https://github.com/astral-sh/ruff/blob/main/crates/ruff_db/src/files.rs)
- ruff_db's `System` trait mixes reading (`path_metadata`, `read_to_string`, `walk_directory`, `is_file`) with `which`, `run_command`, `env_var` and an optional `WritableSystem` — [ruff_db/src/system.rs](https://github.com/astral-sh/ruff/blob/main/crates/ruff_db/src/system.rs)
- rust-analyzer's `PackageRoot { is_local, include, exclude }` and `SourceRoot { is_library, file_set }`: "Libraries are considered mostly immutable", and "a file from one source root can't refer to a file in another source root by path". `is_local = source.is_none()` from `cargo metadata`, which it runs with `command.exec()` — [project-model/src/workspace.rs](https://github.com/rust-lang/rust-analyzer/blob/master/crates/project-model/src/workspace.rs); [base-db/src/input.rs](https://github.com/rust-lang/rust-analyzer/blob/master/crates/base-db/src/input.rs); [project-model/src/cargo_workspace.rs](https://github.com/rust-lang/rust-analyzer/blob/master/crates/project-model/src/cargo_workspace.rs)
- Bazel `http_archive` normalises at mount: `strip_prefix` ("Files outside that directory are discarded"; a mismatch is an error), or alternatively `strip_components`, and `add_prefix`. It supports zip, jar, whl, tar.{gz,xz,zst,bz2}, 7z, deb and more — [Bazel http rules](https://bazel.build/rules/lib/repo/http)
- Nix `fetchTarball`: "If the tarball consists of a single directory, then the top-level path component of the files in the tarball is removed" — [Nix builtins](https://nix.dev/manual/nix/latest/language/builtins.html)
- Go's module zip prefix is `$module@$version/`. A `go.mod` may appear only at the top. Unicode case-fold collisions and Windows reserved names are rejected — [Go modules reference: zip files](https://go.dev/ref/mod#zip-files)
- SCIP: `Metadata.project_root` plus `Document.relative_path`, and symbols of the form `<scheme> <manager> <package-name> <version> <descriptors>`. `Index.external_symbols` covers symbols "defined in an external package" — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)
- Sourcegraph package repositories sync "dependency sources" from npm, JVM, Rust, Ruby, Python and other hosts as repositories. They are discovered from code-graph uploads or listed by hand, and are marked "experimental" — [Sourcegraph package repos (5.7)](https://5.7.sourcegraph.com/admin/code_hosts/package-repos)

### Inferences

- **Proposed interface** (fits R3; it holds no process, environment or write methods, unlike ruff_db's `System`):
  - `Tree::list(dir) -> [(name, Kind)]`, with `Kind ∈ {File, Dir}`. Links are never exposed as links; see the archive-safety findings under Q4.
  - `Tree::read(path) -> bytes`
  - `Tree::stat(path) -> Option<(Kind, size)>`
  - `Tree::id() -> TreeId`, the tree hash (Q4)
  - `Tree::origin() -> Origin`: `{kind: git-commit | wheel | sdist | go-zip | crate | npm-tgz | jar | dir, locator (URL, path or commit), artifact digest(s), declared-by (lockfile path)}`

  Paths are relative, `/`-separated, valid UTF-8, with no `.`, `..`, empty or NUL segment.
- **Backends:**
  - (a) a directory, opened read-only and restricted to an allow-list;
  - (b) a zip, indexed through its central directory with no extraction, as ty does;
  - (c) a tar or tar.gz, read in one sequential pass into an index of offsets or an in-memory or cached blob map; gzip cannot seek, so the first read pays for decompression;
  - (d) git objects, a tree id walked through the object database without running git, in line with §6.1 of resolution.md;
  - (e) nested, for `.gem` = tar inside tar.
- **Mount transforms** applied once, so every tree looks alike:
  - wheel: merge `*.data/purelib` and `*.data/platlib` into the root; keep the rest of `.data` out of the import namespace;
  - Go: strip `module@version/`;
  - npm, crate, sdist: strip the single top directory by position;
  - Bazel style: an optional `strip_prefix` or `subdirectory`, matching pylock's `subdirectory`;
  - `site-packages`: the file set of one distribution = its `RECORD` entries, resolved against the `site-packages` root. This splits one shared directory into one tree per distribution and leaves out stray files.
- Import resolution across trees follows rust-analyzer's rule: each tree is a root, roots never reach each other by path, and "library" roots are immutable. This matches Mental's import roots in §6.1 of resolution.md: a dependency tree is just another import root with an `origin`.

### Gaps

- No source found on how SCIP or Glean join an indexed dependency back to a hash-verified artifact. SCIP's `Package` is only `manager/name/version`.

---

## Q3. Which caches are stable enough to read, and which are explicitly unstable?

### Takeaway

Only Go's module download cache is documented *as an interface*: it follows the GOPROXY protocol and can be served as a proxy. Bazel's repository cache is documented but tied to Bazel. Cargo calls its home "not stabilized", pip calls its cache "an implementation detail", and uv's buckets are versioned internals. The way out: **a cache is safe to read when every artifact taken from it is verified against a hash in a committed lockfile.** Instability then only affects *finding* the file, not trusting it.

### Cited Findings

#### Go and Bazel: documented

- Go: `$GOMODCACHE` (default `$GOPATH/pkg/mod`) holds `$module@$version/` (extracted, read-only) and `cache/download/$module/@v/{list,$v.info,$v.mod,$v.zip,$v.ziphash}`. The download directory "may be used as a proxy when served by an HTTP file server or when referenced with a `file://` URL". Module paths are case-encoded with `!` (`Azure` → `!azure`). Only `list` and `.info` "may change over time" — [Go modules reference: module cache](https://go.dev/ref/mod#module-cache); [GOPROXY protocol](https://go.dev/ref/mod#goproxy-protocol)
- Bazel repository cache: `~/.cache/bazel/_bazel_$USER/cache/repos/v1/`, "shared between all workspaces and installed versions of bazel". An entry is used only "if the download request has a SHA256 sum of the file specified and a file with that hash is in the cache". "The cache is never cleaned up automatically" — [Bazel: repository cache](https://bazel.build/run/build#repository-cache). By default a file is taken from cache only under the same `canonical_id` (the URLs) — [Bazel http rules](https://bazel.build/rules/lib/repo/http)

#### Cargo and pip: declared unstable

- Cargo: "the internal structure of the Cargo home is not stabilized and may be subject to change at any time". `registry/cache` holds `.crate` gzip archives and `registry/src` their unpacked trees — [Cargo home](https://doc.rust-lang.org/cargo/guide/cargo-home.html). Local: `~/.cargo/registry/{cache,index,src}/index.crates.io-1949cf8c6b5b557f/<name>-<version>/`.
- Cargo's registry index `cksum` is "A SHA256 checksum of the `.crate` file" — [Registry index](https://doc.rust-lang.org/cargo/reference/registry-index.html). Vendored directory sources have `.cargo-checksum.json`, which "is not a security mechanism and does not protect against malicious changes" — [Source replacement](https://doc.rust-lang.org/cargo/reference/source-replacement.html). "Cargo treats vendored sources as read-only" — [cargo vendor](https://doc.rust-lang.org/cargo/commands/cargo-vendor.html)
- pip: the cache's filesystem structure "is considered to be an implementation detail" and "may change between any two versions of pip". The HTTP cache is in `http-v2` since pip 23.3 — [pip caching](https://pip.pypa.io/en/stable/topics/caching/)

#### uv: versioned internals

- uv: versioned buckets. "Within a cache version, changes are guaranteed to be both forwards- and backwards-compatible". "uv's cache supports multiple concurrent readers and writers". Git dependencies are keyed by the resolved commit hash; URL dependencies by HTTP headers and the URL — [uv cache](https://docs.astral.sh/uv/concepts/cache/)
- uv bucket names in source today are `wheels-v8`, `sdists-v11`, `simple-v27`, `git-v2`, `archive-v0`, `files-v0` and `builds-v0`. `Archive` is "A cache of unzipped wheels, stored as directories"; other buckets symlink into it so entries "can be atomically replaced". `Files` holds "Content-addressed files that are hardlinked into cached archives" — [uv-cache/src/lib.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-cache/src/lib.rs). Local: `~/.cache/uv/wheels-v7/pypi/<pkg>/0.8.0-py3-none-any -> ../../../archive-v0/UZf-5_G3XdP8ldju`, with `wheels-v7` and `sdists-v9` beside `sdists-v10`. Bucket versions differ between installed uv releases.
- uv PR #21327 (merged 2026-08-31) moved deduplication from whole wheels ("if you download the same wheel twice from different sources, they share a cache entry") to single files stored under their BLAKE3 hash in `files-v0` and hardlinked into `archive-v0`. It saved about 545 MiB, roughly 10% of one cache — [astral-sh/uv#21327](https://github.com/astral-sh/uv/pull/21327)

#### npm and typst: content-addressed but undocumented, or unhashed

- npm cacache: "a Node.js library for managing local key and content address caches", addressed by SRI `<hashAlgorithm>-<base64-hash>`. `get.byDigest` looks up "by integrity hash, bypassing the index entirely". `verify` garbage-collects and re-checks integrity. The page makes no statement on the stability of the on-disk format — [cacache README](https://github.com/npm/cacache/blob/main/README.md). Local: `~/.npm/_cacache/{content-v2,index-v5,tmp}`.
- typst: packages are "cached in `{cache-dir}/typst/packages/preview`", and "Importing a cached package does not result in network access". Local packages in data-dir take precedence. The page warns that "future iterations ... may change/break this local setup" — [typst/packages README](https://github.com/typst/packages/blob/main/README.md)

### Inferences

Ranked by how safe each cache is to read:

1. **Go `cache/download/*.zip`**: documented protocol, verified by `go.sum` `h1:`. Read it.
2. **Cargo `registry/cache/*/<name>-<version>.crate`**: undocumented path, but bytes verifiable against `Cargo.lock` `checksum`. Read the `.crate`, not `registry/src`, which carries no hash and can be edited.
3. **npm `_cacache`**: findable by the lockfile `integrity` (content addressing). Reading it means re-implementing cacache's layout (`content-v2/<algo>/<hex…>`; unverified) and verifying the SRI.
4. **uv `archive-v0`**: unpacked trees with no archive hash. They cannot be verified against `uv.lock`'s sha256, only against each wheel's own RECORD. That rules out `declared` rank. Treat it like a `site-packages`: `observed`.
5. **pip cache**: no. **Bazel and Nix stores**: only when the project itself uses Bazel or Nix.

- **[rule change]** §6.3 of resolution.md says "`mentalc` does not search the machine". Reading `GOMODCACHE`, `~/.cargo/registry/cache` or `~/.npm/_cacache` is new. Proposal: a new sub-rung "verified local artifact". Locations come from `mentalc.toml` or a fixed list of documented defaults. An artifact counts only when its bytes match a hash from a rung-1 lockfile, and it then ranks `declared`, not `observed`, because the bytes are exactly what the lock names. No network and no execution, so R3 holds. This gives rung 4's benefit (E4 option C) without its network edge, whenever the developer already has the cache.
- **Determinism (G8):** record the artifact digest and tree id with the build, never the cache path.

### Gaps

- No primary statement read on npm `_cacache`'s on-disk format (`content-v2`, `index-v5`) or its stability.
- The Maven `~/.m2/repository/<group path>/<artifact>/<version>/` layout and its `.sha1` sidecar files were not checked this session (unverified).
- No source read on pnpm store versioning (`v3`, `v10`; unverified).

---

## Q4. How should a dependency tree be identified in a model so that it is stable and verifiable?

### Takeaway

Use **two digests**:

- an **artifact digest**: the bytes the lockfile pins (sha256 of a wheel, sdist or `.crate`; SRI of an npm tarball; a git commit id), which proves *where the tree came from*;
- a **tree digest**: a canonical hash over normalised relative paths and file contents (Go `h1:`, uv dirhash and Nix NAR are all of this kind), which names *what the model analysed*.

The tree digest keys the dependency in the model. It stays the same when the same files arrive as a wheel, an unpacked cache directory, a vendored copy or a different archive. The artifact digest keeps it verifiable against rung 1.

### Cited Findings

#### What lockfiles pin

- pylock: `hashes` are required for archives, sdists and wheels ("MUST contain at least one entry"; sha256 recommended), with optional `size` and `upload-time`. VCS entries require `commit-id`. `directory` entries have no hashes. Installers "Validate using ... size and ... hashes" — [pylock.toml spec](https://packaging.python.org/en/latest/specifications/pylock-toml/)
- `uv.lock` (local sample, `version = 1`, `revision = 5`): `sdist = { url, hash = "sha256:…", size, upload-time }`. uv guarantees compatibility within one schema `version` — [uv lockfile versioning](https://docs.astral.sh/uv/concepts/resolution/#lockfile-versioning)

#### Hashes in indexes and proxies

- PEP 691 JSON `files[].hashes` maps a hash name to a hex digest and "MUST be present, even if no hashes are available". The API says it "is up to the client to decide what to do with multiple hashes". `core-metadata` (PEP 658, renamed by PEP 714 from `dist-info-metadata`) announces `{file_url}.metadata`, ideally with its own hashes. `size` is mandatory and `provenance` arrived in API 1.3. Clients "MUST hard fail" on a higher major `api-version` — [Simple repository API](https://packaging.python.org/en/latest/specifications/simple-repository-api/)
- Cargo registry index `dl` templates may embed `{sha256-checksum}`, so a download URL can be content-addressed — [Registry index](https://doc.rust-lang.org/cargo/reference/registry-index.html)

#### Tree hashes

- Go `h1:` = base64 of the SHA-256 of the output of `sha256sum $(find . -type f | sort) | sha256sum`: one line per file of "hex SHA-256, two spaces, the file name, newline", sorted by name, with file names containing newlines disallowed. `HashZip` ignores compression, timestamps and other zip metadata, so a zip and its extracted directory hash the same — [dirhash](https://pkg.go.dev/golang.org/x/mod/sumdb/dirhash). A go.sum line covers either the zip contents or `…/go.mod` only. `h1` is "currently the only supported algorithm" — [Go modules reference: go.sum](https://go.dev/ref/mod#go-sum-files)
- `sum.golang.org` is a transparent log. Clients check inclusion and consistency proofs before trusting `/lookup` data — [Go modules reference: checksum database](https://go.dev/ref/mod#checksum-database)
- uv dirhash: BLAKE3. A directory hash is `derive_key("directory", …)` over sorted `name ‖ 0xff ‖ child-hash` entries. Empty directories are kept, symlinks are hashed as their targets, and *only names and contents* count, not the executable bit. This gives "the property that the dirhash of an archive is the same as the dirhash of its unpacked files, even if the archive was prepared on Unix and unpacked on Windows" — [uv-extract/src/dirhash.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-extract/src/dirhash.rs). uv uses it for extracted wheels and RECORD repair — [uv-distribution/src/extracted_wheel.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-distribution/src/extracted_wheel.rs)
- Nix content addressing: *flat* (file bytes, matches `sha256sum`), *NAR* (a canonical serialisation of the whole tree, designed because TAR and ZIP "store more information than we have", such as timestamps) and *git* (experimental; Merkle blobs and trees) — [Nix: content address](https://nix.dev/manual/nix/latest/store/file-system-object/content-address). `builtins.path` uses NAR by default and flat with `recursive = false`. `fetchTree` returns the NAR hash for reuse — [Nix builtins](https://nix.dev/manual/nix/latest/language/builtins.html)
- nixpkgs `fetchzip`'s hash is of the *extracted* content while upstream checksums are of the tarball, so they never match. Changing `stripRoot` changes the hash (snippet-only) — [NixOS Discourse: using upstream hashes](https://discourse.nixos.org/t/using-upstream-hashes/50783)
- SWHID core: `cnt` = git blob SHA-1, `dir` = git tree SHA-1. Modes are normalised to 100644, 100755, 120000 and 40000. Git compatibility is "practical, but incidental" — [SWHID v1.2 core identifiers](https://www.swhid.org/specification/v1.2/5.Core_identifiers/)

#### Weak or unhashed identities

- Bazel: `sha256` or `integrity` is over the downloaded archive; omitting it is "a security risk" and makes the build non-hermetic — [Bazel http rules](https://bazel.build/rules/lib/repo/http)
- Cargo's `.cargo_vcs_info.json` commit is unverified provenance — [cargo package](https://doc.rust-lang.org/cargo/commands/cargo-package.html)
- SCIP's package identity is only `(manager, name, version)` — [scip.proto](https://github.com/sourcegraph/scip/blob/main/scip.proto)

### Inferences

#### Why two digests

- One tree arrives under different artifact hashes: wheel vs `archive-v0` directory vs `site-packages`, a `.crate` vs `registry/src`, a GitHub tarball vs a git commit. Only a tree digest makes "the model of `requests` 2.32.3" the same node whichever way it was mounted. It also lets the model cache key off what was analysed. Conversely, one artifact name can carry different trees (a re-uploaded tarball, mutable GitHub archives (unverified)); the artifact digest catches that.

**Proposed rule for `mentalc`**

- `TreeId = h1-style sha256` over sorted `hex(sha256(content))  relpath\n`, after the mount transforms of Q2 (prefix stripped, `.data` merged). Modes and symlinks are excluded; symlinks are refused at mount (see the safety findings). Reasons:
  - (a) It is a specified, simple algorithm with test vectors in Go.
  - (b) For Go, re-adding the `module@version/` prefix reproduces exactly the `go.sum` value, so the same code verifies Go pins.
  - (c) It is the same for a zip, a tar and a directory.
  - (d) It needs no BLAKE3 dependency.

  SWHID `dir` would interoperate with Software Heritage, but it is SHA-1 and includes modes, so a wheel and its installed copy could differ. It is better as an *optional* extra identifier for git-sourced trees.
- `Origin` = `{artifact digests as the lockfile spells them (sha256:…, sha512-… SRI, crate cksum, git commit), URL or path, size, lockfile path and line}`.
- **Verification by rung:**
  - Rung 1 + local artifact: recompute the artifact digest → `declared`.
  - Rung 2 site-packages: check each file against RECORD; no archive digest exists, so the TreeId is recorded and ranked `observed`. Files that RECORD lists without a hash (RECORD itself, `.pyc`, entry-point scripts) are excluded from the tree.
  - Go: the TreeId check *is* the `go.sum` check.
- **[rule change]** §7.2 of resolution.md (pins) should gain a `tree` field next to version and source, so packs and gaps can name the exact analysed tree. Today a pin names only a version, a commit or an artifact.

### Gaps

- The Bazel REAPI Merkle `Digest` of directories (relevant to the brief's "Bazel Merkle digest") was not read this session (unverified).
- Whether PyPI re-hashes or mutates files after upload is not covered here; PyPI files are generally immutable (unverified).

---

### Additional findings: fetching (rung 4) and archive safety

These belong to the brief's coverage list rather than one key question. They are kept under Q4's heading level to respect the note format.

#### Cited Findings

##### Fetching for rung 4

- uv gets wheel metadata from PEP 658 `.metadata` files first. Failing that, it uses HTTP range requests for the zip central directory (the last ~16 KB, then the METADATA byte range), then a full download, then a source build (snippet-only) — [Simon Willison, Dec 2025](https://simonwillison.net/2025/Dec/26/http-range-wheel-metadata/). pip has the same `lazy_wheel` technique (snippet-only). A crafted zip could show different metadata in a range-read tail than in a full parse (snippet-only) — [discuss.python.org thread](https://discuss.python.org/t/fetch-zip-files-with-http-range-requests-for-wheel-metadata/3834)

##### uv's extraction checks

- Member names go through an `enclosed_name`-style check, "ensure that it cannot escape the extraction root" — [uv-extract/src/archive_path.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-extract/src/archive_path.rs)
- Errors for `DuplicateOutputPath`, `DuplicateLocalFileHeader`, local-vs-central-directory size conflicts, and bad compressed or uncompressed sizes — [uv-extract/src/error.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-extract/src/error.rs)
- "ZIP entries are never followed as symlinks; non-directory entries are materialized and hashed as regular files" — [uv-extract/src/stream.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-extract/src/stream.rs)
- For tar it allows hard links, because old sdists contain them (`pybabel-json==0.2.0`), and skips symlinks on Windows — [uv-extract/src/stream.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-extract/src/stream.rs)

##### Go's checks

- Go's `Unzip` checks every restriction: declared sizes must match, the zip may be at most 500 MiB, total uncompressed content at most 500 MiB, `go.mod` and `LICENSE` at most 16 MiB each, and the target directory must be empty. Files may already be written before an error — [golang.org/x/mod/zip](https://pkg.go.dev/golang.org/x/mod/zip); [Go modules reference](https://go.dev/ref/mod#zip-files)

##### The wheel spec's silence

- The wheel spec does not address absolute paths, `..`, symlinks in general, duplicate entries or compression; only `scripts` is limited to regular files — [Binary distribution format](https://packaging.python.org/en/latest/specifications/binary-distribution-format/)

##### Python's tarfile filters

- PEP 706 tarfile filters:
  - `'data'` refuses absolute paths, paths outside the destination, links to absolute paths or outside it, and device files;
  - it is the default from Python 3.14;
  - it does *not* address DoS: size and file-count limits are left to callers, and the PEP says tarfile is "not suited for extracting untrusted files without prior inspection".
  - It cites CVE-2007-4559.
  - [PEP 706](https://peps.python.org/pep-0706/)

##### Zip Slip

- Zip Slip: extraction code that does not validate member paths. A two-entry variant first writes a symlink pointing outside, then a regular file at the same name, and bypassed earlier fixes in `decompress` and `extract-zip` (snippet-only) — [Snyk: extract-zip](https://security.snyk.io/vuln/SNYK-JS-EXTRACTZIP-19653641); [Snyk: decompress](https://security.snyk.io/vuln/SNYK-JS-DECOMPRESS-16415209)

#### Inferences

##### Partial reads cannot be verified

- A range read or a PEP 658 `.metadata` file cannot be checked against the lockfile's *archive* hash. The `.metadata` file has its own optional hash in the index, not in the lockfile. So rung 4 must download the **whole** artifact and verify it before any of its content counts as `declared`. PEP 658 and range reads are fine for discovery, not evidence (fits R4). This matters for E4 option C's cost estimate.

##### Mounting never extracts

- `mentalc` never extracts to disk: a mount reads the archive index into memory, as ty does. That removes the write-side Zip Slip class entirely. Path validation is still needed so that two names cannot alias in the model.

##### Rules to apply at mount, in one place for all formats

- Reject absolute paths, `..`, empty segments, NUL, backslash-as-separator ambiguity and non-UTF-8.
- Duplicate member names are an error; uv does the same.
- Case-fold collisions are a warning everywhere and an error for Go, where they are forbidden.
- Symlinks and hard links in tar: refuse links that point outside the tree. Inside the tree, either resolve as an alias (Cargo flattens them at packaging) or record a gap. Never expose them as links.
- Zip local-header vs central-directory disagreement is an error, as in uv.
- Size caps per file, per tree and per entry count, modelled on Go's 500 MiB and 16 MiB limits.
- Decompression ratio cap against zip bombs (unverified as a practice in any tool read here).

##### R3 and network

- Nothing here needs execution. R3 holds as written.
- The only proposed changes are:
  - the verified-local-artifact sub-rung (Q3);
  - reading `.pth` path lines as data (Q1);
  - a tree digest on pins (Q4);
  - and in rung 4, "fetch" meaning a full, hash-verified download.

#### Gaps

- No primary source read on zip bombs or decompression-ratio limits in uv, ty or Go's tooling.
- How GitHub-generated archive tarballs change bytes over time was not checked (unverified).
