# How mentalc should read files from git commits (including submodules)

Research date: 2026-11-16. Evidence labels: unlabelled = read at the source (docs, source code in the crates.io tarball, advisory text, or my own `cargo tree` run on a scratch manifest I wrote); "snippet-only" = from a search summary; "unverified" = prior knowledge, not checked in this session. Weights below are crate counts from `cargo tree -e normal,build` on scratch manifests (nothing built), so compile time itself is a gap.

## Summary, one line per option

- **gix (gitoxide) 0.90.0, `default-features = false`** — pure-Rust git library, by far the most complete Rust reader. Weight: 88 crates with `sha1` only, 92 with `sha1,revision` (needed for `rev_parse_single`), 104 with `attributes` added (needed for `gix::Submodule`), 134 with defaults. Safety: `open::Options::isolated()` reads no system/user config and no environment, never spawns `git`, runs no hooks, filters only if asked; 2026 saw a stream of advisories on submodule names, symlinked `.gitmodules`, commit-graph and pack DoS, all fixed by 0.90.0. **Take: recommended, but used as a thin object reader with mentalc's own tree walker and `.gitmodules` parser, not through `gix::Submodule` or checkout APIs.**
- **gix plumbing crates (`gix-odb` + `gix-pack` + `gix-object` + `gix-ref` + `gix-hash` + `gix-validate`)** — the same code without the `gix` facade. Weight: 58 crates, and no `gix-config`, `gix-transport`, `gix-protocol` or `gix-command`. Safety: smallest attack surface, but mentalc must then handle repository discovery, `extensions.objectFormat`, `core.useReplaceRefs` and HEAD/ref peeling itself. **Take: the fallback if the 92-crate facade is too heavy for the allowlist. Prototype it in an experiment against the facade.**
- **git2 0.21.0 (libgit2 1.9.7 bindings)** — mature C library. Weight: 12 Rust crates, but it compiles about 124k lines of C with `cc` (C toolchain needed). SHA-256 only behind `unstable-sha256`, which is ABI-breaking and not thread-safe without OpenSSL. libgit2 had 2026 advisories for delta allocation DoS and submodule path traversal. **Take: not recommended. It breaks the pure-Rust, small-allowlist posture, and SHA-256 is experimental.**
- **Shelling out to `git` (`cat-file --batch`, `ls-tree`, `archive`)** — git is the reference implementation, with zero crate weight. Safety: git's own manual says running git inside an untrusted `.git` is unsafe. Repository-local config is always read and cannot be switched off wholesale, partial clones fetch missing objects lazily over the network, and it is a process start that section 12.4 forbids. Output depends on the user's git version. **Take: rejected. It violates R3 and F-U15 as written.**
- **Pure hand-written reader (loose objects + pack/idx + delta)** — Weight: `zlib-rs`/`miniz_oxide` + `sha1` + own code. Safety: small surface but all of git's sharp edges reimplemented (multi-pack-index, delta chains, alternates, SHA-256, reftable). **Take: not worth it. gix's plumbing crates already are this, audited and fuzzed.**

## Q1. Which library is recommended for read-only access with minimal features, and what is its true weight?

### Takeaway

Use `gix` 0.90 with `default-features = false, features = ["sha1", "sha256", "revision"]` (99 crates) or, to go lighter, the gix plumbing crates (58 crates). Do not enable `attributes` (it brings `gix-filter`, `gix-submodule`, `gix-command`) or any network feature. git2 has fewer Rust crates but compiles libgit2 from C and gates SHA-256 behind an unstable ABI flag.

### Cited Findings

#### Versions (as of 2026-11-16)

