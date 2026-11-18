# Every source becomes one immutable tree, mounted by name

## Per-product summary

Compilers, type checkers and language servers:

- **rust-analyzer**: an in-memory store of path ids and content hashes, fed by a separate 4-method loader that does the I/O; dependencies are ordinary directories marked "library"; no git, no archives.
- **ty / ruff**: a 26-method host trait plus a separate small reader for its own embedded typeshed zip; file identity is an enum with one variant per origin; site-packages read as data, no zip imports.
- **pyright**: a 25-method, Node-shaped file system; zip, egg and jar mounted as directories through a third-party zip FS and a patched private method; the cautionary tale.
- **typst**: `World::file(FileId)`, where `FileId` is (root enum: project or package@version, rooted path); packages are unpacked tarballs; the closest model for `mentalc`.
- **LLVM / clangd**: a POSIX-like refcounted VFS with overlay, in-memory and a YAML redirect layer that is half the code; ten years of bypasses led to a 2025 runtime "IO sandbox".
- **TypeScript 6 (JS)**: about 29 host members mixing files, watching and process; **TypeScript 7 (Go)**: an 11-method FS plus small one-purpose decorators.
- **gopls**: one method, `ReadFile(uri) -> Handle{URI, Hash}`; directory discovery is delegated to `go list`.
- **deno_graph**: one `load(url)` across `file:`, `https:`, `data:`; npm escapes into a separate resolver.

Code search, indexers and analysers:

- **zoekt**: walks branch trees straight from the git object store, never checks out; git, tar/zip and plain directories all feed one `Document{Name, Content, SubRepositoryPath}`; submodules opt-in.
- **Sourcegraph**: git only inside; Perforce and package registries are converted into git repositories by "syncers" with two methods.
- **Software Heritage**: dozens of loaders normalise git, hg, svn, tarballs and about 20 package ecosystems into one git-compatible Merkle graph.
- **GitHub code search (Blackbird)**: crawls blobs from git and shards and deduplicates by blob id.
- **GitLab Gitaly**: the canonical "file at (revision, path)" RPC set over bare repositories.
- **livegrep**: libgit2 tree walk at a revision, optional submodule recursion, no checkout.
- **OpenGrok**: indexes a checkout; 15 native VCS drivers for history only, about 9,300 lines for the 13 non-git ones.
- **Hound**: a 5-method driver per VCS (git, hg, svn, bzr, local) that only clones and pulls a checkout.
- **Sourcebot**: wraps zoekt; local git repositories by `file://`.
- **CodeQL**: reads a checkout or follows the build; the database carries a source archive (zip, directory or bare git).
- **Kythe**: location is (corpus, root, path); content is a SHA-256 digest.
- **Glean**: the database is `<name>/<hash>`, the hash often the revision; files are bare paths.
- **Semgrep**: scans the working tree; a baseline commit means a temporary `git worktree` checkout.
- **SonarQube**: scans the working tree; git and SVN only for blame; tells TFVC users to convert to git.
- **PyDriller**: reads blob streams through GitPython; checkout is an explicit opt-in.
- **CodeScene**: git only (snippet-only).

Build systems and stores:

- **Bazel**: a 25-method mutable path FS, plus content-addressed Merkle directories for remote caching; archives are fetched by sha256 and normalised with `strip_prefix`.
- **Buck2**: a 5-method read-only input trait keyed by (cell, path); the cell is the mount table.
- **Nix**: store paths named by digest plus a human name; NAR hash as tree identity.

General virtual file system libraries:

- **Go `io/fs`**: one required method; one extension added in five years; `fstest.TestFS` checks any backend; the model to copy.
- **Java NIO.2 with zipfs**: 19 provider methods, a read-write zip FS of about 253 KB; too big.
- **IntelliJ VFS**: a live persistent snapshot whose staleness and refresh are the main pain.
- **Eclipse EFS, Apache Commons VFS, Python fsspec**: URI or chained-URL file systems with 26, about 40 and 83 methods; growth from remote transports, caching and async.
- **Rust `vfs`, `object_store`, `opendal`**: write-heavy, async-only, or tokio even for blocking calls; all banned weight.
- **Rust `cap-std`, `rust-embed`, `include_dir`**: capability handles or embedded trees; not needed for inputs.

Git readers for Rust:

- **gix facade 0.90** (`default-features = false`): pure Rust, 88 to 100 crates by feature set; reads objects without running anything when opened isolated.
- **gix plumbing crates**: the same object code without the facade; 58 crates and no process or network code linked.
- **git2 / libgit2**: 12 Rust crates but about 124k lines of C; SHA-256 only behind an unstable ABI flag; not recommended.
- **Running `git`**: always reads repository-local config, can fetch lazily, varies by installed version; rejected.

Dependency formats and caches:

- **Wheel, Go module zip, Maven sources jar**: readable without execution and faithful to what runs.
- **sdist, crate, npm tarball, gem**: readable, but build or install steps add or move files.
- **Installed site-packages**: readable as data; one distribution's files are its `RECORD` lines; no archive hash.
- **Go module cache**: the only cache documented as an interface; **Cargo, pip, uv, npm caches**: internal layouts, readable only if every artifact is verified against a lockfile hash.

Rust archive crates:

- **`zip`**: 20k lines, 11 required dependencies, crypto on by default; usable with `default-features = false`.
- **`rc-zip`**: 3.3k lines but 9 required dependencies; **`tar`**: 3.5k lines, 2 dependencies; **`async_zip`**: async, banned.

## Summary

