# General-purpose virtual file system abstractions: what kept some small and made others bloated

Researched 2026-11-16. Evidence labels: no label = read by me in the source (code, proto, issue or doc page); "snippet-only" = from a search summary; "unverified" = prior knowledge, not checked this session; "title-only" = only an issue title and comment count were read. Source code was read at `master`/`main` on 2026-11-16 unless a version is given. Crate figures come from the crates.io API (`https://crates.io/api/v1/crates/<name>` and `/<version>/dependencies`) on 2026-11-16. "Lines" is crates.io's `linecounts.total_code_lines` for the published crate, so it leaves out proc-macro and sub-crates. No downloaded code was built or run.

**One line per product** (what it is / interface size and composition / take for mentalc)

- **Go `io/fs`** (Go 1.16, 2021): the standard read-only file tree interface. `FS` has 1 method (`Open`) and `File` has 3; 6 optional extension interfaces exist (`ReadFile`, `ReadDir`, `Stat`, `Sub`, `Glob`, `ReadLink`), and only one was added in 5 years. Composition is `Sub` only. Take: the model to copy.
- **Go `archive/zip.Reader`, `embed.FS`, `fstest.MapFS`, `fstest.TestFS`**: backends and a conformance test for `fs.FS`. Each is small, and one generic test checks every backend. Take: copy the conformance-test idea.
- **`golang.org/x/mod/zip`**: the module-zip format. It is a file list with fixed limits on size, names and paths, not a VFS. Take: check archive inputs at the edge.
- **Java NIO.2 `FileSystemProvider` + `jdk.zipfs`**: a pluggable read-write file system with URIs. Provider 19 abstract methods, `FileSystem` 13, `Path` about 40; zipfs is about 253 KB of source. Take: too big.
- **IntelliJ `VirtualFile`/`VirtualFileSystem`**: the IDE's file layer over a persistent cached snapshot. `VirtualFile` has 59 public methods (15 abstract) and `VirtualFileSystem` 8 abstract; protocols are `file://`, `jar://` and so on. Staleness and refresh are the main pain. Take: avoid a live cache.
- **Eclipse EFS `IFileStore`**: a URI-scheme file store for the IDE. 26 methods, read-write. Take: too big.
- **Apache Commons VFS**: a URI-based VFS with 20 provider packages, including 3 generations of HTTP. `FileObject` has about 40 methods. Take: a cautionary tale.
- **Python `fsspec`**: a protocol registry with chained URLs. `AbstractFileSystem` has 83 methods; 56 protocols are known; about 582M downloads a month. Take: it spread widely, and its caching and async are where it hurts.
- **Rust `vfs`**: a sync VFS with Physical/Memory/Overlay/AltRoot/Embedded backends. The trait has 15 methods, 11 of them for writing or changing files; it carries a parallel async module. Take: too much write surface.
- **Rust `cap-std`**: a capability-based `Dir` handle over the OS. About 3.7k lines and 4 required deps. Take: only for sandboxing the working tree, which is not needed.
- **Rust `rust-embed`/`include_dir`**: compile-time embedded trees. Tiny APIs. Take: irrelevant for inputs, fine for fixtures.
- **Rust `object_store`**: an async get/list/put store (local, S3, GCS, Azure, memory). About 10 core trait methods plus an `Ext` trait; 27k lines; 15 required deps. Take: async, so banned.
- **Rust `opendal`**: an async "access data freely" layer over about 50 services and 26 layers. 104 features; the core needs tokio, and its blocking API runs on tokio. Take: banned weight.
- **Rust `zip`** (9.6.0): a zip read/write crate. 20k lines; 11 required deps and 9 default features. Take: use it with `default-features = false` and deflate only.
- **Rust `rc-zip`**: a zip reader (sans-io, unverified). 3.3k lines but 9 required deps (encoding_rs, chardetng, chrono, tracing...). Take: a smaller alternative to `zip`.
- **Rust `tar`**: a tar read/write crate. 3.5k lines and 2 required deps. Take: fine for tarball artifacts.
- **Rust `async_zip`**: an async zip crate. Take: banned (async).
- **Bazel `vfs.FileSystem` + REAPI CAS**: a path file system (25 abstract methods) plus Merkle `Directory` protos addressed by `Digest{hash,size}`. Take: the Merkle side is what mentalc wants.
- **Buck2 `FileOps`**: the read-only build-input trait. 5 async methods keyed by `CellPath`, so cells act as the mount table. Take: the closest shape to mentalc's needs.
- **Nix store paths**: `/nix/store/<digest>-<name>` immutable objects. Identity is the digest plus a human name. Take: name dependency roots by digest.
- **Plan 9 9P / FUSE**: "everything is a file system". A small protocol with a huge, mutable, live meaning. Take: contrast only.