- gix 0.90.0 was released on 2026-10-22, git2 0.21.0 on 2026-10-06, and libgit2-sys 0.18.8+1.9.7 (bundling libgit2 1.9.7) on 2026-10-06. jj-lib is at 0.47.1 — [crates.io gix](https://crates.io/crates/gix), [git2](https://crates.io/crates/git2), [libgit2-sys](https://crates.io/crates/libgit2-sys), [jj-lib](https://crates.io/crates/jj-lib).
- libgit2 upstream released v1.9.7 on 2026-08-13 and v2.0.0-rc.1 on 2026-10-14 — [libgit2 releases](https://github.com/libgit2/libgit2/releases).

#### gix features

- The gix defaults are `max-performance, comfort, basic, extras, auto-chain-error, sha1`. Library developers are told to start from `default-features = false` and add components. "At least one hash algorithm must be enabled" (`sha1`, `sha256`) — [gix 0.90.0 Cargo.toml](https://docs.rs/crate/gix/0.90.0/source/Cargo.toml.orig).
- `revision` enables `gix-revision/describe`, `merge_base` and `index`. `Repository::rev_parse_single` is behind `#[cfg(feature = "revision")]` — [Cargo.toml](https://docs.rs/crate/gix/0.90.0/source/Cargo.toml.orig), [repository/revision.rs](https://docs.rs/crate/gix/0.90.0/source/src/repository/revision.rs).
- `attributes` pulls `gix-filter`, `gix-pathspec`, `gix-attributes`, `gix-submodule` and `command`. The `gix::submodule` module and `Repository::submodules()` are behind `#[cfg(feature = "attributes")]` — [Cargo.toml](https://docs.rs/crate/gix/0.90.0/source/Cargo.toml.orig), [lib.rs](https://docs.rs/crate/gix/0.90.0/source/src/lib.rs).
- Network features are opt-in: `blocking-network-client` and `async-network-client` "may be enabled" one at a time. `parallel` is in `max-performance` and is opt-in otherwise — [Cargo.toml](https://docs.rs/crate/gix/0.90.0/source/Cargo.toml.orig).

**Measured weight** (`cargo tree -e normal,build`, unique crates excluding the root, macOS, cargo 1.99.1; my own scratch manifests, nothing built)

| Dependency set | Crates |
|---|---|
| `gix` `sha1` only | 89 (88 normal) |
| `gix` `sha1,revision` | 93 |
| `gix` `sha1,sha256,revision` | 100 |
| `gix` `sha1,revision,attributes` | 105 |
| `gix` defaults | 134 |
| plumbing (`gix-odb`, `gix-object`, `gix-hash`/sha1, `gix-pack`/sha1, `gix-ref`, `gix-validate`) | 58 |
| `git2` `default-features = false` | 12 (`cc`, `pkg-config`, `vcpkg`, `jobserver`, `libz-sys`, `libgit2-sys`, `log`, `libc`, `bitflags`, …) |

All measured on 2026-11-16.

- Even `gix` with only `sha1` links `gix-config`, `gix-discover`, `gix-protocol`, `gix-transport` and `gix-command`. `gix-command` arrives via `gix-transport` ← `gix-protocol` ← `gix`. The plumbing set has none of these, but does include `gix-path` — measured `cargo tree -i` on the scratch manifest.
- Source sizes in the 0.90 tarballs: `gix` 33.5k lines of Rust, `gix-pack` 11.3k, `gix-config` 10.2k, `gix-ref` 6.4k, `gix-object` 6.3k, `gix-odb` 5.6k. libgit2's bundled `src/*.c` totals about 124k lines — measured `wc -l` on the [libgit2-sys 0.18.8 tarball](https://crates.io/crates/libgit2-sys).

#### git2 features

- git2 0.21 has default features `[]`. `https` brings OpenSSL and `ssh` brings libssh2. `unstable-sha256` "reflect[s] upstream libgit2's GIT_EXPERIMENTAL_SHA256. This is an ABI-breaking change… Library authors: DO NOT enable this feature by default", and "libgit2's builtin SHA256 implementation is not thread-safe" without the OpenSSL route — [git2 0.21.0 Cargo.toml](https://docs.rs/crate/git2/0.21.0/source/Cargo.toml.orig).

#### Read APIs

- gix read APIs exist as: `Repository::find_object`, `try_find_object`, `find_blob`, `rev_parse_single`; `Object::peel_to_commit`; `Commit::tree`; `Tree::decode`, `find_entry`, `lookup_entry_by_path`, `traverse()` — [repository/object.rs](https://docs.rs/crate/gix/0.90.0/source/src/repository/object.rs), [object/peel.rs](https://docs.rs/crate/gix/0.90.0/source/src/object/peel.rs), [object/commit.rs](https://docs.rs/crate/gix/0.90.0/source/src/object/commit.rs), [object/tree/mod.rs](https://docs.rs/crate/gix/0.90.0/source/src/object/tree/mod.rs).
- jj reads a file as `repo.try_find_object(id)?.ok_or(..)?.try_into_blob()?.take_data()`. That is the raw blob, with no filter — [jj git_backend.rs L512-L522](https://github.com/jj-vcs/jj/blob/4351061bba89b3c503d133170952f94a2c95cdfd/git-backend/src/git_backend.rs#L512-L522).

#### Maturity and object-store coverage

- jj uses `gix` 0.90 with `attributes, auto-chain-error, blob-diff, index, max-performance, sha1, sha256` — [jj Cargo.toml L62-L70](https://github.com/jj-vcs/jj/blob/4351061bba89b3c503d133170952f94a2c95cdfd/Cargo.toml#L62-L70). onefetch uses gix 0.90 with `max-performance, blob-diff, mailmap, index, status, sha1, anyhow` — [onefetch Cargo.toml](https://github.com/o2sh/onefetch/blob/main/Cargo.toml). git-cliff's core uses `git2` behind an optional `repo` feature — [git-cliff-core Cargo.toml](https://github.com/orhun/git-cliff/blob/main/git-cliff-core/Cargo.toml).
- gix's object store already supports: alternates ("safe with cycles and recursive configurations"), multi-pack indices, object replacements (`git replace`), and shallow clone/fetch — [gitoxide crate-status.md](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md).
- Still unchecked in that file: "Git 3.0 compatibility (`SHA-256`, `reftable`)" as an overall goal, and all promisor work: "fetch missing objects on demand from promisor remotes", "support … `extensions.partialClone`" — [crate-status.md](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md).
- gix 0.90 nevertheless parses `extensions.objectFormat` (repository format v1) to choose the object hash, so SHA-256 object reading is compiled in with the `sha256` feature — [config/cache/incubate.rs L51-L56](https://docs.rs/crate/gix/0.90.0/source/src/config/cache/incubate.rs).
- gix SHORTCOMINGS: packfiles are memory-mapped, which "hide[s] IO errors". On 32-bit, packs over 2^31/2^32 bytes and objects larger than addressable memory cannot be loaded. Object replacements are read once at open — [SHORTCOMINGS.md](https://github.com/GitoxideLabs/gitoxide/blob/main/SHORTCOMINGS.md).
- unverified: libgit2 supports shallow clones since 1.7 and does not support partial clone or promisor objects.

### Inferences

- The `gix` facade's extra ~35 crates over the plumbing set buy repository discovery, config layering with trust filtering, `objectFormat` and replace-refs handling, and rev-spec parsing. These are exactly the parts where getting it subtly wrong is a security or determinism bug. So the facade with `["sha1","sha256","revision"]` is the default recommendation, and the plumbing set is an experiment arm (it fits the "Experiment" status convention).
- mentalc's allowlist today has 9 normal crates. Any git option adds roughly 50 to 100 crates, nearly all from one project (GitoxideLabs). The `deps.allow` tidy test should allow the `gix-*` family as one justified entry rather than ~90 lines.
- Treat a missing object (partial clone, pruned alternate) as a nameable gap, never a fetch. gix has no promisor support, so the lookup simply returns not-found. That is the behaviour mentalc wants.

### Gaps

- Compile time and binary size were not measured, because building was out of scope. Crate counts are a proxy only.
- I did not verify that gix 0.90 rejects unknown `extensions.*` the way git does for format v1. I saw no such check in `incubate.rs`.
- I did not check reftable support status in gix 0.90 beyond the unchecked crate-status line.

## Q2. Can mentalc read git commits without violating "executes nothing the repository controls"? What must be turned off?

### Takeaway

Yes, as long as it reads the object database in-process with gix and never runs the `git` binary. git's warning is about git honoring commands in `.git/config` and hooks. Reading objects runs neither in gix, if mentalc opens with `Options::isolated()`, leaves `git_binary` off, enables no network, filter or checkout feature, sets `core.useReplaceRefs=false`, and treats `.git` pointers, alternates and `.gitmodules` as untrusted data. R3 and section 12 need a rewording, from "does not run git" to "does not run the git program; reads git objects as data". F-U15 still justifies the no-subprocess part.

### Cited Findings

#### What git's manual says

- git(1) SECURITY: "Some configuration options and hook files may cause Git to run arbitrary shell commands… it is generally safe to clone remote repositories with untrusted content, inspect them with git log… However, it is not safe to run Git commands in a .git directory (or the working tree that surrounds it) when that .git directory itself comes from an untrusted source." It also notes you can "acquire untrusted repositories that are owned by you (for example, if you extract a zip file)", so `safe.directory` ownership checks do not cover that case. The advice is to sanitize with `git clone --no-local` — [git(1) SECURITY](https://git-scm.com/docs/git#_security).
- Environment switches git offers include:
  - `GIT_NO_LAZY_FETCH`: "tells Git not to lazily fetch missing objects from the promisor remote on demand";
  - `GIT_CONFIG_NOSYSTEM`, and `GIT_CONFIG_GLOBAL=/dev/null`;
  - `GIT_NO_REPLACE_OBJECTS`;
  - `GIT_ALLOW_PROTOCOL`, which acts as `protocol.allow=never` plus listed exceptions;
  - `GIT_TERMINAL_PROMPT=0`.

  None of them disables repository-local config — [git(1) environment](https://git-scm.com/docs/git).
- `git cat-file --batch` prints `<oid> SP <type> SP <size> LF <contents> LF` and `<object> SP missing LF` for unresolvable names. `--filters` applies "smudge filters, end-of-line conversion, etc" and `--textconv` applies textconv drivers. Both are opt-in — [git-cat-file](https://git-scm.com/docs/git-cat-file).
- `git archive` honors `export-ignore` and `export-subst` from the archived tree's `.gitattributes` (or `--worktree-attributes`, or `$GIT_DIR/info/attributes`). It can pipe output through the `tar.<format>.command` shell command — [git-archive](https://git-scm.com/docs/git-archive).

#### What gix reads and what it can spawn

- gix `open::permissions::Config` has the fields `git_binary` ("may involve executing the git binary"), `system`, `git` (XDG), `user` (`~/.gitconfig`), `env` and `includes` (`include`/`includeIf`). `Config::isolated()` sets all of them to false. `Config::all()` also keeps `git_binary: false`. "Configuration from inside of the repository is always loaded as it's definitely required for correctness" — [open/permissions.rs](https://docs.rs/crate/gix/0.90.0/source/src/open/permissions.rs).
- `Attributes::isolated()` loads no system or XDG attribute files. `Environment::isolated()` denies `XDG_CONFIG_HOME`, `HOME`, `SSH_*`, `GIT_*`, HTTP proxy, identity and object-database variables. `Permissions::isolated()` combines the three. In 0.90, `Permissions::secure()` (the default) is identical to `all()` — [open/permissions.rs](https://docs.rs/crate/gix/0.90.0/source/src/open/permissions.rs).
- `open::Options::isolated()` is "configured to prevent accessing anything else than the repository configuration file, prohibiting accessing the environment or spreading beyond the git repository location". Other options:
  - `with(Trust)` overrides the ownership-derived trust;
  - `bail_if_untrusted(true)` "mimic[s] git's way of handling untrusted repositories";
  - `filter_config_section` drops by default repository-local sections that are not of `Trust::Full`;
  - `strict_config(true)` is "recommended for all applications that prefer correctness".

  Source: [open/options.rs](https://docs.rs/crate/gix/0.90.0/source/src/open/options.rs).
- `gix-sec` trust model: `Trust::{Reduced, Full}`, and `Trust::from_path_ownership(path)` returns `Full` only if the path is owned by the current user. `Permission::{Forbid, Deny, Allow}` — [gix-sec trust.rs](https://docs.rs/crate/gix-sec/0.16.1/source/src/trust.rs), [lib.rs](https://docs.rs/crate/gix-sec/0.16.1/source/src/lib.rs).
- Process-spawn sites in the crates the minimal `gix` links:
  - `gix-path` `env` (running `git` to find the installation config, and `shell_command`). This is used only when `git_binary` is enabled or the path is otherwise queried.
  - `gix` `commit/sign.rs` (signing), `repository/config/mod.rs` (editor, via `gix_command::prepare`).
  - `gix-transport` ssh/file, and `gix-protocol`.

  None of these is in the object-read path. `gix-odb`, `gix-object`, `gix-ref`, `gix-pack` and `gix-hash` contain no `Command::new` — measured `grep` over the 0.90 tarballs, e.g. [gix-path env/mod.rs](https://docs.rs/crate/gix-path/0.14.1/source/src/env/mod.rs).
- gix does not implement fsmonitor. The only occurrence is the sample hook template written on `init` — measured `grep fsmonitor` over [gix 0.90 src](https://docs.rs/crate/gix/0.90.0/source/src/create.rs).
- Replacement objects are controlled by `core.useReplaceRefs`, with env override `GIT_NO_REPLACE_OBJECTS` — [config/tree/sections/core.rs L103-L104](https://docs.rs/crate/gix/0.90.0/source/src/config/tree/sections/core.rs).
- `gix-odb` follows `<git-dir>/info/alternates` and resolves relative paths into canonical paths outside the repository — [gix-odb alternate/mod.rs](https://docs.rs/crate/gix-odb/0.87.0/source/src/alternate/mod.rs).
- jj opens gix with `open::Options::default()` plus `open_path_as_is(true)` and `strict_config(true)`, not `isolated()`: jj trusts the user's own repo — [jj git_backend.rs L617-L633](https://github.com/jj-vcs/jj/blob/4351061bba89b3c503d133170952f94a2c95cdfd/git-backend/src/git_backend.rs#L617-L633).

**gitoxide advisories relevant to reading untrusted repos** (from the [gitoxide security advisories](https://github.com/GitoxideLabs/gitoxide/security/advisories) list)

- [GHSA-q54r-jf2p-rfr2](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-q54r-jf2p-rfr2) (high, 2026-10-22, fixed in gix ≥ 0.90.0): `protect_options()` read `core.protectHFS`, `core.protectNTFS` and `gitoxide.core.protectWindows` with the unfiltered accessor. An untrusted `.git/config` could disable the `.git` path-confusion checks used by "checkout, index creation, merge, and tree editing". The trust filter is applied elsewhere "for diff drivers, merge drivers, URL rewrites, scheme permissions…".
- [GHSA-989c-xj77-266r](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-989c-xj77-266r) (medium, fixed in gix-commitgraph ≥ 0.42.0): a crafted `.git/objects/info/commit-graph` "makes every history-walking gix command abort the whole process" via `panic!`/`unwrap`.
- [GHSA-x494-mj8g-cj27](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-x494-mj8g-cj27) (high, fixed in gix-pack ≥ 0.69.0): unchecked indexing in delta application and uncapped size headers (panic/OOM) "triggered by malicious pack data received during clone/fetch". Further pack DoS advisories followed on 2026-09-25: GHSA-633h, GHSA-mpf5, GHSA-x862.
- [GHSA-fr8x-3vfx-f45h](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-fr8x-3vfx-f45h), [GHSA-p3hw-mv63-rf9w](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-p3hw-mv63-rf9w) (fixed in gix ≥ 0.82.0, gix-validate ≥ 0.11.1): unvalidated `.gitmodules` names joined to `<common_dir>/modules/<name>` escaped `.git/modules`. `gix_validate::submodule::name()` was bypassable (`a..b/../../../.git/`) and "has zero production callers". Combined with "trust inheritance" in `Submodule::open()`, this led to reading other repos' configs "including credentials".
- [GHSA-pg4w-g64p-qwhj](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-pg4w-g64p-qwhj): a worktree `.gitmodules` read with `std::fs::read` followed symlinks outside the repo.
- [GHSA-f26g-jm89-4g65](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-f26g-jm89-4g65): `.gitmodules` could bypass the `update = !command` trust check (cf. CVE-2019-19604).
- Checkout-only advisories (irrelevant if mentalc never checks out): CVE-2024-35186 traversal outside the worktree, CVE-2026-44471 symlink prefix reuse, GHSA-6p9q, GHSA-pmm9, CVE-2025-22620. Also CVE-2025-31130, "SHA-1 collision attacks are not detected", fixed by `sha1dc` in gix-features ≥ 0.41 — [advisories list](https://github.com/GitoxideLabs/gitoxide/security/advisories), [RustSec gix-features](https://rustsec.org/advisories/RUSTSEC-2025-0021.html).

#### libgit2 and git advisories

- [GHSA-27m5-gxxh-x79j](https://github.com/libgit2/libgit2/security/advisories/GHSA-27m5-gxxh-x79j) (CVE-2026-53585, fixed in 1.9.5 / 1.8.6): `git_delta_apply()` allocates the attacker-claimed result size before validating.
- [GHSA-cw77-j82w-mchm](https://github.com/libgit2/libgit2/security/advisories/GHSA-cw77-j82w-mchm) (CVE-2026-53584): a submodule path with `../` "can cause the library to create directories outside the repository's working tree".
- Older libgit2 advisories: CVE-2024-24577 (heap corruption in `git_index_add`) and CVE-2024-24575 (DoS in `git_revparse_single`) — [libgit2 advisories](https://github.com/libgit2/libgit2/security/advisories).
- git2 itself: RUSTSEC-2026-0008, -0183 and -0184 (UB in `Buf`, `Remote::list`, `BlameHunk`) — [RustSec advisory-db crates/git2](https://github.com/rustsec/advisory-db/tree/main/crates/git2).
- git2 exposes `opts::set_verify_owner_validation`, `opts::set_search_path(ConfigLevel, …)` and `set_extensions`. All are `unsafe fn` global settings — [git2 opts.rs](https://docs.rs/crate/git2/0.21.0/source/src/opts.rs).
- unverified: libgit2 reads global and system config by default, and does not execute external `filter.<driver>` commands, hooks or fsmonitor.
- CVE-2018-11235 (git): "submodule 'names' are obtained from this file [.gitmodules], and then appended to $GIT_DIR/modules, leading to directory traversal with '../' in a name. Finally, post-checkout hooks from a submodule are executed" — [NVD](https://nvd.nist.gov/vuln/detail/CVE-2018-11235).
- CVE-2024-32002 (git < 2.45.1 etc.): "repositories with submodules can be crafted… into writing files not into the submodule's worktree but into a .git/ directory. This allows writing a hook that will be executed while the clone operation is still running" (case-insensitive filesystem plus symlinks) — [NVD](https://nvd.nist.gov/vuln/detail/CVE-2024-32002).

**Git features that execute code or read ambient state** (unverified except where linked)

- Hooks (`.git/hooks/*`, `core.hooksPath`) run on commit, checkout, merge and similar.
- `core.fsmonitor` runs a program for index-reading commands such as `status`.
- `core.sshCommand`, `GIT_SSH`, `credential.helper` and `protocol.*.allow` apply on fetch/push. With a promisor remote, even a read can fetch — [GIT_NO_LAZY_FETCH](https://git-scm.com/docs/git).
- `filter.<d>.clean/smudge/process` from `.gitattributes` `filter=` (git-lfs is one such filter) apply on checkout and add, or `cat-file --filters`.
- `diff.<d>.textconv`/`command` and `diff.external` apply on diff, or `--textconv`.
- `core.pager`, `core.editor` and `sequence.editor` apply to interactive commands. `tar.<fmt>.command` applies to archive ([git-archive](https://git-scm.com/docs/git-archive)).
- `include.path`/`includeIf` pull in other config files. `safe.directory` is only an ownership check ([git(1)](https://git-scm.com/docs/git#_security)).

### Inferences

- **R3 compatibility.** Every code-running feature in the list above is either (a) a git porcelain behaviour gix does not implement (hooks on read, fsmonitor, pager, editor), (b) a gix feature mentalc does not compile in (network and credentials, `attributes`/`gix-filter` smudge, `worktree-*`, `command`), or (c) an API mentalc does not call (signing, editor). So "reading the object database with a library" does not run anything the repository controls. The one residual risk is that `gix-command` and `gix-transport` are linked in via `gix-protocol` even with minimal features. mentalc's own G4 lint cannot see into dependencies, so add a tidy test that bans calls to `gix::Repository::{remote*, find_remote, command_context, filter_pipeline, submodules}` and `gix::open::Options::with`. Alternatively, use the plumbing set, which links none of them.
- **What must be off or forced** (proposed `open` recipe; the key names and APIs were read, the combination is untested):
  - `gix::open::Options::isolated()`, which turns off system, user and XDG config, includes, env and git-binary.
  - `.strict_config(true)`.
  - Let trust be derived from ownership; do not call `.with(Trust::Full)`.
  - Override `core.useReplaceRefs=false`, so commit X means the object whose hash is X.
  - Override `core.commitGraph=false` for any ancestry walk such as `HEAD~3` (GHSA-989c class).
  - Do not read the worktree `.gitmodules` (GHSA-pg4w class).
  - Cap the object size read into memory (GHSA-x494 and libgit2 GHSA-27m5 class).
  - Run object decoding under `std::panic::catch_unwind` at the edge. A malformed repo then yields a typed error ("fail rather than degrade"), not an abort.
- **Repository-controlled indirection is a data-disclosure risk, not code execution.** These include `objects/info/alternates`, `.git` gitfiles (`gitdir: …`), `commondir` in linked worktrees, and submodule gitdirs. They can make mentalc read objects from elsewhere on the user's disk, for example a private repo, into a model that may be shared. Proposed rule: resolve each such pointer, require it to lie under the repository root (or under a root listed in `mentalc.toml`), and record every followed pointer in the build's provenance. git and gix both follow alternates unconditionally.
- **Shelling out to `git`** cannot be made equivalent. Repository-local config is always loaded, `cat-file` on a missing object in a partial clone fetches unless `GIT_NO_LAZY_FETCH=1`, results vary with the user's git version (determinism, G8-style), and section 12.4 allows only the compose renderer to be started. F-U15's conclusion stands for the binary, but not for in-process reading.

### Gaps

- I did not verify which repository-local config keys gix 0.90 honors in the read-only object path beyond `extensions.objectFormat`, `core.repositoryFormatVersion`, `core.useReplaceRefs`, `core.precomposeUnicode`, `core.protect*` and `extensions.worktreeConfig`. A key-by-key audit of `config/cache/access.rs` remains.
- I did not test whether `filter_config_section` applies when trust is `Full` (a repo owned by the user, e.g. an extracted tarball). In that case gix applies all repo-local config, as git does. Since none of it executes in the read path this matters little, but it is unproven.

## Q3. What exact rules must a safe tree walker enforce (path validation, symlinks, gitlinks, case)?

### Takeaway

Walk trees in memory and never materialize them on disk. Validate every entry name with `gix_validate::path::component` (all protections on, regardless of host OS or repo config). Accept only the five git modes. Treat symlinks as data with in-tree resolution only. Treat gitlinks as pointers resolved through a validated submodule name. Report case-fold and Unicode-normalization collisions as gaps, because the meaning of an import on macOS or Windows depends on them.

### Cited Findings

- `gix_validate::path::component(input, mode, Options)` rejects these errors: `Empty`, `PathSeparator`, `WindowsPathPrefix`, `WindowsReservedName`, `WindowsIllegalCharacter`, `DotGitDir`, `SymlinkedGitModules` and `Relative`. Its options are `protect_windows`, `protect_hfs` ("ignoring codepoints just like HFS+ would", = `core.protectHFS`) and `protect_ntfs` ("8.3 filenames and alternate data streams", = `core.protectNTFS`). "The Default implementation maximizes safety by enabling all protections." "input must not make it possible to exit the repository, or to specify absolute paths." — [gix-validate path.rs](https://docs.rs/crate/gix-validate/0.13.1/source/src/path.rs).
- GHSA-q54r shows that letting `.git/config` set `core.protectHFS`/`protectNTFS` is itself an attack. "On case-insensitive filesystems this lets crafted tree entries resolve to the .git directory" — [GHSA-q54r-jf2p-rfr2](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-q54r-jf2p-rfr2).
- Submodule names must be validated before being joined to `.git/modules`. The pre-0.82 validator only checked the first `..` — [GHSA-p3hw-mv63-rf9w](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-p3hw-mv63-rf9w), [CVE-2018-11235](https://nvd.nist.gov/vuln/detail/CVE-2018-11235).
- gix 0.90 `Submodule::git_dir()` now uses `validated_name()`. `open_modules_file` "silently ignore[s]" symlinked worktree `.gitmodules`. `Submodule::open()` clears `git_dir_trust` so the child re-derives trust — [gix submodule/mod.rs](https://docs.rs/crate/gix/0.90.0/source/src/submodule/mod.rs), [repository/submodule.rs](https://docs.rs/crate/gix/0.90.0/source/src/repository/submodule.rs).
- jj models a tree value as `File { id, executable, copy_id } | Symlink(SymlinkId) | Tree(TreeId) | GitSubmodule(CommitId)`. It maps gix `EntryKind::Commit` to `GitSubmodule` — [jj backend.rs](https://github.com/jj-vcs/jj/blob/4351061bba89b3c503d133170952f94a2c95cdfd/core/src/backend.rs), [git_backend.rs L1212-L1214](https://github.com/jj-vcs/jj/blob/4351061bba89b3c503d133170952f94a2c95cdfd/git-backend/src/git_backend.rs#L1212-L1214).
- CVE-2024-32002 combined a submodule, a symlink and case-insensitivity to redirect writes into `.git/` — [NVD](https://nvd.nist.gov/vuln/detail/CVE-2024-32002).
- unverified: git's `fsck` flags `duplicateEntries`, `badFilemode`, `hasDot`, `hasDotdot`, `hasDotgit`, `zeroPaddedFilemode`, `gitmodulesSymlink` and `treeNotSorted`, and accepts the legacy mode `100664` as a regular file.

### Inferences

Proposed walker rules for `repo.rs` (each one should be a unit test from a hand-built tree object):

1. **Names.** Validate each entry name with `gix_validate::path::component(name, mode, Options { protect_windows: true, protect_hfs: true, protect_ntfs: true })`, never from repo config. Also reject NUL, `.` and `..`, an empty name, and any `/` or `\`. The rejection is a per-path diagnostic (G5: input problems are data) and the rest of the tree is still read.
2. **Modes.** Accept `100644` and `100755` (file, executable bit recorded but not interpreted), `120000` (symlink), `040000` (tree) and `160000` (gitlink). Any other mode is a diagnostic. `100664` may be normalized to `100644` (unverified convention).
3. **Duplicates and order.** Two entries with the same name in one tree are a diagnostic, and the first is not silently preferred. Unsorted trees are a diagnostic.
4. **Case and Unicode collisions.** Within each directory, compute a case-folded, NFC-normalized key. Two entries with the same key are a `collision` gap, since a checkout on macOS or Windows keeps only one, and Python import resolution then differs by OS. Never resolve an import case-insensitively.
5. **Symlinks.** The blob content is the target string. Resolve it only virtually inside the tree: join it to the link's directory, normalize lexically, and require it to stay under the repo root (or the submodule root). Cap hops at, say, 40 to break cycles. An escaping or absolute target becomes an "outside reference" gap. Never call `fs::read` through it. A symlink named `.gitmodules` (or `.gitattributes`) is ignored with a diagnostic (`SymlinkedGitModules`).
6. **Gitlinks.** Record the path and commit id. Resolve them as in the submodule rules in Q5.
7. **Limits.** Cap tree depth, the total number of entries and per-blob bytes, with every limit named in the report. A bounded result is a gap, not a silent truncation.
8. **Never materialize.** mentalc never writes the tree to disk. That removes the whole checkout-escape advisory class (CVE-2024-35186, CVE-2026-44471, GHSA-6p9q, GHSA-pmm9).

### Gaps

- No primary source was read for git's `verify_path()`/`fsck` message list in this session. The fsck rules above are unverified.
- There is no data on how often real repositories contain case-fold collisions.

## Q4. Fidelity: what do "the files at commit X" mean?

### Takeaway

Define them as the raw blobs of the commit's tree, with modes, plus gitlinks as pointers. Do not apply `.gitattributes` (`text`/`eol`/`filter`/`export-ignore`), because checkout output depends on the user's config and on filter programs. Recognize LFS pointer files and report them as a gap instead of smudging them.

### Cited Findings

- Raw versus converted: `cat-file --filters` shows "the content as converted by the filters configured in the current working tree… (i.e. smudge filters, end-of-line conversion, etc)". Without it, cat-file returns "the raw (though uncompressed) contents" — [git-cat-file](https://git-scm.com/docs/git-cat-file).
- `git archive` applies `export-ignore` and `export-subst` from the tree's `.gitattributes`, and can be overridden by `--worktree-attributes` or `$GIT_DIR/info/attributes`. It stamps the commit time as the mtime for a commit, and "the current time" for a bare tree — [git-archive](https://git-scm.com/docs/git-archive).
- jj's `read_file` returns raw blob data. It builds a filter pipeline only for diffs — [jj git_backend.rs L512-L537](https://github.com/jj-vcs/jj/blob/4351061bba89b3c503d133170952f94a2c95cdfd/git-backend/src/git_backend.rs#L512-L537).
- zoekt (Sourcegraph's code search) indexes blobs straight from git objects at a ref, using go-git. Its option comment reads: "If set, follow submodule links. This requires RepoCacheDir to be set." — [zoekt gitindex/index.go L447-L461](https://github.com/sourcegraph/zoekt/blob/main/gitindex/index.go), [go.mod](https://github.com/sourcegraph/zoekt/blob/main/go.mod).
- Smudge filters are user-config commands (`filter.<driver>.smudge/process`). gix's own filter base implementations are unchecked in crate-status — [crate-status.md gix-filter](https://github.com/GitoxideLabs/gitoxide/blob/main/crate-status.md).
- unverified: an LFS pointer is a small text blob starting with `version https://git-lfs.github.com/spec/v1`, followed by `oid sha256:<hex>` and `size <n>`. `text=auto`/`eol=crlf` only change line endings at checkout, and committed blobs are normally LF.

### Inferences

- Raw blobs are the only definition that is the same on every machine (determinism). They are also what a parser should see: Python and TOML parsers accept both LF and CRLF.
- Record the executable bit and the symlink target in the model as facts. Only gitlinks and symlinks affect resolution.
- `.gitattributes` `export-ignore` and `linguist-vendored`/`linguist-generated` are useful hints, but they are hints only. Reading them as data, without the `attributes` feature, by parsing the blob, is fine. Applying `filter=` never is.
- An LFS pointer (detected by its prefix and a size under 1 KiB) under a path mentalc would parse becomes an `lfs_pointer` gap with the fix `git lfs pull`. Without that rule, mentalc would silently parse pointer text as source.

### Gaps

- I did not survey which other static analysers (CodeQL, Semgrep, SonarQube) read raw blobs versus a checkout.

## Q5. Submodules: where are the objects, what if they are absent, and how do gix and cargo handle them?

### Takeaway

A gitlink holds only the submodule commit id. Its objects live in `<superproject git dir>/modules/<name>` (or in an old-form `.git` directory inside the submodule's worktree), located via the `.gitmodules` name-to-path map. mentalc should parse `.gitmodules` from the commit's own tree, validate the name, open that object store read-only with the same isolated options, and check that the gitlink commit exists. An absent store or a missing commit is a nameable gap (`git submodule update --init`). Recurse with a depth cap. Never fetch.

### Cited Findings

- "a submodule usually (but not always) consists of (i) a Git directory located under the `$GIT_DIR/modules/` directory of its superproject, (ii) a working directory…, and a `.git` file at the root of the submodule's working directory pointing to (i)." "The gitlink entry contains the object name of the commit that the superproject expects the submodule's working directory to be at." `.gitmodules` "mainly serves as the mapping between the name and path of submodules". When a submodule is deinitialized, "the directory which is supposed to be the working directory is empty", while "the submodule's Git directory may be there" — [gitsubmodules(7)](https://git-scm.com/docs/gitsubmodules).
- gix `Submodule` offers `name`, `validated_name`, `path`, `url`, `update`, `branch`, `fetch_recurse`, `ignore`, `shallow`, `is_active`, `index_id`, `head_id` (from the HEAD tree), `git_dir`, `work_dir`, `git_dir_try_old_form`, `state` and `open`. `Repository::modules()` prefers the worktree `.gitmodules` file, "If a file on disk isn't present, we will try to load it from the index, and finally from the current tree". Git configuration also contributes — [repository/submodule.rs](https://docs.rs/crate/gix/0.90.0/source/src/repository/submodule.rs), [submodule/mod.rs](https://docs.rs/crate/gix/0.90.0/source/src/submodule/mod.rs).
- Cargo (git2), for each `repo.submodules()`:
  - calls `child.init(false)`;
  - skips submodules with `update = none`;
  - ignores a submodule "listed in .gitmodules but not actually checked out" (no head id);
  - otherwise fetches the submodule's own database at the gitlink commit (`with_git_precise(head)`), copies it in, and recurses.

  Source: [cargo src/sources/git/utils.rs L405-L530](https://github.com/rust-lang/cargo/blob/aaa0637ab0f0680e4addc999b767895386ecc733/src/sources/git/utils.rs#L405-L530).
- Advisories show the dangers of this path: name traversal, symlinked `.gitmodules`, and `update=!cmd` — [GHSA-p3hw](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-p3hw-mv63-rf9w), [GHSA-pg4w](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-pg4w-g64p-qwhj), [GHSA-f26g](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-f26g-jm89-4g65), [libgit2 GHSA-cw77](https://github.com/libgit2/libgit2/security/advisories/GHSA-cw77-j82w-mchm).

### Inferences

- **Do not use `gix::Submodule`.** It needs the `attributes` feature (+12 crates), mixes in worktree and config state, and reads the worktree `.gitmodules` first. For commit fidelity, the `.gitmodules` that counts is the blob in commit X's tree. Parse it as git-config text, either with `gix-config` (already in the facade's dependency tree) or with a small parser. Use only `submodule.<name>.path` and `.url`, and ignore `update`.
- **Resolution order for gitlink path P in commit X.** Find the name N whose `path == P`. Validate N (no `..` component, no leading `/`, no `\`, not empty; `gix_validate::submodule::name` from ≥ 0.11.1). Then:
  - (a) open `<common_dir>/modules/N` if it exists;
  - (b) otherwise, the old form `<worktree>/P/.git`, only when mentalc runs against a worktree and the path is a real directory, not a symlink;
  - (c) otherwise, emit a `submodule_absent` gap.

  If the store opens but lacks the commit, emit a `submodule_commit_missing` gap. Reuse the same isolated options for the child, and apply the alternates/pointer containment rule from Q2.
- Nested submodules: the child's `modules/` lives inside the child's git dir (unverified; this is git's layout for nested absorbed submodules). Recurse with a depth cap, keyed by (store path, commit) to break cycles.
- `.gitmodules` URL: record it in the model as provenance (a package URL for the `git` type, F-U20). Never contact it.
- This replaces section 6.1's "directory is empty" heuristic with a precise one: the gitlink says which commit, and the gap says whether the store or only the commit is missing.

### Gaps

- I did not read gix's `git_dir_from_name` for linked worktrees, which keep submodules in their private git dir per the doc comment, beyond that comment.
- I did not confirm the layout of nested submodules from a primary source.

## Q6. Working tree vs commit, and what jj's `Backend` trait teaches

### Takeaway

Offer two explicit sources: `--rev <commit>`, which reads objects and is deterministic, and a worktree mode, which reads files and is what an agent editing code needs. Record which one was used. Never mix them silently. jj's backend trait shows a storage interface needs only about five read methods. mentalc needs fewer, because it is read-only and has no copy tracking.

### Cited Findings

- jj's `Backend` trait (core/src/backend.rs, from L794) has:
  - metadata: `name`, `commit_id_length`, `change_id_length`, `root_commit_id`, `root_change_id`, `empty_tree_id` and `concurrency`;
  - async read and write methods: `read_file`, `write_file`, `read_symlink`, `write_symlink`, `read_copy`, `write_copy`, `get_related_copies`, `read_tree`, `write_tree`, `read_commit`, `write_commit`, `get_copy_records`, and `gc`.

  In total, about 20 methods. `read_tree` and `read_file` take the `RepoPath` as well as the id. The concurrency comment notes that a local Git backend "may want to set this to 1" — [jj backend.rs L794-L941](https://github.com/jj-vcs/jj/blob/4351061bba89b3c503d133170952f94a2c95cdfd/core/src/backend.rs#L794-L941).
- jj's git backend implements the async `read_file` by calling the synchronous `read_file_sync` and wrapping the bytes in a `Cursor` — [git_backend.rs L1108-L1115](https://github.com/jj-vcs/jj/blob/4351061bba89b3c503d133170952f94a2c95cdfd/git-backend/src/git_backend.rs#L1108-L1115).
- Cargo materializes git dependencies by `reset --hard` (git2 `CheckoutBuilder`) into a checkout directory with a `.cargo-ok` ready marker, and then reads files from disk — [cargo utils.rs L35, L326-L394, L903-L910](https://github.com/rust-lang/cargo/blob/aaa0637ab0f0680e4addc999b767895386ecc733/src/sources/git/utils.rs).
- unverified:
  - ruff, ty and rust-analyzer read the working tree from the file system (rust-analyzer through its VFS) and consult git only for ignore files;
  - difftastic gets its two files from git's external-diff interface;
  - onefetch and git-cliff read history through gix and git2 respectively ([onefetch Cargo.toml](https://github.com/o2sh/onefetch/blob/main/Cargo.toml), [git-cliff-core Cargo.toml](https://github.com/orhun/git-cliff/blob/main/git-cliff-core/Cargo.toml)).

### Inferences

- **A `Snapshot` trait for mentalc** (sync, per G16), inside `repo.rs`:
  - `fn tree(&self, dir: &RelPath) -> Result<Vec<Entry>>`;
  - `fn read(&self, file: &RelPath) -> Result<Bytes>`;
  - `fn identity(&self) -> SnapshotId`.

  `Entry` would be `File{exec}|Symlink{target}|Dir|Gitlink{commit}`. The two implementations are `GitCommit` (gix) and `WorkTree` (`std::fs`, with the same name validation). The fixture implementation is in-memory (G4: tests run from strings). jj's lesson: keep ids opaque, pass the path alongside the id, and make the empty tree and root explicit. jj's async trait plus sync implementation also shows that async buys a local backend nothing.
- **Worktree mode semantics.** Read the files as they are on disk. Optionally report "dirty versus HEAD" by comparing blob ids, which needs `index`/`status` features (and `status` pulls `dirwalk`, `attributes`). Simpler: report only `HEAD` commit id plus a "worktree" flag, and leave dirtiness unasserted. Untracked files: mentalc would need ignore rules to skip `.venv` and similar. Parse `.gitignore` itself, or rely on the existing import-root rules (section 6.1).
- **Default.** When the user names a rev, use commit mode. When an agent is iterating on uncommitted code, use worktree mode. Commit mode is what makes a model cacheable and shareable across machines.

### Gaps

- I did not read ruff, ty or rust-analyzer source for their git interaction in this session.

## Q7. Determinism and identity: git object ids or blake3?

### Takeaway

Use the git commit and tree ids as provenance and as cache keys. Keep the model's content identity as blake3 over the raw bytes, so a SHA-1 repo, a SHA-256 repo and a worktree snapshot of the same files give the same file identities. Reading objects involves no mtimes, so commit mode is mtime-free by construction.

### Cited Findings

- gix selects the object hash from `extensions.objectFormat` (SHA-1 or SHA-256), so the same content has different ids in the two formats — [incubate.rs](https://docs.rs/crate/gix/0.90.0/source/src/config/cache/incubate.rs).
- gix detects SHA-1 collisions with `sha1dc` since the CVE-2025-31130 fix. The `sha1dc` crate is in the minimal dependency set — [GHSA-2frx-2596-x5r6](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-2frx-2596-x5r6), measured `cargo tree`.
- Replace refs can change what an id resolves to, unless `core.useReplaceRefs=false` / `GIT_NO_REPLACE_OBJECTS` — [core.rs](https://docs.rs/crate/gix/0.90.0/source/src/config/tree/sections/core.rs), [git(1)](https://git-scm.com/docs/git).
- `git archive` invents mtimes: the commit time for commits and the current time for trees — [git-archive](https://git-scm.com/docs/git-archive).
- mentalc's rules: blake3 is used for cache keys and prompt hashes, and no timestamp is stored (G8) — [docs/guidelines/mentalc.md](../../../guidelines/mentalc.md), [docs/resolution.md 6.3](../../../resolution.md).
- unverified: a git blob id is `H("blob " + len + "\0" + content)`, and a tree id covers names, modes and child ids.

### Inferences

- Reading a git id is not "a second hash" under the blake3-only rule, because nothing is computed. mentalc stores `(object_format, commit_id, tree_id)` as build provenance, and can memoize the `blob_id → blake3` mapping to skip re-hashing unchanged blobs between builds of nearby commits.
- With replace refs off, no smudge, no checkout and no mtimes, commit mode depends on two things only: the object bytes, and the pointer files (alternates, submodule gitdirs) that mentalc records. It is then reproducible across machines. Worktree mode stays dependent on the machine, as section 6.3 already accepts for environments.

### Gaps

- I did not measure the cost of blake3 re-hashing versus reusing git ids on a large repository.