The tools that stay small all do the same thing: they turn every origin of files into **one immutable, read-only tree value** at the edge, name that tree by an enum of origins, and give the analysis core nothing but "bytes by identity" plus listing inside a tree. The content interfaces that last are tiny (**typst 1 method, gopls 1, deno_graph 1, rust-analyzer's loader 4, Go `io/fs` 1, Buck2 5**); the large ones (**ty 26, pyright 25, TypeScript `System` about 29, fsspec 83**) are host abstractions for long-running servers, carrying write, watch, cwd, environment, attributes, async or remote transports, none of which a batch compiler on a frozen snapshot needs. No compiler or language server studied reads git; they read directories. The scalable indexers (zoekt, livegrep, Blackbird, Gitaly, Software Heritage) read git objects directly and never check out, and the industry answer to other version control systems is to **convert to git at the edge**, not to write drivers: OpenGrok's native support costs about 9,300 lines for 13 non-git drivers and still needs a checkout. Archives work best as a root kind with their own reader, not as directories inside paths (pyright's regret). For `mentalc` this means: keep the accepted `RepoSnapshot` and extend it into a `Snapshot` of mounted roots, each produced by one of a closed set of reader functions (git commit, submodule, worktree, zip, tar, installed distribution) that all return the same `Tree` value; read git objects in-process with gix and never run the git program; treat submodules as Merkle mounts at their gitlink commit and dependencies as separate library roots; identify content by blake3 and record git ids as provenance; and enforce the edge mechanically from the first commit. The design is about **1,700 lines of non-test code**. It needs one wording change in `resolution.md` (from "does not run git" to "does not run the git program; reads git objects as data"), a new git dependency, two archive crates and a narrowed "blake3 is the only hash" rule. Three questions stay open as experiments: the gix facade against its plumbing crates, blake3 against git ids as the content key (blake3 recommended), and installed environments against verified caches as the dependency source (installed environments recommended for version one, because [report-jobs](../../../examples/report-jobs/) commits no hashes and no environment).

Evidence labels: a claim with no label was read in full at the source by the researchers (code at a pinned commit, a spec, a doc page or a measurement); **snippet-only** means it comes from a search-result summary and is a lead; **unverified** means it rests on prior knowledge and was not checked this session. The five notes are [language_tool_vfs.md](notes/language_tool_vfs.md), [indexers_and_vcs_sources.md](notes/indexers_and_vcs_sources.md), [general_vfs_abstractions.md](notes/general_vfs_abstractions.md), [git_access_rust.md](notes/git_access_rust.md) and [dependency_artifacts.md](notes/dependency_artifacts.md).

## 1. Compilers keep the core on "bytes by identity" and push discovery to the edge

Two shapes recur in compiler-grade tools. One is a POSIX-like file system over paths (LLVM, pyright, TypeScript's `System`, ty's `System`). The other is a content source keyed by an opaque identity (rust-analyzer's `FileId`, typst's `FileId`, gopls's `Identity{URI, Hash}`, deno_graph's URL). The tools whose core is an analyzer put an interned identity at the core and keep anything file-system-shaped at the edge ([language_tool_vfs.md §1](notes/language_tool_vfs.md)). rust-analyzer's database "doesn't know about file system and file paths"; its `Vfs` stores only content hashes and pending changes, and "VFS doesn't do IO or file watching itself" ([RA vfs/src/lib.rs](https://github.com/rust-lang/rust-analyzer/blob/15d97b98eacfad690e81757e867ca0800f1c9e07/crates/vfs/src/lib.rs#L1), [loader.rs](https://github.com/rust-lang/rust-analyzer/blob/15d97b98eacfad690e81757e867ca0800f1c9e07/crates/vfs/src/loader.rs#L78)). Relative lookups such as `mod foo;` go through a `FileSet` index, not through a file system. typst's `World` has **no directory listing, no `exists` and no metadata at all**, and its newer `typst-kit` file layer is a one-method `FileLoader::load(FileId)` in about 470 lines ([typst lib.rs](https://github.com/typst/typst/blob/c56dfa6043014b2c1cae2b104b6bab0dd1b195c7/crates/typst-library/src/lib.rs#L63), [typst-kit files.rs](https://github.com/typst/typst/blob/c56dfa6043014b2c1cae2b104b6bab0dd1b195c7/crates/typst-kit/src/files.rs#L256)).

Every tool still needs both discovery (list a directory, test existence) and content (bytes for an id). The clean designs keep them apart. Discovery happens once, at load time, and the core sees only ids. Where the whole file system reaches the core, it grows: pyright had to add `isInZip`, `isMappedUri`, `getOriginalUri` and `getMappedUri` to its core interface once archives and partial stub packages were mounted as directories, and it overrides a *private* method of yarn's zip FS with a comment promising "to remove this hack in favor of a full ZipOpenFS fork" ([pyright realFileSystem.ts](https://github.com/microsoft/pyright/blob/b2df5a4fc0c126236b90919d41c47f353dcc5dd1/packages/pyright-internal/src/common/realFileSystem.ts#L119)). For a frozen snapshot, Python's need for `exists` and `list_dir` inside an import root is best met by a pure query over a sorted in-memory index. Because the snapshot cannot change, a miss is a fact and no negative lookups need tracking, which is the whole reason tsgo's `trackingvfs` exists ([trackingvfs.go](https://github.com/microsoft/TypeScript/blob/ef0c06e25aa9265d3e1d69b1282b0c6528426161/tsc/internal/vfs/trackingvfs/trackingvfs.go#L1)).

The regrets are specific and they all point the same way. ruff renamed `FileSystem` to `System` in 2024 because the trait "isn't about abstracting a way a file-system like API; it's solely for abstracting system-dependent logic", and its embedded typeshed zip deliberately does not implement it ([ruff PR #12214](https://github.com/astral-sh/ruff/pull/12214)). typst replaced `Option<PackageSpec>` with a `VirtualRoot` enum in January 2026 because an enum "gives us the option to introduce additional kinds of roots", and replaced `PathBuf` with a portable path that "validates upon construction that its associated root is not escaped" ([typst PR #7689](https://github.com/typst/typst/pull/7689), [PR #7555](https://github.com/typst/typst/pull/7555)). LLVM's stateful working directory is documented as "usually thread-hostile", its YAML `RedirectingFileSystem` is **about 1,550 of 3,072 implementation lines**, and an accepted 2025 RFC adds an "IO sandbox" because code still bypasses the VFS a decade after it was introduced, which "breaks CAS-based caching" ([VirtualFileSystem.h](https://github.com/llvm/llvm-project/blob/20b943451383f3e8af4ae8c37b71700e9423602d/llvm/include/llvm/Support/VirtualFileSystem.h#L381), [RFC](https://discourse.llvm.org/t/rfc-file-system-sandboxing-in-clang-llvm/88791)). ruff enforces the same boundary statically with `clippy.toml` `disallowed-methods` over `std::fs` and `std::env` ([ruff clippy.toml](https://github.com/astral-sh/ruff/blob/0222c19dd8e0c9a927f2e47bf3774a108431da25/clippy.toml)), which is exactly Mental's G4. The bypass risk LLVM describes is `mentalc`'s content-addressed cache risk, so the boundary must be machine-checked from day one.

Archives and packages follow the same lesson. ty reads only its own embedded zip, through a separate small reader whose revision is the entry CRC32; typst downloads `.tar.gz` packages and unpacks them to a cache; tsgo serves embedded lib files under a `bundled:///` scheme through a wrapping FS ([vendored.rs](https://github.com/astral-sh/ruff/blob/0222c19dd8e0c9a927f2e47bf3774a108431da25/crates/ruff_db/src/vendored.rs#L37), [typst packages.rs](https://github.com/typst/typst/blob/c56dfa6043014b2c1cae2b104b6bab0dd1b195c7/crates/typst-kit/src/packages.rs#L95), [tsgo embed.go](https://github.com/microsoft/TypeScript/blob/ef0c06e25aa9265d3e1d69b1282b0c6528426161/tsc/internal/bundled/embed.go#L16)). rust-analyzer and ty treat dependencies as ordinary directories under a root flagged "library" or "search path", and rust-analyzer's rule that "a file from one source root can't refer to a file in another source root by path" keeps them separate ([RA workspace.rs](https://github.com/rust-lang/rust-analyzer/blob/15d97b98eacfad690e81757e867ca0800f1c9e07/crates/project-model/src/workspace.rs#L43)). Tests follow from the shape: rust-analyzer's fixtures never touch disk and use virtual paths, while ruff and pyright each carry 1 to 2 thousand lines of in-memory file system because their cores call a file system interface ([language_tool_vfs.md §6](notes/language_tool_vfs.md)). A core that consumes a snapshot value needs no fake file system at all: the fixture string *is* the snapshot.

## 2. Indexers read git objects directly and convert everything else to git

Code search and indexing split into three families ([indexers_and_vcs_sources.md](notes/indexers_and_vcs_sources.md)). **Object-store readers** (zoekt, livegrep, Blackbird, Gitaly, Sourcegraph's gitserver, PyDriller, Software Heritage's git loader) never materialise a working tree. **Checkout readers** (OpenGrok, Hound, CodeQL, Semgrep, SonarQube) rely on a file system and treat the VCS as metadata for history and blame. **Normalisers** (Sourcegraph's syncers, Software Heritage's loaders) convert each foreign source once into a git-shaped store. zoekt is the best template for `mentalc`: its `RepoWalker` walks a commit's tree with go-git, keeps regular, executable and symlink entries, reads the ignore file from the indexed tree rather than disk, and every front end (git, archive, directory) produces the same `index.Document{Name, Content, Branches, SubRepositoryPath, SkipReason}` for one builder that never knows where a file came from ([zoekt tree.go](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/tree.go#L104-L262), [document.go](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/index/document.go#L6-L33)). Oversized and binary files stay as name-only placeholders with a skip reason, which preserves completeness without reading them. zoekt's one operational scar is instructive: its `git cat-file --batch` path was disabled by default "after git grew past 9GB of memory in production" because of lazy fetching in partial clones ([zoekt indexing.md](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/doc/indexing.md#L47-L69)). Running the git program brings the repository's fetch behaviour with it.

Native support for many version control systems is rare and expensive. Only OpenGrok (15 drivers) and Hound (5) do it, and both only to the depth of "a checkout plus history". OpenGrok's history package is **18,113 lines plus 8,932 lines of tests**, with the 13 non-git drivers totalling about 9,300 lines against Git's 940, each shelling out to its CLI and parsing text ([OpenGrok history](https://github.com/oracle/opengrok/tree/dd385925e974873e5cdf44fe3599ae40e1866166/opengrok-indexer/src/main/java/org/opengrok/indexer/history)). Hound stays cheap because a driver does nothing but Clone, Pull and HeadRev ([hound vcs.go](https://github.com/hound-search/hound/blob/cf795f1889a7711cd46bfbef5f865544beeec391/vcs/vcs.go#L16-L31)). Sourcegraph's contract for every syncer is that the result "is expected to be a valid Git repository"; Perforce needed its own converter, p4-fusion, claimed to be over 100 times faster than git-p4; and for CVS and SVN "we do not offer native integrations" ([syncer.go](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/syncer.go#L34-L64), [p4-fusion](https://github.com/sourcegraph/p4-fusion/blob/master/README.md), [Sourcegraph code hosts](https://sourcegraph.com/docs/admin/code_hosts)). SonarQube tells TFVC users plainly to "convert your project to a Git repository" ([Sonar SCM](https://docs.sonarsource.com/sonarqube-server/analyzing-source-code/scm-integration)). Packages are handled the same way: Sourcegraph unpacks each package version and commits it into a synthetic git repository with a fixed author and date, so the commit id depends only on the content; Software Heritage makes a synthetic release pointing at the unpacked directory and keeps the tarball hash as an `ExtID` ([packages_syncer.go](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/packages_syncer.go#L136-L295), [SWH data model](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/docs/data-model.rst)).

Submodules are off by default or best effort everywhere. zoekt follows them only when a repository cache is configured, parses `.gitmodules` from the tree rather than the worktree, recurses at the gitlink commit with a path prefix, logs and ignores submodule errors, and rejects them in delta builds ([zoekt tree.go](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/tree.go#L148-L230), [index.go](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/index.go#L951-L953)). livegrep skips gitlinks unless asked, and Gitaly's own docs discourage `GetBlobs` because it "will also return submodules as commits and trees" ([livegrep git_indexer.cc](https://github.com/livegrep/livegrep/blob/923d5ad71dfe60900e6c2017b2fa4a5ff902ad71/src/git_indexer.cc#L60-L127), [Gitaly blob.proto](https://gitlab.com/gitlab-org/gitaly/-/blob/master/proto/blob.proto)). The common shape is correct for `mentalc`, but "logs and ignores" is not: under Mental's rules a submodule that cannot be read is a named gap.

On identity, the git blob id is the de facto content key. Blackbird shards on it "avoiding any duplication" and found only about 28 TB unique in 115 TB of code; zoekt keys files on (sub-repo, path, blob id); Software Heritage makes all its ids git-compatible so a tarball and a commit with the same files get the same directory id ([GitHub blog](https://github.blog/engineering/architecture-optimization/the-technology-behind-githubs-new-code-search/), [SWH identifiers](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/docs/persistent-identifiers.rst#L144-L168)). Kythe separates "where" (corpus, root, path) from "what" (a SHA-256 digest) ([Kythe analysis.proto](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/analysis.proto#L260-L290)). These are hosting-scale systems whose whole world is git; section 6 explains why `mentalc` takes the Kythe split with blake3 instead.

## 3. Read-only snapshots remove everything that bloats general VFS libraries

General-purpose VFS libraries show what drives growth, in order of evidence ([general_vfs_abstractions.md](notes/general_vfs_abstractions.md)): write and mutation (**11 of 15 methods of Rust's `vfs`, 12 of Bazel's 25 abstract methods**, and the 141 KB `ZipFileSystem.java` of read-write zipfs), attributes and permissions, remote services (opendal's 50-plus services, Commons VFS's three generations of HTTP providers side by side), async (object_store and opendal are async-first), live coherence (IntelliJ's refresh, events and "no facility for removing files from the snapshot"), and capability negotiation once backends differ (opendal's 91-field `Capability`). Go's `io/fs` avoided all of it by design: "The `FS` interface defines the *minimum* requirement for an implementation: just an `Open` method", because "previous attempts at file system interfaces floundered in the complexity of defining a complete set of operations" ([Go draft design](https://go.googlesource.com/proposal/+/master/design/draft-iofs.md)). In five years only one extension interface was added, and the writable-interfaces proposal was declined after 157 comments because the open modes are not portable ([golang/go#49580](https://github.com/golang/go/issues/49580), [#45757](https://github.com/golang/go/issues/45757)). opendal's own RFC records the cost of serving two execution models: a "Huge maintenance burden: We need to maintain two versions of the same code", resolved by requiring an async runtime even for blocking calls ([opendal #6189](https://github.com/apache/opendal/pull/6189)).

A git commit, a zip and a published artifact are all immutable once identified, so the coherence layer disappears entirely for `mentalc`, and with it IntelliJ's and fsspec's main failure modes. What remains is composition. Go offers only `Sub`; Buck2 addresses every input as `CellPath = (cell, relative path)`, so cells act as a mount table; Bazel's remote-execution `Directory` places a child tree by digest, so mounting a subtree is one `DirectoryNode{name, digest}` ([buck2 trait_.rs](https://github.com/facebook/buck2/blob/main/app/buck2_common/src/file_ops/trait_.rs), [remote_execution.proto](https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto)). A commit with submodules and dependency archives needs exactly that: a submodule is a Merkle placement at the gitlink path, and a dependency is its own named root. Overlay and union are not needed because nothing shadows anything, and chained URL syntax (`zip::s3://...`, `zip:http://host/a.zip!/dir`) is where fsspec's and Commons VFS's nesting complexity lives; a typed enum gives the same reach with no string grammar. Go's path rules are worth copying verbatim: unrooted, slash-separated, never `.` or `..`, separator `/` on every system ([draft-iofs.md](https://go.googlesource.com/proposal/+/master/design/draft-iofs.md)). And Go's `fstest.TestFS` "walks the entire tree of files in fsys, opening and checking that each file behaves correctly", so one generic test checks every backend ([testfs.go](https://github.com/golang/go/blob/master/src/testing/fstest/testfs.go)).

The Rust crate survey settles what not to take. `vfs` 0.13 carries a parallel async module and mostly write methods; `object_store` is async with 15 required dependencies; `opendal-core` requires tokio; `async_zip` is async. Only `tar` (3.5k lines, 2 required dependencies), `rc-zip` (3.3k lines, 9) and `zip` with `default-features = false` (20k lines, 11 required, 9 default features including AES, bzip2, lzma and zstd) fit G16 ([crates.io zip](https://crates.io/crates/zip), [tar](https://crates.io/crates/tar), [rc-zip](https://crates.io/crates/rc-zip)). These counts are direct dependencies from crates.io, not measured transitive closures.

## 4. Git can be read as data without running anything the repository controls

git's own manual is the reason `resolution.md` says `mentalc` "does not run git": "it is not safe to run Git commands in a .git directory ... when that .git directory itself comes from an untrusted source", and the danger is that "some configuration options and hook files may cause Git to run arbitrary shell commands" ([git(1) SECURITY](https://git-scm.com/docs/git#_security)). The same manual says inspecting history of untrusted content is generally safe. The danger is the git *program* honouring repository config, not the object format. Repository-local config cannot be switched off wholesale in git, `cat-file` on a missing object in a partial clone fetches unless `GIT_NO_LAZY_FETCH=1`, and output depends on the installed version ([git_access_rust.md Q2](notes/git_access_rust.md)). Running git stays rejected. Reading objects in-process is a different act, and gix can do it with nothing executed: `open::Options::isolated()` reads no system, user or XDG config, no includes and no environment, and its permission struct keeps `git_binary: false` even under `Config::all()` ([gix open/permissions.rs](https://docs.rs/crate/gix/0.90.0/source/src/open/permissions.rs), [options.rs](https://docs.rs/crate/gix/0.90.0/source/src/open/options.rs)). A grep of the 0.90 tarballs finds no `Command::new` in `gix-odb`, `gix-object`, `gix-ref`, `gix-pack` or `gix-hash`; the spawn sites are in `gix-path` (only with `git_binary`), signing, the editor, and the transport and protocol crates. Hooks, fsmonitor, pagers and smudge filters are porcelain behaviour gix does not implement on read or that `mentalc` does not compile in.

The library choice is between four options, with crate counts measured by `cargo tree` on scratch manifests ([git_access_rust.md Q1](notes/git_access_rust.md)):

| Option | Crates | For | Against |
|---|---|---|---|
| `gix` 0.90, `sha1` only | **88** | discovery, config layering with trust filtering, `objectFormat`, replace refs handled for you | links `gix-config`, `gix-protocol`, `gix-transport` and `gix-command` even with minimal features; `revision` (+4) needed for `rev_parse_single`; `attributes` (+12) needed for `gix::Submodule` |
| gix plumbing (`gix-odb`, `gix-pack`, `gix-object`, `gix-ref`, `gix-hash`, `gix-validate`) | **58** | no config, transport or command crates linked at all | `mentalc` writes discovery, `extensions.objectFormat` and ref resolution itself |
| `git2` 0.21 (libgit2 1.9.7) | 12 Rust, about 124k lines of C | mature | needs a C toolchain; SHA-256 behind `unstable-sha256`, "ABI-breaking" and not thread-safe without OpenSSL; 2026 advisories on delta allocation and submodule path traversal |
| run `git` | 0 | the reference implementation | rejected: R3, F-U15, version drift, lazy fetch |

gitoxide's 2026 advisories map the hazards precisely. An untrusted `.git/config` could disable the `.git` path-confusion checks through `core.protectHFS`/`protectNTFS`; a crafted commit-graph aborted every history walk; pack and delta data caused panics and unbounded allocation; unvalidated `.gitmodules` names escaped `.git/modules` and, combined with trust inheritance, read other repositories' configs "including credentials"; and a symlinked worktree `.gitmodules` was followed outside the repository. All are fixed in gix 0.90 ([GHSA-q54r](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-q54r-jf2p-rfr2), [GHSA-989c](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-989c-xj77-266r), [GHSA-x494](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-x494-mj8g-cj27), [GHSA-p3hw](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-p3hw-mv63-rf9w), [GHSA-pg4w](https://github.com/GitoxideLabs/gitoxide/security/advisories/GHSA-pg4w-g64p-qwhj)). Each one translates into a rule: validate names with all protections on regardless of repository config, never walk ancestry (so the commit-graph is never read), cap object sizes and decode under `catch_unwind` so a malformed repository is a typed error rather than an abort, validate submodule names before joining them to a path, and never read the worktree `.gitmodules`. Never checking out removes the whole checkout-escape class (CVE-2024-35186, CVE-2026-44471 and others) by construction.

Two git questions need a definition rather than a library. **Fidelity**: "the files at commit X" are the raw blobs of the commit's tree with their modes, plus gitlinks as pointers. `.gitattributes` filters, end-of-line conversion and `export-ignore` are not applied, because checkout output depends on user config and filter programs; jj's `read_file` and zoekt both read raw blobs ([jj git_backend.rs](https://github.com/jj-vcs/jj/blob/4351061bba89b3c503d133170952f94a2c95cdfd/git-backend/src/git_backend.rs#L512-L537)). A Git LFS pointer (a small text blob beginning `version https://git-lfs.github.com/spec/v1`, unverified) under a path `mentalc` would parse becomes a gap, never parsed as source. **Indirection**: alternates, `gitdir:` files, `commondir` and submodule git dirs can make `mentalc` read objects from elsewhere on the user's disk into a model. This is a data-disclosure risk, not code execution, and both git and gix follow alternates unconditionally ([gix-odb alternate](https://docs.rs/crate/gix-odb/0.87.0/source/src/alternate/mod.rs)). The rule is to follow a pointer only when it resolves under the repository root or a root listed in `mentalc.toml`, and to record every pointer followed.

Submodule objects live in `<superproject git dir>/modules/<name>`, located through the `.gitmodules` name-to-path map, and a deinitialised submodule has an empty directory while "the submodule's Git directory may be there" ([gitsubmodules(7)](https://git-scm.com/docs/gitsubmodules)). That gives a precise replacement for `resolution.md` §6.1's "directory is empty" heuristic: the gitlink names the commit, and the gap says whether the store or only the commit is missing. Cargo does the same with git2, skipping a submodule "listed in .gitmodules but not actually checked out" ([cargo utils.rs](https://github.com/rust-lang/cargo/blob/aaa0637ab0f0680e4addc999b767895386ecc733/src/sources/git/utils.rs#L405-L530)); `mentalc` reports it instead of skipping it.

## 5. Dependency trees are readable without execution, but only some are faithful

Every package format studied can be unpacked and read without running code. The real question is whether the unpacked files are the files the program sees ([dependency_artifacts.md Q1](notes/dependency_artifacts.md)). That holds for **wheels** (once `*.data/purelib` and `platlib` are merged into the root), **Go module zips** (no symlinks, no install hooks) and real **Maven `-sources.jar`** files, which Central allows to be placeholders. It holds for an **installed site-packages** except for import-hook editables. It holds only partly for **sdists** (the build backend decides the module layout; `PKG-INFO` is trustworthy only at Metadata 2.2 and above for fields not marked `Dynamic`), **crates** (`build.rs` output is missing), **npm tarballs** (install-script output is missing, unverified) and **gems** (native extensions are missing) ([wheel spec](https://packaging.python.org/en/latest/specifications/binary-distribution-format/), [core metadata](https://packaging.python.org/en/latest/specifications/core-metadata/), [Go modules zip](https://go.dev/ref/mod#zip-files), [Sonatype requirements](https://central.sonatype.org/publish/requirements/)). So each dependency tree carries an origin kind, and imports into generated modules missing from a pre-build tree are gaps with a reason.

Caches are a weaker source than they look. Only Go's module download cache is documented as an interface: it "may be used as a proxy when served by an HTTP file server" ([Go module cache](https://go.dev/ref/mod#module-cache)). Cargo says its home "is not stabilized and may be subject to change at any time"; pip calls its cache "an implementation detail" that "may change between any two versions"; uv's buckets are versioned internals, and uv keeps wheels as *unpacked* directories in `archive-v0` with no archive hash, so they cannot be checked against `uv.lock`'s sha256 ([Cargo home](https://doc.rust-lang.org/cargo/guide/cargo-home.html), [pip caching](https://pip.pypa.io/en/stable/topics/caching/), [uv-cache lib.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-cache/src/lib.rs)). A cache is safe to read only when every artifact taken from it is verified against a hash in a committed lockfile; its instability then affects only finding the file, not trusting it. For Python today that leaves little: a verifiable wheel must exist as an archive somewhere the configuration names.

Identity needs two digests. An **artifact digest** is what the lockfile pins (a wheel's sha256, an npm SRI, a crate checksum, a git commit) and proves where the tree came from. A **tree digest** is a canonical hash over normalised relative paths and contents and names what was analysed, so the same files arriving as a wheel, an installed copy or a vendored directory give the same node. Go's `h1:` (SHA-256 over sorted file hashes), uv's BLAKE3 dirhash and Nix's NAR hash are all tree digests; uv's is designed so "the dirhash of an archive is the same as the dirhash of its unpacked files" and counts only names and contents, not the executable bit ([dirhash](https://pkg.go.dev/golang.org/x/mod/sumdb/dirhash), [uv dirhash.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-extract/src/dirhash.rs)). uv is precedent that BLAKE3 works for this purpose.

Archive safety rules converge across uv, Go and PEP 706, and mounting in memory makes them simpler. uv rejects member names that escape the root, duplicate output paths, duplicate local headers and local-versus-central size conflicts, and "ZIP entries are never followed as symlinks"; Go's `x/mod/zip` caps a module zip and its uncompressed content at **500 MiB** and `go.mod` at **16 MiB**; PEP 706's `data` filter refuses absolute paths, escapes and device files but leaves size and count limits to callers ([uv-extract error.rs](https://github.com/astral-sh/uv/blob/main/crates/uv-extract/src/error.rs), [x/mod/zip](https://pkg.go.dev/golang.org/x/mod/zip), [PEP 706](https://peps.python.org/pep-0706/)). Zip Slip's two-entry variant, a symlink pointing outside followed by a regular file of the same name, bypassed earlier fixes in two npm extractors (snippet-only, [Snyk](https://security.snyk.io/vuln/SNYK-JS-EXTRACTZIP-19653641)). Since `mentalc` never extracts to disk, the write-side class disappears; path validation is still needed so that two names cannot alias in the model.

The example repository sharpens all of this. `examples/report-jobs/src/requirements.lock` (since replaced by `uv.lock`) pinned 11 packages as `name==version` lines **with no hashes**, and the example has **no `.venv`**. Measured on this machine on 2026-11-16, the mental repository at `HEAD` is a SHA-1 repository with 158 tree entries, packed objects plus loose ones, no `.gitmodules`, and report-jobs is a subdirectory holding 8 regular files. As committed, report-jobs exercises commit reading and subdirectory mounting, but no submodule, no archive and no dependency tree can be mounted from it, verified or not.

## 6. Three experiments to settle before writing the reader

**Status: Experiment.** Each keeps two credible designs side by side until a measurement decides, as agreed for this phase.

### S1. The git reader: gix facade or gix plumbing crates

| | A. `gix` facade, `default-features = false, features = ["sha1"]` | B. gix plumbing crates |
|---|---|---|
| **Crates** | 88 (92 with `revision`) | 58 |
| **Process or network code linked** | `gix-command`, `gix-transport`, `gix-protocol` | none |
| **`mentalc` writes** | tree walker, `.gitmodules` parser (both arms need these) | the same, plus finding the git dir (directory or `gitdir:` file, `commondir`), reading `core.repositoryFormatVersion` and `extensions.*` from `.git/config` with the same small git-config parser used for `.gitmodules`, and resolving `HEAD` and ref names through `gix-ref` |
| **Extra enforcement** | a tidy test banning `Repository::{remote*, find_remote, command_context, filter_pipeline, submodules}` and `open::Options::with` | none beyond "gix crates only in `source/git.rs`" |
| **Risk** | linked code the G4 lint cannot see | a subtle mistake in discovery or format detection |

The research note leaned towards A for its trust-filtered config handling ([git_access_rust.md Q1](notes/git_access_rust.md)). With one narrowing, I lean towards B: `mentalc --rev` accepts only `HEAD`, a full ref name, a branch or tag name, or a full object id, never revision arithmetic such as `HEAD~3` (an agent resolves that with git itself and passes the id). That removes the need for `revision` and the commit-graph. Both arms already write their own walker and `.gitmodules` parser, and `mentalc` opens isolated anyway, so the facade's config layering buys nothing. B also lets `mentalc` refuse any unknown `extensions.*` key with a named error, which the note could not confirm the facade does. **Decide by**, on this machine: (1) the `Cargo.lock` delta and the clean release build time and binary size of each arm; (2) the lines `mentalc` writes in each; (3) a conformance list both arms must pass: report-jobs at `HEAD` of the mental repository with `subdir = examples/report-jobs` (packed plus loose objects), a linked worktree, a `gitdir:` file, `packed-refs` only, a fixture with an absorbed submodule in `.git/modules/<name>`, alternates pointing outside the root (must be a gap), a SHA-256 repository (must be a named error unless `sha256` is enabled), and a corrupt pack (must be a typed error, not an abort); (4) load time for report-jobs and for one repository of about 50,000 files. Choose B if it passes the list with no more than about 300 extra lines; otherwise A with the bans.

### S2. Content identity: blake3 or git object ids

| | A. blake3 of raw bytes (recommended) | B. git object ids |
|---|---|---|
| **File id** | blake3 of the bytes; the git blob id is kept as provenance | the git blob id; computed as SHA-1 of `"blob <len>\0" + bytes` for worktree, zip and tar sources |
| **Tree id** | blake3 tree digest over sorted (path, kind, file digest), executable bit excluded, child roots by their own digest | git tree id |
| **Same files via commit, worktree and zip** | same id by construction | same only if every non-git source hashes git-style |
| **SHA-1 versus SHA-256 repositories** | same id | different ids for the same content |
| **Rule fit** | blake3 stays the only hash `mentalc` computes | adds SHA-1 computation outside the git reader |
| **Cost** | one hash pass over every byte read | free for committed files |

The indexers' preference for git ids (section 2) comes from systems whose world is git hosting. `mentalc`'s cache key already hashes the exact *slice* placed in a prompt, not the file ([C12](../../guidelines/mentalc.md#c12-the-cache-key-is-content-never-the-llm)), so the file id does not even enter the key; it serves snapshot equality, provenance and the tree digest on pins. The evidence is clear enough to recommend A. **Decide by** cost only: time the blake3 pass against total load time on report-jobs (8 files, about 6.5 KB, trivially small) and on the 50,000-file repository from S1. If hashing exceeds about 10% of load time, memoise `git blob id -> blake3` in `.mental/` rather than switching identity. Go's `h1:` is computed only when Go is scheduled, to verify `go.sum` (it is SHA-256).

### S3. Where dependency files come from: the installed environment or verified caches

| | A. Installed environment only (recommended for version one) | B. A plus verified local artifacts |
|---|---|---|
| **Reads** | the environment rung 2 already names (`mentalc.toml` path, else `.venv`); one library root per distribution, its file set taken from `RECORD` lines, never a directory walk | wheel or sdist archives at locations named in `mentalc.toml` or a fixed list of documented defaults, counted only when their sha256 matches a lockfile hash |
| **Rank** | `observed` | `declared` |
| **New rules** | none (rung 2 as written) | `resolution.md` §6.3 "does not search the machine" changes; a SHA-256 implementation joins the allowlist |
| **Fits Python today** | yes, wherever an environment exists | poorly: uv keeps unpacked wheels with no archive hash, pip's cache is unreadable by policy |
| **Fits Cargo and Go (Later)** | registry sources and the module cache | well: the `.crate` and module zip are kept and match `Cargo.lock` and `go.sum` |

On report-jobs as committed, both arms mount **zero** dependency trees: no hashes means B can verify nothing, and no environment means A has nothing to read. The experiment therefore needs a corpus variant: report-jobs plus an exported `pylock.toml` with hashes and an environment the user created (`uv venv` and an install from the lock, run by the user, never by `mentalc`). **Decide by**, on that variant and on the corpus of `resolution.md` E1 to E5: the share of the 11 locked packages each arm mounts, how many are `declared` against `observed`, bytes read and load time. If B mounts a declared tree for under half of the locked packages, defer B until Cargo or Go is scheduled. Whether phases then read these trees at all is the separate experiment E4 in [`resolution.md`](../../resolution.md); source access only has to be able to mount them.

## 7. Proposed changes to `resolution.md` and the guidelines

**The wording change.** The notes support replacing "`mentalc` does not run git" with: **"`mentalc` does not run the git program, nor any program the repository names. It reads git objects as data, in-process, with a reader that runs no hooks, filters, fsmonitor, credential helpers or network, reads no user or system configuration, and turns replace refs off. Pointers the repository controls (alternates, `gitdir:` files, submodule git dirs) are followed only inside the repository root and are recorded."** F-U15 keeps justifying the first sentence; it says nothing against the second. None of the changes below is applied.

| # | Document and place | Today | Proposed |
|---|---|---|---|
| 1 | `resolution.md` [R3](../../resolution.md#r3-mentalc-executes-nothing-the-repository-controls) | "No build tool, package manager, interpreter or script" | add "and no version-control program; git objects are read in-process as data" |
| 2 | `resolution.md` [§6.1](../../resolution.md#61-rung-0-the-repository), submodules paragraph | an empty directory named in `.gitmodules` signals an absent submodule; "reads `.gitmodules` as text and does not run git" | the gitlink commit is the pin; `.gitmodules` is read from the commit's own tree, not the worktree; names are validated; the store is `<git dir>/modules/<name>`; gaps `submodule-absent` and `submodule-commit-missing` with the fix `git submodule update --init`; the URL is recorded and never contacted; the wording change above |
| 3 | `resolution.md` [§6.3](../../resolution.md#63-rung-2-the-observed-environment) | "`mentalc` does not search the machine" | unchanged under S3-A; under S3-B, a sub-rung "verified local artifact" ranked `declared` |
| 4 | `resolution.md` §6.3 | "`.pth` files are not interpreted" | read path lines as data, as ty does; an `import` line is a gap |
| 5 | `resolution.md` [§7.2](../../resolution.md#72-pins) | a pin names a version, a commit or a source | `deps.jsonl` lines gain `tree` (the blake3 tree digest of what was mounted) and `artifact` (the digest as the lockfile spells it), when files were mounted |
| 6 | `resolution.md` [§12.1](../../resolution.md#121-the-rule) | "The one process it starts today is `docker compose config`" | unchanged; add that git object stores are read in-process |
| 7 | `resolution.md` [§12.3](../../resolution.md#123-what-mentalc-does-instead) | "It does not run git. `.gitmodules` is read as text" | the wording change above |
| 8 | `resolution.md` [§12.4](../../resolution.md#124-enforcement) | one tidy test over `Command::new` arguments | add: the git crate's Cargo features are an allowlist (no network, credentials, `attributes`, `worktree-*`); gix calls only in `source/git.rs`; under S1-A, the API bans listed in S1 |
| 9 | `resolution.md` [§14](../../resolution.md#14-other-ecosystems), git submodules row | observed: "the checked-out directory" | "the submodule's object store in `.git/modules/<name>`" |
| 10 | `resolution.md` §15 and §16 | — | add rows 1 to 9 to §15; extend the R3 row of §16 with the new tidy tests and add a "source conformance" row (section 8.9) |
| 11 | [`findings.md`](../../findings.md) F-U15, "Implication" column | "reads `.gitmodules` as text and does not run git" | "does not run the git program; reads git objects as data" |
| 12 | Specification [§6.1](../../specification.md#41-commands-and-exit-codes), inputs | "one repository at one commit" | "one repository at one commit, with its submodules at the commits it records, archives attached in `mentalc.toml`, and dependency trees found by the resolution rungs"; add `--rev` and the worktree mode |
| 13 | Specification [§4.9](../../specification.md#75-storage) | — | add `.mental/sources.jsonl`: one line per mounted root with origin, ids, tree digest, pointers followed and limits applied; no machine paths |
| 14 | Guidelines [README](../../guidelines/README.md#the-workspace-at-a-glance) "blake3 is the only hash in the workspace", [G7](../../guidelines/README.md#g7-stable-hashes), tidy test `lockfile_allowlist` | blake3 is the only hashing crate | "blake3 is the only hash Mental computes for its own identities. The git reader links SHA-1 to address git's objects; a digest a lockfile states is verified with its own algorithm" (SHA-256 only if S3-B wins) |
| 15 | Guidelines README, I/O edges table | `mentalc`: `main.rs`, `cli.rs`, `repo.rs`, `cache.rs`, `ledger.rs`, `backend/*.rs` | add `source/git.rs` and `source/dir.rs` |
| 16 | Guidelines README, clippy sketch | `std::fs::read`, `read_to_string`, `read_dir`, ... | add `std::fs::metadata`, `symlink_metadata`, `read_link` and `canonicalize`, which the directory reader needs and nothing else may call; under S1-A also `gix::open*` and `gix::discover*` |
| 17 | [G16](../../guidelines/README.md#g16-banned-weight) | banned list | add `git2`, `libgit2-sys`, `opendal`, `object_store`, `vfs`, `async_zip`, and every gix network or credential feature |
| 18 | [`mentalc.md` allowlist](../../guidelines/mentalc.md#dependency-allowlist) | 9 normal crates | add the git crate or crate family as one justified entry (S1 decides); `zip` with `default-features = false` and `deflate`; `tar` with `default-features = false`; a gzip decoder (`flate2` with its Rust backend, if gix does not already bring one; unmeasured) |
| 19 | `mentalc.md` module layout | `repo.rs  RepoSnapshot: reads files, runs docker compose config` | add `source/` as in section 8.4; `repo.rs` assembles roots and still runs the compose renderer |
| 20 | `mentalc.md` [C2](../../guidelines/mentalc.md#c2-phases-are-pure-and-take-only-what-they-read) | "an in-memory `RepoSnapshot` loaded once by `repo.rs`" | "an in-memory `Snapshot` of mounted roots, loaded once by `repo.rs`"; phases never see where bytes came from |
| 21 | `mentalc.md` [C12](../../guidelines/mentalc.md#c12-the-cache-key-is-content-never-the-llm) | key from input-slice hashes | no change to the key; add one sentence: commit ids, tree digests and origins never enter it, so the same code at another commit or from another origin hits the cache |
| 22 | [`extract.md`](../../guidelines/extract.md) | `extract(files: &Files, ...)` keyed by path | `Files` entries carry their root kind, so `extract` resolves into library roots but anchors and diagnostics only in project and submodule roots |

## 8. Proposed design for `mentalc`'s source access

**Status: Proposed**, except where it names S1 to S3.

### 8.1 What it does, in order

`repo.rs` loads one `Snapshot` per build, before phase 1, in five steps. It opens the project origin: by default the worktree (what an agent edits), or with `--rev` a commit, optionally at a subdirectory (report-jobs is `examples/report-jobs` inside the mental repository). It reads that root into a `Tree`. It plans the further roots from typed data: the gitlinks of the tree plus `.gitmodules` from the same tree, archives listed in `mentalc.toml`, and dependency trees from the rungs. It reads each planned root with its reader, recursing into submodules with a depth cap. Finally it assembles them into the `Snapshot`, which phases read through pure queries. Planning is pure and reading is edge code, the same split C3 makes for LLM phases. Nothing here walks history, writes to disk, checks out, fetches or runs a program; the compose renderer stays the one process `repo.rs` starts.

### 8.2 Types

```rust
// source.rs: pure; no I/O
pub(crate) struct Snapshot {
    pub roots: Vec<Root>,                         // index is RootId
    pub files: BTreeMap<FileKey, File>,           // every readable file of every root
    pub gaps: Vec<Diagnostic>,                    // input problems found while loading (G5)
}
pub(crate) struct RootId(u16);
pub(crate) struct FileKey { pub root: RootId, pub path: RelPath }
pub(crate) struct Root {
    pub kind: RootKind,                           // Project | Submodule | Library
    pub mount: Mount,
    pub origin: Origin,
    pub digest: TreeDigest,                       // blake3 over the normalised tree (S2)
}
pub(crate) enum Mount {
    Top,                                          // the project namespace
    Under { parent: RootId, at: RelPath },        // a submodule, placed at its gitlink path
    Library { name: String },                     // its own namespace, e.g. "requests@2.32.3"
}
pub(crate) enum Origin {
    Commit   { store: StoreRef, format: ObjectFormat, commit: GitOid, tree: GitOid, subdir: RelPath },
    Gitlink  { name: SubmoduleName, url: String, commit: GitOid, store: StoreRef },
    WorkTree { head: Option<GitOid> },
    Archive  { file: RelPath, format: ArchiveFormat, digest: blake3::Hash, layout: Layout },
    Installed { dist: String, version: String },  // one distribution of an environment
}
pub(crate) struct File { pub digest: blake3::Hash, pub bytes: Box<[u8]>, pub exec: bool, pub git: Option<GitOid> }
pub(crate) struct RelPath(String);                // parsed once (8.6); unrooted, '/'-separated
pub(crate) struct Tree { pub entries: BTreeMap<RelPath, Entry> }   // what every reader returns
pub(crate) enum Entry { File(File), Symlink(String), Gitlink(GitOid), Skipped { size: u64, reason: SkipReason } }
```

`StoreRef` and `RelPath` inside `Origin` are relative to the repository root, so `sources.jsonl` holds no machine paths (G8). The only queries phases get are `Snapshot::file(&FileKey) -> Option<&File>`, `list(root, dir)` (a range scan over the sorted map), `exists(root, path)`, and `locate(path) -> FileKey`, which maps a project-namespace path into the submodule root that owns it. These replace `read_dir`, `metadata` and `canonicalize`, the methods every file-system-shaped interface grew, with no I/O.

### 8.3 Readers: a closed set of functions, no trait

Every reader returns the same `Tree` value plus diagnostics; that value is the seam, as zoekt's `Document` is. There is no `Source` trait: the readers differ in what they take, a trait would have one meaningful implementation per backend and invite plugins, and G3 already forbids single-implementation traits. `Origin` is a closed enum dispatched by `match` in `repo.rs`.

| Function | File | Edge |
|---|---|---|
| `git::open(dir) -> Result<GitStore>`, isolated, unknown `extensions.*` refused | `source/git.rs` | yes |
| `git::resolve(&GitStore, &Rev) -> Result<GitOid>`, `Rev = Head \| Ref(String) \| Oid` | `source/git.rs` | yes |
| `git::read_commit(&GitStore, GitOid, subdir, &Limits) -> Result<(Tree, Vec<Diagnostic>)>` | `source/git.rs` | yes |
| `git::open_submodule(&GitStore, &SubmoduleName) -> Result<Option<GitStore>>` | `source/git.rs` | yes |
| `dir::read_worktree(dir, &FileSet, &Limits) -> Result<(Tree, Vec<Diagnostic>)>` | `source/dir.rs` | yes |
| `dir::read_installed(site_packages, record: &str, &Limits) -> Result<(Tree, Vec<Diagnostic>)>` | `source/dir.rs` | yes |
| `archive::read_zip(&[u8], Layout, &Limits) -> (Tree, Vec<Diagnostic>)` | `source/archive.rs` | no: bytes in |
| `archive::read_tar(&[u8], Compression, Layout, &Limits) -> (Tree, Vec<Diagnostic>)` | `source/archive.rs` | no |
| `gitmodules::parse(&str) -> (Vec<SubmoduleDecl>, Vec<Diagnostic>)` | `source/gitmodules.rs` | no |
| `plan(&Config, &Tree, &[SubmoduleDecl], &Pins) -> Vec<RootSpec>` | `source.rs` | no |
| `assemble(Vec<(Root, Tree)>, &Limits) -> Snapshot` | `source/assemble.rs` | no |

Archive decoding takes bytes, so it is pure and fuzzable; `repo.rs` reads the archive file and hands the bytes over. `Layout` is the mount transform applied once: `StripFirst` (npm, crate, sdist: the single top directory, by position, not by name), `StripExact(prefix)` (Go's `module@version/`, Bazel's `strip_prefix`), `Wheel` (merge `*.data/purelib` and `platlib` into the root and keep the rest of `.data` out of the import namespace), and `AsIs`. An unreadable archive is not a tool failure: its root is absent and every import into it carries the reason `archive-unreadable`.

### 8.4 Module layout and the I/O edge

```text
mentalc/src/
  repo.rs                 [edge] open the project origin, run the plan, read archive bytes, run the compose renderer
  source.rs               Snapshot, Root, Origin, Mount, FileKey, File, Entry, queries, plan()
  source/path.rs          RelPath::parse, component rules, case-fold keys
  source/assemble.rs      symlinks, submodule mounts, collisions, tree digests, limits
  source/archive.rs       zip and tar readers over bytes; layouts
  source/gitmodules.rs    git-config-syntax parser for .gitmodules (and .git/config under S1-B)
  source/git.rs           [edge] the only file that names gix
  source/dir.rs           [edge] worktree and installed-distribution readers; the only std::fs::metadata caller
```

Three edge files, each opting out of G4 with `#[expect(clippy::disallowed_methods, reason = "...")]`. A tidy test asserts that `gix` (or `gix_odb` and its siblings) appears only in `source/git.rs`, and the 12.4 test over `Command::new` keeps covering `repo.rs`.

### 8.5 How submodules and dependency trees are mounted

A **submodule** is a Merkle mount, not an overlay. For each gitlink at path P in the parent tree, find the `.gitmodules` entry whose `path` equals P, read from the blob in the same commit's tree (in worktree mode, the worktree file, refused if it is a symlink). Validate the name (no `..` component, no leading `/`, no `\`, not empty; `gix_validate::submodule::name` from 0.11.1). Open `<common git dir>/modules/<name>` with the same isolated options; in worktree mode only, fall back to a real `P/.git` directory, never a symlink. If no store opens, report `submodule-absent`; if the store lacks the gitlink commit, report `submodule-commit-missing`. Read the child tree at the commit and mount it `Under { parent, at: P }`. Its files appear in the project namespace at `P/...`, so import roots and `PYTHONPATH` settings see them as repository code (resolution.md §6.1), while their `FileKey` names the submodule root for provenance and anchors. Recurse with a depth cap, keyed by (store, commit) to break cycles. Only `path` and `url` are read from `.gitmodules`; `update`, `branch` and the rest are ignored, and the URL is recorded as a `pkg:` identity and never contacted.

A **dependency tree** is a separate root with `Mount::Library { name }` and `RootKind::Library`. It is its own import root; no path from the project reaches into it, and nothing in it reaches the project (rust-analyzer's rule). Phases may resolve names into it; no diagnostic is anchored in it and nothing in it is compiled unless E4 says so. Under S3-A its file set is one distribution's `RECORD` lines resolved against `site-packages`, which splits a shared directory into one tree per distribution and leaves stray files out. Under S3-B it is an archive read with `Layout::Wheel` or `StripFirst`, counted only after its digest matches the lockfile.

### 8.6 Path, archive and pointer safety

These rules live in `source/path.rs` and `source/assemble.rs` and apply to every reader, so every root looks alike.

| Rule | Applies to | Outcome on violation |
|---|---|---|
| `RelPath`: valid UTF-8, `/`-separated, no empty, `.` or `..` segment, no NUL, no `\`, no leading `/`, no Windows drive or reserved name | every entry | `source-path-invalid`; the entry is dropped, the rest of the tree is read |
| `gix_validate::path::component` with `protect_windows`, `protect_hfs` and `protect_ntfs` all on, never taken from repository config; no component equal to `.git` in any case or HFS-ignorable spelling | every entry, archives included | `source-path-invalid` |
| only modes `100644`, `100755`, `120000`, `040000`, `160000` | git trees | `source-mode-unknown` |
| no duplicate names in one directory; unsorted git trees reported | git trees, archives | `source-duplicate-entry`; neither copy is preferred |
| per directory, two names with the same lowercase key are a collision | every root | `source-case-collision` gap; imports never resolve case-insensitively |
| a symlink is data: its target is joined to the link's directory, normalised lexically, and must stay inside its root; at most 40 hops | git trees, tar archives, worktree | `source-symlink-escapes` or `source-symlink-loop`; never followed on disk |
| zip symlink entries and tar device or FIFO entries are not files; in-tree tar hard links are aliases | archives | `archive-entry-invalid` |
| zip local header and central directory must agree; declared sizes must match | zip | `archive-entry-invalid` |
| a symlinked `.gitmodules` or `.gitattributes` is ignored | git trees, worktree | `source-symlinked-gitmodules` |
| a blob under 1 KiB that begins with the LFS pointer line, at a path phases would parse | every root | `lfs-pointer` gap with the fix `git lfs pull` |
| alternates, `gitdir:`, `commondir` and submodule git dirs must resolve under the repository root or a root in `mentalc.toml`; at most 4 pointer hops | git stores | `pointer-outside-root` gap; every pointer followed is written to `sources.jsonl` |
| `core.useReplaceRefs` off; no ancestry walk, so no commit-graph read; no `.gitattributes` filter applied | git stores | — |
| object decoding runs under `catch_unwind` with an object-size cap | git stores | a typed error naming the object (exit 2) |

### 8.7 Limits

Every loop and size has a named cap from `mentalc.toml`, whose values `mentalc init` writes (C18: no `Default`). Hitting one is a `source-limit` diagnostic naming the limit and the root, never a silent truncation, and each is tested at its edges ([T13](../../guidelines/testing.md#t13-test-every-limit-at-its-edges)). The starting values come from Go's module zip rules where one exists; the rest are unmeasured and are to be calibrated in S1.

| Limit | Starting value | Basis |
|---|---|---|
| bytes per file read | 16 MiB; larger files become `Skipped { too_large }`, a gap only if a phase would parse the path | Go's `go.mod` cap; zoekt's skip placeholders |
| archive size, and total uncompressed content per archive | 500 MiB each | Go's `x/mod/zip` |
| compression ratio per archive entry | 200:1 | no studied tool documents one (unverified practice) |
| entries per root | 200,000 | unmeasured |
| tree depth | 64 | unmeasured |
| total bytes per snapshot | 1 GiB | unmeasured |
| submodule depth | 8 | unmeasured |
| symlink hops; pointer hops | 40; 4 | git note proposal |

### 8.8 How gaps and errors are reported

Input problems are data (G5): every rule in 8.6 and every absent root produces a `model::Diagnostic` with a stable `Code`, the tree keeps loading, and the diagnostics go into `Snapshot::gaps`. The resolution gate then treats a source gap exactly like any other cause: an import that lands in an absent submodule or an unreadable archive is `Unresolved` with that root cause, grouped by R6 and counted by R2. Tool failures are `Result` errors that exit 2: the repository or revision not found, an unknown `extensions.*` key or object format, a corrupt pack, an I/O error. This matches G11: a missing input is either a named error or an accounted gap, and nothing is skipped silently.

| Code | When | Fix text |
|---|---|---|
| `submodule-absent` | no store for a gitlink | `git submodule update --init <path>` |
| `submodule-commit-missing` | the store lacks the gitlink commit | `git -C <path> fetch` or `git submodule update` |
| `submodule-name-invalid` | the name fails validation | none; the submodule is not read |
| `pointer-outside-root` | an alternate or git dir resolves outside the allowed roots | add the root to `mentalc.toml` |
| `lfs-pointer` | an LFS pointer where source is expected | `git lfs pull` |
| `archive-unreadable`, `archive-entry-invalid`, `archive-digest-mismatch` | a bad or mismatching archive | name the file and the expected digest |
| `source-path-invalid`, `source-mode-unknown`, `source-duplicate-entry`, `source-case-collision`, `source-symlink-escapes`, `source-symlink-loop`, `source-symlinked-gitmodules` | 8.6 | name the path |
| `source-limit` | 8.7 | name the limit and its config key |

### 8.9 How it is tested

| Layer | What | How |
|---|---|---|
| Phases | never need git or disk | the C19 fixture DSL builds a `Snapshot` directly from `//- path` blocks; a `//- root lib:requests@2.32.3` header adds library roots |
| Conformance | every reader's output obeys the same invariants | one `conform(&Tree)` helper, in the spirit of Go's `fstest.TestFS`, run on every reader's output in every test: sorted, every path a valid `RelPath`, digests match bytes, limits respected, every collision reported |
| Equivalence | one file set gives one tree digest whatever the origin | a proptest generates small file sets, writes them as a git commit (loose objects written by `gix-odb` in a temp dir), a worktree, a zip and a tar.gz, and asserts identical `TreeDigest` |
| Walker and archive rules | each row of 8.6 | one hand-built hostile fixture per row: tree objects with `.git`, `..`, bad modes, case collisions and a symlinked `.gitmodules`; zip and tar bytes with `../`, absolute names, duplicates, the two-entry Zip Slip, header mismatch and a ratio bomb; `.gitmodules` with the advisory names (`a..b/../../../.git/`) |
| Fuzzing | pure decoders never panic | `cargo fuzz` targets for `read_zip`, `read_tar`, `gitmodules::parse` and `RelPath::parse` (CI already builds fuzz targets) |
| Integration | the example | report-jobs from the mental repository at `HEAD` with `subdir = examples/report-jobs` yields 8 files and a snapshotted `sources.jsonl`; the clean worktree yields the same tree digest |
| Submodules | absent and present | a generated superproject with one absorbed submodule, the same with the store removed (`submodule-absent`) and with the commit missing (`submodule-commit-missing`) |
| Determinism and enforcement | G8, G4, 12.4 | two loads give byte-identical `sources.jsonl`; tidy tests for gix call sites and git crate features |

### 8.10 Size

| File | Estimate (non-test lines) |
|---|---|
| `source.rs` (types, queries, plan) | 250 |
| `source/path.rs` | 150 |
| `source/assemble.rs` | 250 |
| `source/archive.rs` | 300 |
| `source/gitmodules.rs` | 120 |
| `source/git.rs` | 300, plus about 150 under S1-B |
| `source/dir.rs` | 150 |
| `repo.rs` additions | 200 |
| **Total** | **about 1,700 (1,850 under S1-B)**, with about 2,000 lines of tests |

That sits between typst-kit's file layer (about 470 lines, with no git and no archives read in place) and rust-analyzer's `vfs` (about 1,300), well under tsgo's VFS (about 2,300), LLVM's (about 4,300) and OpenGrok's history package (18,113). Every file stays under G17's 400 lines. The sizes are estimates from the note sizes and the rules above, not from code.

### 8.11 Other version control systems

The evidence says convert outside, not drive natively: OpenGrok's per-VCS cost, Sourcegraph's "the result must be a git repository", Software Heritage's loaders that "are hooked deeply into their respective VCS' internals" ([SWH loader overview](https://github.com/SoftwareHeritage/swh-loader-core/blob/639eb16bbd543ac47747907013325149c7951845/docs/vcs-loader-overview.rst)), and SonarQube's advice. `mentalc` should ship no Mercurial, Subversion or Perforce driver. Three routes already cover them with no extra code: the worktree origin reads any checkout as a plain directory (recorded with `head: None`, so it is reproducible only on that machine); the archive origin reads an export (`hg archive`, `svn export`); and commit mode works after a conversion the user runs (git-remote-hg, git-svn, p4-fusion). A gap or error on a non-git checkout names these routes in its fix text.

## Open questions

1. **Worktree file set.** Which files make up a worktree root: the paths in the git index (needs `gix-index`, more crates), a directory walk with `mentalc`'s own `.gitignore` subset, or the `HEAD` tree's paths read from disk with untracked files reported as one gap? Report-jobs gives no signal; a repository with a `.venv` inside it does.
2. **Compose in commit mode.** `docker compose config` reads the compose file and its `include`, `extends` and `env_file` targets from disk, so with `--rev` it sees the worktree, not the commit. Proposed: in commit mode, compare the blake3 of every compose-related file on disk with the commit's and fail with a named error when they differ. Is that acceptable, or should commit mode refuse compose entirely until a renderer can take the commit's bytes?
3. **Anchor paths.** Are anchors relative to the repository root or the project subdirectory (`examples/report-jobs`), and how does an anchor name a file in a submodule root, so that identity stays stable when a submodule moves?
4. **Unicode collisions.** The lowercase key from `std` catches case collisions; NFC-normalisation collisions on macOS need a Unicode-normalisation crate. Are they worth a dependency, or a later rule?
5. **SHA-256 repositories.** Enable gix's `sha256` feature (7 more crates on the facade) now, or keep a named error until a user has such a repository?
6. **Default origin.** Is the worktree the right default for `mentalc build` (the agent loop), with `--rev` for reproducible builds, or should a dirty worktree with no `--rev` be an error that names both choices?