---

## Which interface shapes stayed small and stable, which grew, and what drove the growth?

### Takeaway
The read-only interfaces with one required method (Go `fs.FS`) or a handful (Buck2 `FileOps`: 5) stayed small and stable for years. Everything that grew was carrying write support, attributes and permissions, a live or remote backend (URIs, auth, retries), async, or change watching. In the Rust crates the growth shows up as dependency weight: opendal's core requires tokio, object_store is async-only, and the `zip` crate pulls in crypto by default.

### Cited Findings

**Go: small and stable**
- `fs.FS` is one method, `Open(name string) (File, error)`. `File` is `Stat`, `Read` and `Close`; `ReadDirFile` adds `ReadDir(n)`; `DirEntry` has 4 methods (`Name`, `IsDir`, `Type`, `Info`). — [io/fs/fs.go](https://github.com/golang/go/blob/master/src/io/fs/fs.go)
- The optional extension interfaces are `ReadFileFS.ReadFile`, `ReadDirFS.ReadDir`, `StatFS.Stat`, `SubFS.Sub`, `GlobFS.Glob` and `ReadLinkFS{ReadLink, Lstat}`, each 1 or 2 methods. — [readfile.go](https://github.com/golang/go/blob/master/src/io/fs/readfile.go), [readdir.go](https://github.com/golang/go/blob/master/src/io/fs/readdir.go), [stat.go](https://github.com/golang/go/blob/master/src/io/fs/stat.go), [sub.go](https://github.com/golang/go/blob/master/src/io/fs/sub.go), [glob.go](https://github.com/golang/go/blob/master/src/io/fs/glob.go), [readlink.go](https://github.com/golang/go/blob/master/src/io/fs/readlink.go)
- In 5 years only one extension was added. `ReadLinkFS` was filed 2021-11-14 and closed 2025-02-03 after 101 comments, and it landed in Go 1.25 ("a new API that would be safe to land in the next upcoming major Go release, Go 1.25"). — [golang/go#49580](https://github.com/golang/go/issues/49580)
- The design says: "The `FS` interface defines the _minimum_ requirement for an implementation: just an `Open` method"; "Previous attempts at file system interfaces floundered in the complexity of defining a complete set of operations"; "This design reduces the necessary implementation to an absolute minimum, with the extension pattern allowing the provision of new functionality". — [Go draft design: file system interfaces](https://go.googlesource.com/proposal/+/master/design/draft-iofs.md)
- On why `http.FileSystem` was not reused: "every `File` need not be required to implement `Seek` and `Readdir`." On the earlier godoc-derived VFS: it "seemed too complex to be the right common abstraction". — [draft-iofs.md](https://go.googlesource.com/proposal/+/master/design/draft-iofs.md)
- In the accepting proposal, rsc said optional interfaces are "an established pattern in Go that we understand how to use well (informed in part by some earlier mistakes, such as optional interface like http.Hijacker with methods that cannot return an error to signal failure/unavailability)". It was accepted for Go 1.16 (closed 2021-02-11, 115 comments). — [golang/go#41190](https://github.com/golang/go/issues/41190)
- Path rules keep backends trivial: "paths are unrooted, slash-separated sequences of path elements", "the separator is a forward slash on all systems, even Windows", "`FS` path names never contain a '`.`' or '`..`' element", and the root is "`.`". — [draft-iofs.md](https://go.googlesource.com/proposal/+/master/design/draft-iofs.md)
- `archive/zip.Reader` implements `fs.FS` by building a file list once (`initFileList`). It synthesizes the directories the zip leaves implicit (`dirs := make(map[string]bool)`) and rejects ambiguous archives (`": duplicate entries in zip file"`). — [archive/zip/reader.go](https://github.com/golang/go/blob/master/src/archive/zip/reader.go)
- `fstest.TestFS(fsys fs.FS, expected ...string) error` "walks the entire tree of files in fsys, opening and checking that each file behaves correctly", so one generic test checks any backend. — [testing/fstest/testfs.go](https://github.com/golang/go/blob/master/src/testing/fstest/testfs.go)
- `embed.FS` implements `Open`, `ReadDir` and `ReadFile`; `fstest.MapFS` is a `map[string]*MapFile`. (unverified)

**Buck2: a small read-only input trait**
- `pub trait FileOps: Send + Sync` has 5 methods: `read_file_if_exists(CellPath) -> Option<String>`, `read_dir(CellPath)`, `is_ignored(CellPath)`, `read_path_metadata_if_exists(...)` and `buildfiles(CellName)`. All are async; there is no write. — [buck2 file_ops/trait_.rs](https://github.com/facebook/buck2/blob/main/app/buck2_common/src/file_ops/trait_.rs)

**Interfaces that grew: Java, IDEs, Commons VFS, fsspec, Bazel**
- Java NIO.2 `FileSystemProvider` has 19 `public abstract` methods (`getScheme`, `newFileSystem`, `getFileSystem`, `getPath`, `newByteChannel`, `newDirectoryStream`, `createDirectory`, `delete`, `copy`, `move`, `isSameFile`, `isHidden`, `getFileStore`, `checkAccess`, `getFileAttributeView`, 2× `readAttributes`, `setAttribute`...). `java.nio.file.FileSystem` has 13 abstract methods and `Path` about 40 methods. — [FileSystemProvider.java](https://github.com/openjdk/jdk/blob/master/src/java.base/share/classes/java/nio/file/spi/FileSystemProvider.java), [FileSystem.java](https://github.com/openjdk/jdk/blob/master/src/java.base/share/classes/java/nio/file/FileSystem.java), [Path.java](https://github.com/openjdk/jdk/blob/master/src/java.base/share/classes/java/nio/file/Path.java)
- `jdk.zipfs` is 13 source files totalling about 253 KB, and `ZipFileSystem.java` alone is 141 KB. It is a read-write zip file system, so it carries in-place update logic. — [jdk.zipfs sources](https://github.com/openjdk/jdk/tree/master/src/jdk.zipfs/share/classes/jdk/nio/zipfs)
- IntelliJ `VirtualFile.java` is 739 lines with 59 public methods, 15 of them abstract. `VirtualFileSystem.java` has 8 abstract methods. — [VirtualFile.java](https://github.com/JetBrains/intellij-community/blob/master/platform/core-api/src/com/intellij/openapi/vfs/VirtualFile.java), [VirtualFileSystem.java](https://github.com/JetBrains/intellij-community/blob/master/platform/core-api/src/com/intellij/openapi/vfs/VirtualFileSystem.java)
- Eclipse EFS `IFileStore` has 26 public methods in 672 lines. — [IFileStore.java](https://github.com/eclipse-platform/eclipse.platform/blob/master/resources/bundles/org.eclipse.core.filesystem/src/org/eclipse/core/filesystem/IFileStore.java)
- Apache Commons VFS ships 20 provider packages: `bzip2, compressed, ftp, ftps, gzip, http, http4, http4s, http5, http5s, https, jar, local, ram, res, sftp, tar, temp, url, zip`. Its `FileObject` interface has about 40 methods. — [commons-vfs provider dir](https://github.com/apache/commons-vfs/tree/master/commons-vfs2/src/main/java/org/apache/commons/vfs2/provider), [FileObject.java](https://github.com/apache/commons-vfs/blob/master/commons-vfs2/src/main/java/org/apache/commons/vfs2/FileObject.java)
- `fsspec.AbstractFileSystem` defines 83 methods in a 2,465-line `spec.py`, and `registry.known_implementations` lists 56 protocols. — [fsspec/spec.py](https://github.com/fsspec/filesystem_spec/blob/master/fsspec/spec.py), [fsspec/registry.py](https://github.com/fsspec/filesystem_spec/blob/master/fsspec/registry.py)
- fsspec has 1,385 GitHub stars and 456 open issues (repo created 2018-04-23), but about 582M PyPI downloads in the last month (15.1M in the last day). It spreads as a transitive dependency. — [GitHub API repos/fsspec/filesystem_spec](https://github.com/fsspec/filesystem_spec), [pypistats fsspec](https://pypistats.org/packages/fsspec)
- fsspec is pulled in by pandas, dask, Hugging Face `datasets` and others. (unverified)
- Bazel `vfs/FileSystem.java` is 939 lines with 25 abstract methods. Read-side methods include `stat`, `getDirectoryEntries`, `getInputStream`, `readSymbolicLink` and `getFileSize`. Half (12) change state: `createDirectory`, `createDirectoryAndParents`, `delete`, `setLastModifiedTimeNanos`, `createSymbolicLink`, `setReadable/Writable/Executable`, `getOutputStream`, `createReadWriteByteChannel`, `renameTo`, `createFSDependentHardLink`. It also has capability probes (`supportsModifications`, `supportsSymbolicLinksNatively`, `mayBeCaseOrNormalizationInsensitive`). — [bazel vfs/FileSystem.java](https://github.com/bazelbuild/bazel/blob/master/src/main/java/com/google/devtools/build/lib/vfs/FileSystem.java)

**Rust crates: measured weight**

| Crate (version) | Downloads total / recent 90d | Lines | Required / optional normal deps | Notes |
|---|---|---|---|---|
| `vfs` 0.13.0 | 3.66M / 0.87M | 6,533 | 1 (`filetime`) / 7 (tokio, async-std, async-trait...) | `FileSystem` trait has 15 methods: `read_dir, create_dir, open_file, create_file, append_file, metadata, set_{creation,modification,access}_time, exists, remove_file, remove_dir, copy_file, move_file, move_dir`; 11 of 15 write. Backends: `physical, memory, overlay, altroot, embedded`, plus a parallel `async_vfs` module. — [vfs filesystem.rs](https://docs.rs/vfs/0.13.0/src/vfs/filesystem.rs.html) |
| `cap-std` 4.0.3 | 34.0M / 17.1M | 3,692 | 4 (`cap-primitives, io-extras, io-lifetimes, rustix`) / 2 | [crates.io](https://crates.io/crates/cap-std) |
| `rust-embed` 8.14.0 | 68.8M / 21.9M | 55 (facade) | 3 / 10 (actix-web, axum, rocket, warp...) | web-framework integrations are optional deps — [crates.io](https://crates.io/crates/rust-embed) |
| `include_dir` 0.7.4 | 87.3M / 22.4M | 236 | 1 (macros) / 1 (glob) | last release 2024-06-17 — [crates.io](https://crates.io/crates/include_dir) |
| `object_store` 0.14.2 | 111.1M / 31.3M | 27,084 | 15 (async-trait, futures-*, chrono, http, url, itertools, parking_lot, wasm-bindgen-futures...) / 21 (reqwest, hyper, tokio, ring, quick-xml...) | `default = ["fs"]`; the core trait has `put_opts, put_multipart_opts, get_opts, get_ranges, delete_stream, list, list_with_offset, list_with_delimiter, copy_opts, rename_opts`, and the convenience methods `get, get_range, head, put, delete, copy, rename...` live in `ObjectStoreExt`. — [object_store lib.rs](https://docs.rs/object_store/0.14.2/src/object_store/lib.rs.html) |
| `opendal` 0.59.5 | 20.7M / 7.1M | 339 (facade) | 1 (`opendal-core`) / 93 | 104 features; defaults are `auto-register-services, http-transport-reqwest, executors-tokio, layers-{concurrent-limit,logging,retry,timeout}`. Optional deps: about 26 `opendal-layer-*` and about 50+ `opendal-service-*` crates (s3, gcs, azblob, ftp, sftp, hdfs, redis, mysql, postgresql, rocksdb, gdrive, dropbox, github, hf, ipfs...). — [crates.io deps](https://crates.io/crates/opendal/0.59.5/dependencies) |
| `opendal-core` 0.59.5 | 5.1M | 22,275 | 20 required, **including `tokio`**, `futures`, `http`, `url`, `quick-xml`, `serde_json`, `md-5`, `base64`, `uuid`, `jiff`, `anyhow` | `Capability` struct has 91 public fields; 91 RFCs in `src/docs/rfcs`. — [crates.io](https://crates.io/crates/opendal-core), [Capability docs](https://docs.rs/opendal/latest/opendal/struct.Capability.html) |
| `zip` 9.6.0 | 347.9M / 106.0M | 20,011 | 11 (`constant_time_eq, crc32fast, getrandom, indexmap, libc, memchr, nix, typed-path, windows-sys, zeroize`) / 19 | `default = [aes-crypto, bzip2, deflate64, deflate, lzma, ppmd, time, zstd, xz]`; 152 versions — [crates.io](https://crates.io/crates/zip) |
| `rc-zip` 5.4.1 | 2.5M / 0.77M | 3,260 | 9 (`chardetng, chrono, crc32fast, encoding_rs, oem_cp, oval, ownable, tracing, winnow`) / 7 | [crates.io](https://crates.io/crates/rc-zip) |
| `tar` 0.4.46 | 291.7M / 80.3M | 3,509 | 2 (`filetime, libc`) / 1 (`xattr`, default) | [crates.io](https://crates.io/crates/tar) |
| `async_zip` 0.0.19 | 8.1M / 2.6M | 4,566 | 5 (`binrw, crc32fast, futures-lite, pin-project, thiserror`) / 5 (tokio...) | [crates.io](https://crates.io/crates/async_zip) |

### Inferences
- The things that drove growth, in order of evidence: (1) **write and mutation**: 11 of 15 `vfs` methods, 12 of 25 Bazel abstract methods, and zipfs's 141 KB file. (2) **attributes and permissions**: NIO `readAttributes`/`setAttribute`/`checkAccess`, Bazel `setExecutable`. (3) **remote services**: opendal's 50+ services, object_store's reqwest/ring/quick-xml, and Commons VFS's 3 HTTP generations. (4) **async**: object_store and opendal are async-first, and `vfs` and fsspec carry parallel async paths. (5) **live-FS coherence**: IntelliJ refresh and events. (6) **capability negotiation** once backends differ: opendal's 91-field `Capability`, Bazel's `supports*`.
- Method count alone misleads. opendal's public surface is small per call, but its weight sits in features and services. Measure dependency closure and lines, not only methods.
- For mentalc: a read-only trait with 2 or 3 methods (`read(path) -> Option<bytes>`, `list(dir) -> sorted entries`, maybe `stat`) matches what stayed stable (Go, Buck2). Every crate in the table except `tar`, `rc-zip` and `zip` with `default-features = false` breaks G16 (async or tokio) or brings far more than the reading mentalc needs.

### Gaps
- I did not run `cargo tree`, so the numbers above are direct dependencies only, not transitive closure. For opendal with default features the closure clearly includes tokio and reqwest, but the total crate count is unmeasured.
- I did not count the methods of opendal's internal `Access` trait (the grep missed it); the RFC excerpt below shows its shape.
- I did not read the history of Eclipse EFS or Commons VFS releases to date when their surfaces grew.

---

## Read-only immutable snapshot vs mutable live FS: how much simpler is the read-only case?

### Takeaway
It is much simpler. The read-only snapshot designs need 1 to 5 methods and no coherence machinery. The mutable live designs need 15 to 83 methods plus refresh, events, locking and capability probes, and the Go team declined to standardize writes even after 157 comments because the semantics (open modes) are not portable. Pain from a live FS concentrates in cache staleness (IntelliJ) and cache concurrency (fsspec).

### Cited Findings
- Go deliberately excluded writing: "Although the initial interface is limited to read-only file systems, the design can be extended to support write operations later, even from third-party packages", and "The `FS` in this design also provides no way to open a file for writing." — [draft-iofs.md](https://go.googlesource.com/proposal/+/master/design/draft-iofs.md)
- The writable-interfaces proposal was **declined** on 2023-08-30 after 157 comments. rsc: "the very first interface poses a problem: what are the open modes and where are they defined? ... io/fs is meant to be a completely portable API, independent of 'os'... it's a sign that maybe we don't have the right answers yet. I still haven't seen a compelling reason this needs to be in the standard library now." — [golang/go#45757](https://github.com/golang/go/issues/45757)
- IntelliJ's VFS is a live snapshot with a cache. "the VFS manages a persistent snapshot of some of the user's hard disk contents"; "All VFS access operations go through the snapshot", "which may not always match the disk's actual contents"; "deleted files can still be visible in the UI for some time"; "There is currently no facility for removing files from the snapshot"; "Refresh operations are based on file timestamps", so a change without a timestamp change "will not pick up the updated contents"; "In some cases, synchronous refreshes can cause deadlocks"; "a refresh operation fires events only for changes in files that have been loaded in the snapshot." — [IntelliJ Platform SDK: Virtual File System](https://plugins.jetbrains.com/docs/intellij/virtual-file-system.html)
- IntelliJ also needs an event system ("VFS events are always fired in the event dispatch thread and in a write action", `BulkFileListener`) and still says "The VFS itself does not honor ignored files". — [IntelliJ VFS docs](https://plugins.jetbrains.com/docs/intellij/virtual-file-system.html)
- Bazel's mutable `FileSystem` needs probes such as `supportsModifications(PathFragment)` and `mayBeCaseOrNormalizationInsensitive()`. — [bazel FileSystem.java](https://github.com/bazelbuild/bazel/blob/master/src/main/java/com/google/devtools/build/lib/vfs/FileSystem.java)
- fsspec issue titles show where its live, cached, remote design hurts: "Cached filesystem not concurrency-safe?" (#1107, 35 comments), "Directories and delimiter handling" (#562, open, 22 comments), "pd.read_csv(s3_path) fails with 'TypeError: 'coroutine' object is not subscriptable'" (#385, 28 comments), "GCSFilesystem creation is surprisingly slow" (#1768, open). (title-only) — [#1107](https://github.com/fsspec/filesystem_spec/issues/1107), [#562](https://github.com/fsspec/filesystem_spec/issues/562), [#385](https://github.com/fsspec/filesystem_spec/issues/385), [#1768](https://github.com/fsspec/filesystem_spec/issues/1768)
- Read-only zip backends are small. Go's `archive/zip` builds an in-memory file list once and serves `Open` from it. — [archive/zip/reader.go](https://github.com/golang/go/blob/master/src/archive/zip/reader.go)
- By contrast, the read-write `jdk.zipfs` is about 253 KB of source. — [jdk.zipfs](https://github.com/openjdk/jdk/tree/master/src/jdk.zipfs/share/classes/jdk/nio/zipfs)
- The `rc-zip` reader is 3.3k lines, against 20k for the read-write `zip` crate. — [crates.io rc-zip](https://crates.io/crates/rc-zip), [crates.io zip](https://crates.io/crates/zip)

### Inferences
- A git commit, a zip and a published package artifact are all immutable once identified. For mentalc the whole coherence layer (refresh, watch, events, staleness, locking) simply disappears, which removes IntelliJ's and fsspec's main failure modes.
- Read-only also removes the "open modes" question that stalled Go's writable proposal, plus permissions, attribute setters, and the half of Bazel's abstract methods that change state.
- The only place "live" shows up for mentalc is the working tree (uncommitted files), if it is ever supported. Keeping it as one more backend that takes a snapshot once (read and hash every file up front) keeps the layer free of live semantics.

### Gaps
- I found no published measurement such as "read-only cut N% of code" for any single project. The evidence is comparative: different projects, similar scope.

---

## Composition: overlay/union, sub-tree, mount table, chained URL. Which does "repo commit + submodules + dependency archives" need?

### Takeaway
Go offers only `Sub` (go into a subtree), and that has been enough. Buck2 composes with a mount table of named cells (`CellPath = cell + relative path`). fsspec and Commons VFS compose by chaining URL schemes, and Rust `vfs` offers overlay and alt-root. A repo commit with submodules and dependency archives needs a mount table, meaning disjoint prefixes or named roots, each backed by one immutable tree. Overlay and union are not needed, since nothing shadows anything, and chained URLs add a parsing language without adding capability.

### Cited Findings
- Go `SubFS.Sub(dir string) (FS, error)` is the only built-in composition; `fs.Sub` gives a view rooted at a subdirectory. — [io/fs/sub.go](https://github.com/golang/go/blob/master/src/io/fs/sub.go)
- Buck2 `FileOps` takes `CellPath` everywhere and `buildfiles(cell: CellName)`. Inputs are addressed as (cell, path), and the cell acts as the mount table. — [buck2 file_ops/trait_.rs](https://github.com/facebook/buck2/blob/main/app/buck2_common/src/file_ops/trait_.rs)
- Rust `vfs` backends are `physical, memory, overlay, altroot, embedded`. Overlay is layered and writable-on-top, and altroot is a chroot. — [vfs 0.13.0 src/impls](https://docs.rs/vfs/0.13.0/src/vfs/impls/mod.rs.html) (layering semantics unverified)
- Bazel REAPI composes trees structurally. A `Directory` holds `repeated FileNode files`, `repeated DirectoryNode directories` and `repeated SymlinkNode symlinks`, and each child directory is referenced by digest, so mounting a subtree just means placing a `DirectoryNode{name, digest}`. — [remote_execution.proto](https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto)
- fsspec chains URLs such as `zip::s3://bucket/a.zip` and `filecache::s3://...`, so composition is string syntax resolved by the protocol registry. (unverified; the 56-entry `known_implementations` registry was read: [registry.py](https://github.com/fsspec/filesystem_spec/blob/master/fsspec/registry.py))
- Commons VFS nests the same way with layered URIs such as `zip:http://host/a.zip!/dir/file`. (unverified; the provider list was read: [commons-vfs providers](https://github.com/apache/commons-vfs/tree/master/commons-vfs2/src/main/java/org/apache/commons/vfs2/provider))
- IntelliJ also uses a layered syntax: `jar://<path to jar>!/<path inside jar>`. (unverified)

### Inferences
- **Submodules**: in git a submodule is a tree entry of type gitlink (mode 160000) pointing to a commit in another repository (unverified, standard git). The natural model is a mount: the outer commit's tree has a node at `path/to/sub` that resolves to (repo B, commit X). This is Merkle-style placement (REAPI `DirectoryNode`), not overlay.
- **Dependency archives**: each is its own root (a zip or tarball by digest), mounted under a named prefix or cell such as `deps/<name>@<version>`. They never merge with the repo tree.
- **Overlay/union** matters only for "commit plus uncommitted edits". That is out of scope for the snapshot input layer and better handled as a separate snapshot backend.
- **Chained URLs** (`zip::s3://`) are the source of fsspec's and Commons VFS's parsing and nesting complexity. A typed enum (`Source::GitCommit{..} | Source::Zip{digest} | Source::Tarball{digest}`) plus a mount table gives the same reach with no string grammar.

### Gaps
- I did not read the `OverlayFS` source in the `vfs` crate to confirm its write and whiteout semantics.
- I did not verify fsspec chaining syntax against its docs this session.

---

## Identity: path vs URI vs content hash. Who uses hashes for caching, and what did it buy?

### Takeaway
Build systems that must cache (Bazel REAPI, Nix) identify files and trees by content digest and treat paths as names inside a Merkle directory. IDE and general VFSs (IntelliJ, NIO, EFS, Commons VFS, fsspec) identify by path or URI and pay for it with staleness, refresh and equality puzzles. A git commit already is a Merkle tree with content ids, so mentalc gets hash identity for free on its main input and only needs to hash archives once.

### Cited Findings
- REAPI `Digest` is `{ string hash; int64 size_bytes }`. `Directory` is a message of named file, directory and symlink nodes plus `NodeProperties` (which can carry `mtime`). `Tree` is `root` plus all `children` directories, and "Servers SHOULD ensure that these are ordered consistently such that two actions producing equivalent output directories ... also produce Tree messages with matching digests." — [remote_execution.proto](https://github.com/bazelbuild/remote-apis/blob/main/build/bazel/remote/execution/v2/remote_execution.proto)
- Bazel's path-based `FileSystem` still exposes `getDigest(PathFragment)` and `getFastDigest(PathFragment)` (non-abstract), so the local path FS is fitted with digest hooks for caching. — [bazel FileSystem.java](https://github.com/bazelbuild/bazel/blob/master/src/main/java/com/google/devtools/build/lib/vfs/FileSystem.java)
- Nix store paths are pairs of "A 20-byte digest for identification" and "A symbolic name for people to read", as in `/nix/store/a040m110amc4h71lds2jmr8qrkj2jhxd-git-2.38.1`; "A store path will always reference exactly one store object." "Exactly how the digest is calculated depends on the type of store path" (input-addressed vs content-addressed). — [Nix manual: Store Path](https://nix.dev/manual/nix/2.28/store/store-path)
- IntelliJ identity is object-based: "multiple `VirtualFile` instances may correspond to the same disk file. They are equal, have the same `hashCode`, and share the user data." Change detection is by timestamp, not content. — [IntelliJ VFS docs](https://plugins.jetbrains.com/docs/intellij/virtual-file-system.html)
- Go `io/fs` has no identity notion beyond the path string, which is enough because it does not cache. — [io/fs/fs.go](https://github.com/golang/go/blob/master/src/io/fs/fs.go)
- Go module zips identify a module by (path, version) and checksum it in `go.sum`. `x/mod/zip` enforces `MaxZipFile = 500 << 20`, `MaxGoMod = 16 << 20`, `MaxLICENSE = 16 << 20` and errors such as `"file is in vendor directory"`, so archive content is validated, not trusted. — [golang.org/x/mod/zip/zip.go](https://github.com/golang/mod/blob/master/zip/zip.go) (the go.sum hash part is unverified)

### Inferences
- What hashes bought: cache hits across machines and runs (REAPI), sharing and immutability guarantees (Nix), and cheap equality of whole trees (compare root digests). IntelliJ shows what happens without them: timestamp-based refresh that misses same-timestamp edits.
- For mentalc: identify each mounted root by a digest (git commit or tree oid; blake3 of the archive bytes per G7) and each file by (root digest, normalized path) or by its blob id. The git blob oid is SHA-1 or SHA-256 (git's choice, unverified), while mentalc's own stored hashes are blake3, so keep "source-native id" and "mentalc cache key" as distinct fields.
- Go's path rules (unrooted, slash-separated, no `.` or `..`, root is `.`) are worth adopting verbatim as the normalized path inside every root. They remove Windows and zip-slip ambiguity at the edge.

### Gaps
- I did not fetch the Buck2 DICE docs to confirm how it keys file-ops results (digest vs path plus invalidation). I believe DICE invalidates by path on file-watcher events, which would be path identity plus explicit invalidation. (unverified)
- I did not fetch Pants' docs; it reportedly uses a REAPI-compatible CAS with `Digest`/`Snapshot` types. (unverified)

---

## Concrete warnings: regrets, rewrites, deprecations

### Takeaway
The documented regrets are about supporting two execution models at once (opendal removed native blocking and now needs tokio even for blocking calls), churning remote transports (Commons VFS's deprecated HTTP providers), Go's early optional-interface mistake (`http.Hijacker`), and the cost of live snapshots (IntelliJ). The steady pattern is that the minimal read-only design (Go) was never rewritten, and only one interface was added.

### Cited Findings
- **opendal RFC 6189 "remove_native_blocking"** (2025-05-15): offering both native async and blocking carried a "Huge maintenance burden: We need to maintain two versions of the same code" and "Increased complexity". "Only a few services have native blocking support, such as `fs`, `memory` and `moka`", and the result is that "An async runtime is now required to use the blocking APIs." All `blocking_*` methods were removed from `Access`. — [apache/opendal#6189](https://github.com/apache/opendal/pull/6189) (read in `opendal-core-0.59.5/src/docs/rfcs/6189_remove_native_blocking.md`)
- opendal ships 91 RFCs. Many are interface churn: `0409_accessor_capabilities`, `0627_split_capabilities`, `2852_native_capability`, `7700_capability_api`, `1477_remove_object_concept`, `5314_remove_metakey`, `3017_remove_write_copy_from`, `0203_remove_credential`, `0112_path_normalization`, `7799_path_normalization_and_secure_hardening`, and list-API RFCs 2774/2779/3232/3243/3526/3574/5495. (titles only) — [opendal RFC index](https://github.com/apache/opendal/tree/main/core/core/src/docs/rfcs) (directory listing read from the crate source; GitHub path unverified)
- **Commons VFS**: `provider/http/HttpFileProvider` is `@Deprecated` with "Use {@link org.apache.commons.vfs2.provider.http5}", and `http4`/`http4s`/`http5`/`http5s` sit next to it. That is three HTTP client generations kept side by side. — [HttpFileProvider.java](https://github.com/apache/commons-vfs/blob/master/commons-vfs2/src/main/java/org/apache/commons/vfs2/provider/http/HttpFileProvider.java), [provider dir](https://github.com/apache/commons-vfs/tree/master/commons-vfs2/src/main/java/org/apache/commons/vfs2/provider)
- **Go**: the optional-interface design was shaped by "earlier mistakes, such as optional interface like http.Hijacker with methods that cannot return an error to signal failure/unavailability", and `net/http`'s own `FileSystem` "must be left for compatibility". — [golang/go#41190](https://github.com/golang/go/issues/41190), [draft-iofs.md](https://go.googlesource.com/proposal/+/master/design/draft-iofs.md)
- **Go**: an `ioutil.ReadFile`-to-`fs.ReadFile` migration issue stayed open from 2021-02-16 to 2025-04-11. — [golang/go#44286](https://github.com/golang/go/issues/44286) (title-only)
- **Go**: `os.FS()` was proposed and closed in favour of the existing `os.DirFS`. — [golang/go#47803](https://github.com/golang/go/issues/47803) (title-only; resolution unverified)
- **object_store** moved the convenience methods (`get`, `put`, `head`, `delete`, `copy`, `rename`...) off the core trait into `ObjectStoreExt`, leaving `*_opts` forms as the methods implementors write. That is a recent surface reshuffle. — [object_store 0.14.2 lib.rs](https://docs.rs/object_store/0.14.2/src/object_store/lib.rs.html) (the version in which it happened is unverified)
- **IntelliJ**: "There is currently no facility for removing files from the snapshot"; synchronous refresh "can cause deadlocks". — [IntelliJ VFS docs](https://plugins.jetbrains.com/docs/intellij/virtual-file-system.html)
- **Rust `vfs`** keeps a whole parallel `async_vfs` module next to the sync one, the same dual-model cost opendal removed. — [vfs 0.13.0 src](https://docs.rs/vfs/0.13.0/src/vfs/lib.rs.html)
- **Plan 9 / 9P and FUSE (contrast)**: 9P2000 has about 13 request types (version, auth, attach, flush, walk, open, create, read, write, clunk, remove, stat, wstat). The protocol is tiny, yet a 9P or FUSE server must implement a live, mutable, permissioned, concurrent file system, and FUSE's low-level ops table runs to dozens of callbacks. A small protocol does not give small semantics. (unverified)

### Inferences
- For mentalc, the warnings point one way. Do not take opendal, object_store, fsspec-style URLs, or a general VFS crate. Write a closed set of 3 or 4 backends in its own edge files (G4): git commit (with gitlink mounts), zip, tarball, and maybe directory snapshot. Put them behind a sync read-only trait of 2 or 3 methods, a typed mount table, and one conformance test in the style of `fstest.TestFS` that every backend must pass.
- If a backend later needs network access (fetching another repository or a package), do the fetch as a separate step that produces an immutable local artifact by digest, then read it with the same zip, tar or git backend. That is the Nix and REAPI split between fetching and reading, and it keeps auth, retries and HTTP out of the file-access layer. opendal's and Commons VFS's history shows those concerns are what bloats it.

### Gaps
- I did not find a written post-mortem for IntelliJ's persistent VFS cache (for example index-corruption bugs) or for Eclipse EFS; the pain evidence for them is limited to the official limitation statements above.
- I did not read the IntelliJ `jar://` handler (`ArchiveFileSystem`) source to compare archive-handling size with Go's.
