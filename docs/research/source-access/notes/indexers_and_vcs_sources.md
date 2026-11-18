# How code indexers, search engines and analysers get source files out of version control and archives

As of 2026-11-16. Evidence labels: unlabelled = read (source at the pinned commit, or the page); "snippet-only" = from a search summary; "unverified" = prior knowledge.

Summary, one line per product (what it reads / how it abstracts sources / take for mentalc):

- **Sourcegraph** — git only on the inside; Perforce, packages (JVM, npm, Go, Python, Ruby, Rust) are turned into bare git repos by `VCSSyncer`s; analysis asks gitserver for `(repo, commit, path)` or a tar/zip `Archive` of a treeish / **take:** "convert everything to one internal format at the edge" works; keep the converter interface tiny (Clone, Fetch).
- **zoekt** — reads branch trees straight from the git object DB (go-git walk + `git cat-file --batch`), never checks out; also indexes tar/zip archives and plain dirs; all feed one `index.Builder.Add(Document{Name, Content, Branches, SubRepositoryPath})` / **take:** the best template for mentalc: a source produces `(path, blob id, bytes)` and the builder never knows where it came from.
- **OpenGrok** — indexes a checked-out working tree; 15 native `Repository` drivers (Git via JGit, the rest via each VCS's CLI) only for history, annotate and old revisions; ~18k lines in the history package / **take:** the cautionary tale; native multi-VCS is costly and still needs a checkout.
- **Software Heritage** — dozens of loaders (git, hg, svn, ~20 package ecosystems, tarballs) normalise everything into one git-compatible Merkle DAG (content, directory, revision, release, snapshot, origin) with SWHIDs; native IDs kept as `ExtID`s / **take:** the "many sources, one model" reference; identify trees by git-compatible hashes, so a zip and a git commit with the same files get the same directory ID.
- **GitHub code search (Blackbird)** — crawls blobs from git on push events, shards by blob object ID, dedups by blob SHA and delta-indexes repos against similar repos / **take:** content-addressed blob IDs are the dedup and cache key, not (repo, path).
- **CodeQL** — reads the checked-out source root (or follows the build); copies sources into the database as a "source reference" (zip, directory, or bare git repo) / **take:** the analysis artifact carries its own sources snapshot; mentalc's model could carry the blob IDs (or the blobs) it used.
- **Kythe** — VName `(corpus, root, path)` for location, SHA-256 digest for content; kzip archives hold compilation units plus files by digest; a `revision` marker per compilation / **take:** separate "where" (corpus/root/path) from "what" (content digest).
- **Glean** — a DB is `<name>/<hash>` where the hash is often the indexed repo revision; `src.File` is just a path string / **take:** revision lives once at the DB level; per-file facts stay path-only.
- **livegrep** — libgit2 tree walk at given revisions, optional submodule recursion, no checkout; or `fs_paths` directories (unverified for the latter) / **take:** same shape as zoekt, smaller.
- **Hound** — `Driver` interface (git, hg, svn, bzr, local) that clones/pulls a working directory, then walks the filesystem / **take:** multi-VCS is cheap if you only need "make a checkout", but it costs a checkout.
- **Sourcebot** — wraps zoekt; connections are git hosts plus `file://` local git repos / **take:** confirms zoekt's gitindex as a reusable layer.
- **GitLab Gitaly** — gRPC service over bare repos: `TreeEntry(revision, path)`, `GetBlobs([(revision, path)])`, `ListBlobs(revisions)`, `GetTreeEntries` / **take:** the canonical "file at (commit, path)" API; note it warns that GetBlobs returns submodules as commits.
- **Semgrep** — scans the working tree (`git ls-files`); `--baseline-commit` creates a temporary `git worktree` (a full checkout) of the baseline / **take:** the simple way, at the cost of disk and time.
- **SonarQube** — scans the working tree; SCM plugin (Git via JGit, or native git blame; SVN built in) only adds blame / **take:** VCS as metadata, not as file source.
- **PyDriller** — GitPython over a local or temp-cloned repo; file contents before/after a commit come from blob streams, `checkout()` is an explicit opt-in / **take:** history analysis does not need checkouts.
- **CodeScene** — git only (clone over ssh/http/git or a local repo root) (snippet-only) / **take:** history analysers skip non-git.

## Per product: what is read, the source/analysis interface, identity, and checkout avoidance

### Takeaway

The scalable tools (zoekt, livegrep, Blackbird, Sourcegraph gitserver, Gitaly, Software Heritage) read git objects directly and identify content by git blob SHA; the older or simpler tools (OpenGrok, Hound, Semgrep baseline, CodeQL) need a filesystem checkout. Each scalable tool has a narrow internal interface of the form "for a tree, yield (path, blob id, bytes)".

### Cited Findings

#### Sourcegraph (gitserver)

- The public monorepo snapshot `sourcegraph/sourcegraph-public-snapshot` is archived; its last commit is 2024-08-22 (c864f15). Everything below comes from that snapshot, so later private changes are not visible. — [repo metadata via GitHub API](https://github.com/sourcegraph/sourcegraph-public-snapshot)
- `VCSSyncer` interface: `Type()`, `IsCloneable(repo)`, `Clone(ctx, repo, targetDir, tmpPath, progressWriter)`, `Fetch(...)`. Its doc comment: "regardless of the VCSSyncer implementation, the result that ends up in tmpPath is expected to be a valid Git repository and should be initially optimized (repacked, commit-graph written, etc)". — [syncer.go L34-L64](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/syncer.go#L34-L64)
- `NewVCSSyncer` switches on the external-service type: Perforce, JVM packages, npm packages, Go modules, and so on. Files exist for go_modules, jvm, npm, python, ruby and rust packages. The vcssyncer package is 4,746 lines of non-test Go in total, of which perforce.go is 375 lines and packages_syncer.go is 397. — [syncer.go L115-L140](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/syncer.go#L115-L140), [vcssyncer dir](https://github.com/sourcegraph/sourcegraph-public-snapshot/tree/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer)
- Package repos are synthesised as git. `packagesSource.Download(ctx, dir, dep)` must "Download the given dependency's archive and unpack it into dir". Then `git init`, `git add .`, `git commit -m <pkg@version> --date <stable date>`, `git tag <version>`, and `git push --force origin --tags` into the bare repo, giving one tag per version. Versions no longer tracked have their tags deleted. — [packages_syncer.go L48-L58, L136-L295](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/packages_syncer.go#L48-L295)
- Author, committer and date are fixed (`stableGitCommitDate = "Thu Apr 8 14:24:52 2021 +0200"`), so the synthetic commit hash depends only on the package content. — [jvm_packages.go L35](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/jvm_packages.go#L35), [packages_syncer.go L339-L344](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/packages_syncer.go#L339-L344)
- Perforce depots are converted with `p4-fusion` (or git-p4), followed by a repack. — [perforce.go L124-L186](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/perforce.go#L124-L186)
- p4-fusion is a "fast Perforce depot to Git repository converter using the Helix Core C/C++ API" plus libgit2, written to replace git-p4.py. Its README claims it is ">100 times faster": 26 s against about 42 min for 3,393 changelists. The fork is active (last pushed 2026-11-12); the README badges point to salesforce/p4-fusion. — [p4-fusion README](https://github.com/sourcegraph/p4-fusion/blob/master/README.md)
- Analysis talks to gitserver over gRPC by `(repo, commit/treeish, path)`. RPCs include `ReadFile` ("gets a file from the repo ODB and streams the contents back"), `ReadDir`, `Stat`, `GetCommit`, `ResolveRevision`, `ListRefs`, `ChangedFiles`, `Blame`, and `Archive(repo, treeish, format ZIP|TAR, paths[])`. — [gitserver.proto L115-L344, L1032-L1040](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/internal/gitserver/v1/gitserver.proto#L115)
- The unindexed searcher calls `FetchTar(repo, commit, paths)` and caches a local zip per `(repo, commit)`. The cache key hashes `repo` and the 40-char resolved commit, and an unresolved commit is rejected. — [searcher store.go L55-L164](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/searcher/internal/search/store.go#L55-L164)
- Non-git hosts: "Sourcegraph natively supports all Git-based Version Control Systems". For others it offers `src-expose`, which can snapshot a `p4 sync` (or an hg/svn checkout) as a single new git commit, losing history. `src serve-git` is recommended for serving existing git repos. — [Sourcegraph non-git docs](https://sourcegraph.com/docs/admin/code_hosts/non-git)
- Current docs list Perforce and Plastic SCM as Tier 2 hosts. For CVS and SVN, "we do not offer native integrations". — [Sourcegraph code hosts](https://sourcegraph.com/docs/admin/code_hosts)
- Package repos (Rust, JVM, npm, Ruby, Go, Python) are marked "experimental and might change or be removed" in the 5.7 docs. — [package-repos 5.7](https://5.7.sourcegraph.com/admin/code_hosts/package-repos)
- Sourcegraph's zoekt indexserver gets the indexed commit from gitserver with `git fetch --depth=1 --no-tags` into a local cache before running gitindex. — [zoekt git_cache.go L164](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/cmd/zoekt-sourcegraph-indexserver/git_cache.go#L164)

**zoekt** (sourcegraph/zoekt @ 33ce1d0, 2026-11-16)

- `gitindex.Options`: `RepoDir`, `Branches []string` (e.g. `HEAD`, `stable`), `BranchPrefix`, `Submodules bool` ("follow submodule links. This requires RepoCacheDir"), `Incremental`, and `DeltaShardNumberFallbackThreshold`. — [index.go L442-L480](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/index.go#L442-L480)
- Tree walk: `RepoWalker.CollectFiles(tree, branch)` walks a commit's tree with go-git's `TreeWalker`. It keeps only `Regular`, `Executable` and `Symlink` entries and skips paths matched by the ignore file. It collects `fileKey{SubRepoPath, Path, ID(blob hash)} -> BlobLocation{GitRepo, URL, Branches}`. The same `(path, blob)` seen on several branches becomes one entry with several branches. — [tree.go L104-L262](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/tree.go#L104-L262)
- Blob reading: main-repo blobs are streamed through a pipelined `git cat-file --batch --buffer` with `--filter=blob:limit=SizeMax+1`, so large blobs are skipped without being read. Submodule blobs go through go-git. `ZOEKT_DISABLE_CATFILE_BATCH=true` is the kill switch. The cat-file path was once disabled by default "after git grew past 9GB of memory in production" because of promisor lazy fetching in partial clones; `GIT_NO_LAZY_FETCH=1` avoids it. — [index.go L701-L779](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/index.go#L701-L779), [doc/indexing.md L47-L69](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/doc/indexing.md#L47-L69)
- The interface to analysis is `index.Document{Name, Content []byte, Branches, SubRepositoryPath, Language, Category, SkipReason, Symbols}`, passed to `builder.Add(doc)`. Skipped files are kept as name-only placeholders (`SkipReasonTooLarge`, `Binary`, and so on). — [index/document.go L6-L33](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/index/document.go#L6-L33), [index.go L1247-L1290](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/index.go#L1247-L1290)
- The ignore file is `.sourcegraph/ignore`, read from the indexed tree itself (`tree.File(...)`) rather than from disk. — [ignore.go L26](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/ignore/ignore.go#L26), [index.go L914-L927](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/index.go#L914-L927)
- Submodules: `.gitmodules` is parsed from the tree, not the worktree. With a `RepoCache`, the walker opens the submodule's repo by URL, takes the gitlink commit's tree, and recurses with `SubRepoPath` prefixed. Submodule errors are logged and ignored. Without a cache, only the gitlink commit ID is recorded per path (`subRepoVersions`), or go-git's worktree submodules are used. — [tree.go L148-L230](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/tree.go#L148-L230), [index.go L1155-L1240](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/index.go#L1155-L1240)
- Delta builds reject submodules ("delta builds currently don't support submodule indexing"). — [index.go L951-L953](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/index.go#L951-L953)
- Archives: `zoekt-archive-index` reads tar, tgz or zip from a URL or path (`Archive.Next() -> File`). Zip needs random access ("streaming zip files not supported"). Options are `Branch`, `Commit` and `Strip` (path components). GitHub tarball URLs are recognised and get `Strip = 1`. The archive becomes one repository with one branch whose `Version` is the given commit. — [internal/archive/archive.go L30-L192](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/internal/archive/archive.go#L30-L192), [internal/archive/index.go L23-L160](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/internal/archive/index.go#L23-L160)
- Other front-ends: `zoekt-index` (plain directory via `filepath.Walk`), `zoekt-repo-index` (Android `repo` manifests over pre-fetched git repos), and mirror commands for GitHub, GitLab, Gerrit, Gitiles, Gitea and Bitbucket. — [cmd/zoekt-index/main.go L216](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/cmd/zoekt-index/main.go#L216), [cmd/zoekt-repo-index/main.go L14-L27](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/cmd/zoekt-repo-index/main.go#L14-L27), [cmd/](https://github.com/sourcegraph/zoekt/tree/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/cmd)

**OpenGrok** (oracle/opengrok @ dd38592, 2026-11-11)

- `RepositoryFactory` registers 15 drivers: Git, Mercurial, Repo (Android), BitKeeper, Bazaar, Monotone, Subversion, SCCS, Razor, RCS, CVS, SSCM, AccuRev, ClearCase and Perforce. — [RepositoryFactory.java L60-L78](https://github.com/oracle/opengrok/blob/dd385925e974873e5cdf44fe3599ae40e1866166/opengrok-indexer/src/main/java/org/opengrok/indexer/history/RepositoryFactory.java#L60-L78)
- The abstract `Repository` methods work on `java.io.File`s in the source root: `fileHasHistory(File)`, `getHistory(File)`, `getHistoryGet(out, parent, basename, rev)` (content at an old revision), `annotate(File, rev)`, `isRepositoryFor(File)`, `determineBranch/Parent/CurrentVersion`, and ignored files and dirs. The abstraction covers history, blame and old revisions; enumerating current files is a working-tree walk. — [Repository.java L91-L494](https://github.com/oracle/opengrok/blob/dd385925e974873e5cdf44fe3599ae40e1866166/opengrok-indexer/src/main/java/org/opengrok/indexer/history/Repository.java#L91-L494)
- Git uses JGit (`org.eclipse.jgit.api.Git`, `BlameCommand`). The others shell out to the VCS CLI, e.g. Subversion `CMD_FALLBACK = "svn"`. — [GitRepository.java L55-L57](https://github.com/oracle/opengrok/blob/dd385925e974873e5cdf44fe3599ae40e1866166/opengrok-indexer/src/main/java/org/opengrok/indexer/history/GitRepository.java#L55-L57), [SubversionRepository.java L82](https://github.com/oracle/opengrok/blob/dd385925e974873e5cdf44fe3599ae40e1866166/opengrok-indexer/src/main/java/org/opengrok/indexer/history/SubversionRepository.java#L82)
- Size, measured with `wc -l` over the main `history` package: 18,113 lines of Java. Per-driver files (repository plus history, annotation and tag parsers): Git 940, Mercurial 1,241, Subversion 933, BitKeeper 813, Perforce 787, Bazaar 768, AccuRev 739, SCCS 737, CVS 616, Razor 575, RCS 551, SSCM 544, Monotone 531, ClearCase 494. That is about 9,300 lines for the 13 non-git drivers, roughly 10 times Git's. History tests are 8,932 lines. — [history package](https://github.com/oracle/opengrok/tree/dd385925e974873e5cdf44fe3599ae40e1866166/opengrok-indexer/src/main/java/org/opengrok/indexer/history)
- Issue counts by title keyword (GitHub search, `is:issue`, 2026-11-16): git 75, mercurial 28, perforce 21, cvs 15, clearcase 12, subversion 11. This is a crude proxy; titles mentioning "git" include generic issues. — [GitHub issue search, oracle/opengrok](https://github.com/oracle/opengrok/issues?q=is%3Aissue+mercurial+in%3Atitle)

#### Software Heritage

- Artifacts: contents ("blobs"; raw bytes without names), directories, revisions ("commits"), releases ("tags"), snapshots (all branches of an origin at a visit), origins `(type, url)`, and visits. Deduplication "is built-in", across "file content, commits, entire directories or project snapshots". VCS-to-VCS migrations (e.g. SVN to Git) are named explicitly as a source of duplication. — [data-model.rst](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/docs/data-model.rst)
- Identifiers are git-compatible: revision = "same result as a git commit hash", directory = "git tree hash", content `sha1_git` = SHA1 of `"blob <len>\0<bytes>"`. — [persistent-identifiers.rst L144-L168](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/docs/persistent-identifiers.rst#L144-L168)
- Content also stores `sha1`, `sha256`, `blake2s256` and `length`. Objects can keep a `raw_manifest` so that verbatim git objects keep their original hashes when they don't round-trip. `RevisionType` ∈ {git, tar, dsc, svn, hg, cvs, bzr}. — [model.py L499-L530, L1238-L1245, L1629-L1636](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/swh/model/model.py#L1238-L1245)
- `ExtID` links a native identifier to a core SWHID, e.g. a directory to "the cryptographic hash of the tarball that originally contained it", with an optional payload for rebuilding the tarball. — [data-model.rst (extid)](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/docs/data-model.rst)
- The Mercurial loader stores `hg-nodeid` extids. — [mercurial loader.py L67](https://github.com/SoftwareHeritage/swh-loader-mercurial/blob/53a251bb06a8f3a761fd5057609e89d0a0ea398b/swh/loader/mercurial/loader.py#L67)
- Package loaders exist for arch, archive, aur, bioconductor, conda, cpan, cran, crates, debian, deposit, golang, hackage, hex, maven, nixguix, npm, opam, pubdev, puppet, pypi, rpm and rubygems. — [swh/loader/package/](https://github.com/SoftwareHeritage/swh-loader-core/tree/639eb16bbd543ac47747907013325149c7951845/swh/loader/package)
- Each package version becomes a synthetic release (e.g. "Synthetic release for archive at {url}"), with `synthetic = true` and author and date taken from intrinsic or extrinsic metadata, pointing at the unpacked directory. — [package-loader-specifications.rst](https://github.com/SoftwareHeritage/swh-loader-core/blob/639eb16bbd543ac47747907013325149c7951845/docs/package-loader-specifications.rst)
- The package loader flow is to download, `uncompress` to disk, and hash with `from_disk`. Already-seen artifacts are skipped through extid `package-manifest-sha256`. — [package/loader.py L36-L41, L116-L154, L266-L291](https://github.com/SoftwareHeritage/swh-loader-core/blob/639eb16bbd543ac47747907013325149c7951845/swh/loader/package/loader.py#L116)
- Package loaders "are somewhat uniform (list tarballs, download tarballs, load content of tarball, done)". VCS loaders "are hooked deeply into their respective VCS' internals", and `DVCSLoader` subclasses implement `fetch_data` plus `get_contents`, `get_directories`, `get_revisions`, and so on. — [vcs-loader-overview.rst](https://github.com/SoftwareHeritage/swh-loader-core/blob/639eb16bbd543ac47747907013325149c7951845/docs/vcs-loader-overview.rst)
- The SVN loader replays revisions and exports files to disk when needed (externals). A separate `svn-export` visit type exports given paths at a revision. — [swh-loader-svn replay.py L115-L122](https://github.com/SoftwareHeritage/swh-loader-svn/blob/655c567c678a3a310f581278b9a1a1c76d82189e/swh/loader/svn/replay.py#L115-L122), [directory.py L23-L90](https://github.com/SoftwareHeritage/swh-loader-svn/blob/655c567c678a3a310f581278b9a1a1c76d82189e/swh/loader/svn/directory.py#L23-L90)
- docs.softwareheritage.org was behind an Anubis bot wall during this research, so everything here was read from the GitHub mirrors of the docs sources.

#### GitHub code search (Blackbird)

- The index is "sharded by Git blob object ID", which gives "a nice way of evenly distributing documents between the shards while avoiding any duplication". — [GitHub blog: technology behind code search](https://github.blog/engineering/architecture-optimization/the-technology-behind-githubs-new-code-search/)
- Delta indexing: repositories form a tree (an approximate minimum spanning tree over a similarity graph built with "geometric filters"). Each repo is crawled only for blobs not in its parent, cutting crawled documents by "over 50%". The scale was 115 TB of code and 15.5 billion documents across about 45M repos, deduplicated to about 28 TB unique, with a 25 TB index. — [GitHub blog](https://github.blog/engineering/architecture-optimization/the-technology-behind-githubs-new-code-search/)
- Ingest: Kafka events (e.g. a push) trigger crawlers that fetch blobs from git, extract symbols and publish documents to a per-shard Kafka partition. Per-document metadata holds locations (path, branch, repository). Queries are consistent at the commit level. The post has no fork-specific section; forks are covered by the delta tree. — [GitHub blog](https://github.blog/engineering/architecture-optimization/the-technology-behind-githubs-new-code-search/)

#### CodeQL

- A database is "a directory containing queryable data, extracted from the code" plus "a source reference, for displaying query results". The reference is either a source archive directory, "a source archive, typically in ZIP format", or "a source archive repository, typically in `git` format, typically bare". — [CodeQL glossary](https://codeql.github.com/docs/codeql-overview/codeql-glossary)
- Capture works from a checkout: `--source-root` defaults to `.` ("In many cases, this will be the checkout root"). Compiled languages capture what the build compiles ("code that is not built will not be included"). `database create` has no commit or git options. — [database create manual](https://docs.github.com/en/code-security/reference/code-scanning/codeql/codeql-cli-manual/database-create), [preparing code](https://docs.github.com/en/code-security/codeql-cli/getting-started-with-the-codeql-cli/preparing-your-code-for-codeql-analysis)
- The file name `src.zip` was not confirmed by any page read; "ZIP format" is confirmed. — snippet-only via search; `src.zip` itself is unverified.

#### Kythe

- VName = (signature, corpus, root, path, language). A corpus is e.g. "the contents of a given source repository". An empty root "corresponds to a file under version control"; a non-empty root marks generated files. — [Kythe storage model](https://kythe.io/docs/kythe-storage.html)
- `FileInfo{path, digest}`: digest is the "lowercase ascii hex SHA-256 digest of the file contents". `FileData{content, info, missing}` comes from a `FileDataService`, and a `CompilationUnit.required_input` with equal digests "are the same file". Compilation records carry a `revision` marker. — [analysis.proto L40-L47, L122-L124, L260-L290](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/analysis.proto#L260-L290)
- The kzip layout stores files under their digest. — unverified (kzip doc not read).

#### Glean

- "The name and hash together uniquely identify a database. This is written `<name>/<hash>`." The name is often the repository name, and the hash "can be the revision of the repository that was indexed". — [Glean databases](https://glean.software/docs/databases/)
- `predicate File : string` holds a path string, with no content hash. — [src.angle L6](https://github.com/facebookincubator/Glean/blob/4aeef0241ac56808439f308047d137fc5cb08552/glean/schema/source/src.angle#L6)

**livegrep** (livegrep/livegrep @ 923d5ad, 2026-02-10)

- `git_indexer::walk_tree` uses libgit2 and calls `index_file(tree, submodule_prefix + path, git_blob_rawcontent(...))`. Gitlink entries (`GIT_OBJ_COMMIT`) are skipped unless `walk_submodules` is set, in which case `git_submodule_lookup` is used and a child indexer recurses with a path prefix. — [git_indexer.cc L60-L127](https://github.com/livegrep/livegrep/blob/923d5ad71dfe60900e6c2017b2fa4a5ff902ad71/src/git_indexer.cc#L60-L127)

**Hound** (hound-search/hound @ cf795f1, 2026-09-10)

- `vcs.Driver`: `Clone(dir, url)`, `Pull(dir)`, `HeadRev(dir)`, `SpecialFiles()`, `AutoGeneratedFiles(dir)`. Drivers exist for git, hg, svn, bzr and local. The git driver uses `git ls-files -z` and `git check-attr` for generated files. The indexer walks the working directory with `filepath.Walk`. — [vcs.go L16-L31](https://github.com/hound-search/hound/blob/cf795f1889a7711cd46bfbef5f865544beeec391/vcs/vcs.go#L16-L31), [git.go L146-L159](https://github.com/hound-search/hound/blob/cf795f1889a7711cd46bfbef5f865544beeec391/vcs/git.go#L146-L159), [index.go L387](https://github.com/hound-search/hound/blob/cf795f1889a7711cd46bfbef5f865544beeec391/index/index.go#L387)

#### Sourcebot

- Bundles zoekt ("Updated the bundled Zoekt version"). A generic git connection takes a remote URL or `file:///path` (must be a git repo root; globs allowed; "treated as read-only"). The public GitHub repo at v6.1.0 contains only the README, CHANGELOG, schemas and tools, not the app source. — [CHANGELOG.md](https://github.com/sourcebot-dev/sourcebot/blob/2914eb23aa6257874f459f93a683eb04366c6cc5/CHANGELOG.md), [genericGitHost.json L7-L18](https://github.com/sourcebot-dev/sourcebot/blob/2914eb23aa6257874f459f93a683eb04366c6cc5/schemas/v3/genericGitHost.json#L7-L18)

**GitLab Gitaly** (master eb2b1219, 2026-11-16)

- `TreeEntry(revision, path, limit)`, `GetTreeEntries`, `ListFiles`, `GetBlob`, `GetBlobs(revision_paths[], limit)`, `ListBlobs(revisions)` (a graph walk of all blobs reachable from revisions), `ListAllBlobs`, LFS pointer RPCs and `RawBlame`. GetBlobs "does not only return blobs, but will also return submodules as commits and trees. It's use is thus discouraged in favor of ListBlobs". — [blob.proto L17-L125](https://gitlab.com/gitlab-org/gitaly/-/blob/master/proto/blob.proto), [commit.proto L43-L131, L360-L368](https://gitlab.com/gitlab-org/gitaly/-/blob/master/proto/commit.proto)

**Semgrep** (semgrep/semgrep @ 06d9993, 2026-11-10)

- Targeting uses `git ls-files -z` (plus `--others` options) over the working tree. The wrapper shells out to `git` rather than libgit2 or ocaml-git because those lack porcelain such as `ls-files`. — [Git_wrapper.ml L22-L33, L358-L389](https://github.com/semgrep/semgrep/blob/06d9993409f6698f78f6956cd39d968e7f4d7b6a/libs/git_wrapper/Git_wrapper.ml#L22-L389)
- `--baseline-commit`: `run_with_worktree ~commit f` runs `git worktree add <tmp>/semgrep_git_worktree_<uuid> <commit>`, scans there, then `git worktree remove`. This is a full checkout of the baseline. — [Git_wrapper.ml L487-L530](https://github.com/semgrep/semgrep/blob/06d9993409f6698f78f6956cd39d968e7f4d7b6a/libs/git_wrapper/Git_wrapper.ml#L487-L530). It is used from `src/osemgrep/cli_scan/Diff_scan.ml` (snippet-only, from a code-search hit).

#### SonarQube

- "Git and SVN are supported by default. For other SCM providers, see the Marketplace." "TFVC is not supported. Please convert your project to a Git repository." Git blame uses JGit ("pure Java"); "A full Git clone is required"; shallow clones and submodules cause missing blame. — [Sonar SCM integration](https://docs.sonarsource.com/sonarqube-server/analyzing-source-code/scm-integration)
- A native-git blame path also exists (`GIT_NATIVE_BLAME` vs `GIT_FILES_BLAME`, chosen by a heuristic). — snippet-only, [Sonar community thread](https://community.sonarsource.com/t/sonarscanner-blame-falling-back-to-git-native-blame-how-to-debug/116685)

**PyDriller** (ishepard/pydriller @ 9729e01, 2026-09-26)

- A remote URL is cloned into a temp folder. `ModifiedFile.content` and `content_before` read `diff.b_blob` / `a_blob` `.data_stream.read()` through GitPython, with no checkout. `Git.checkout(hash)` (`git checkout -f`) exists as an explicit API that the analysis itself does not call. — [commit.py L195-L236](https://github.com/ishepard/pydriller/blob/9729e010b0a07378e20382dabeda5d5dfb5d40de/pydriller/domain/commit.py#L195-L236), [git.py L151-L182](https://github.com/ishepard/pydriller/blob/9729e010b0a07378e20382dabeda5d5dfb5d40de/pydriller/git.py#L151-L182), [PyDriller docs](https://pydriller.readthedocs.io/en/latest/repository.html)

#### CodeScene

- Git only: it clones over ssh, http or git, or analyses a local repo root (the folder with `.git`). No Mercurial or SVN mention was found. — snippet-only, [CodeScene project config](https://codescene.io/docs/configuration/project-config/index.html)

### Inferences

- Three families exist:
  - **Object-DB readers** (zoekt, livegrep, Blackbird, Gitaly, Sourcegraph gitserver, PyDriller, SWH git loader) never materialise a worktree.
  - **Checkout readers** (OpenGrok, Hound, CodeQL, Semgrep, Sonar, CodeScene) rely on the filesystem and treat VCS as metadata (history, blame).
  - **Normalisers** (Sourcegraph syncers, Software Heritage loaders) convert each foreign source once into a git-shaped object store.
- For mentalc, which compiles one repo at one commit, the object-DB-reader shape is the right fit: resolve commit, get tree, walk entries, read blobs. Archives and packages then become synthetic trees, as in Sourcegraph and SWH.

### Gaps

- Sourcegraph's post-2024 private code is not visible; whether hg and svn gained native syncers after August 2024 is unknown.
- There was no time to verify Kythe's kzip layout, CodeQL's exact `src.zip` name, or gitoxide-based analysers (e.g. cargo's registry index reading, or `gix`-based tools).
- Gerrit's code search was not investigated beyond zoekt's `zoekt-mirror-gerrit`.

## Which tools support non-git VCS natively vs by conversion to git? What did native multi-VCS support cost?

### Takeaway

Only OpenGrok (15 drivers) and Hound (5 drivers) support many VCSs natively. Both do it only to the depth of "a checkout plus history", and OpenGrok pays about 9.3k lines for 13 non-git drivers. Sourcegraph and Software Heritage convert at the edge (into git, or into a git-compatible DAG); zoekt, livegrep, Blackbird, Gitaly, CodeScene and PyDriller are git-only. Sonar has git and SVN, and tells TFVC users to convert to git.

### Cited Findings

- OpenGrok registers 15 `Repository` drivers. The non-git ones total about 9,300 lines against Git's 940, and the history package is 18,113 lines plus 8,932 lines of tests. — [history package](https://github.com/oracle/opengrok/tree/dd385925e974873e5cdf44fe3599ae40e1866166/opengrok-indexer/src/main/java/org/opengrok/indexer/history)
- Every OpenGrok non-git driver shells out to its CLI and parses text output: each VCS has its own `*HistoryParser` and `*AnnotationParser` class. — [history package listing](https://github.com/oracle/opengrok/tree/dd385925e974873e5cdf44fe3599ae40e1866166/opengrok-indexer/src/main/java/org/opengrok/indexer/history)
- Sourcegraph's contract is that the result of any syncer "is expected to be a valid Git repository". — [syncer.go L43-L47](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/syncer.go#L43-L47)
- Sourcegraph's Perforce support needed its own high-performance converter (p4-fusion), because git-p4.py "has ... performance issues with any depot greater than 1 GB". — [p4-fusion README](https://github.com/sourcegraph/p4-fusion/blob/master/README.md)
- Sourcegraph CVS and SVN: "we do not offer native integrations". `src-expose` snapshots non-git checkouts into single git commits, losing history. — [code hosts](https://sourcegraph.com/docs/admin/code_hosts), [non-git](https://sourcegraph.com/docs/admin/code_hosts/non-git)
- Software Heritage keeps one model but per-VCS loaders, which "are hooked deeply into their respective VCS' internals". Native IDs survive as `ExtID` (e.g. `hg-nodeid`), and `RevisionType` records the origin VCS. — [vcs-loader-overview.rst](https://github.com/SoftwareHeritage/swh-loader-core/blob/639eb16bbd543ac47747907013325149c7951845/docs/vcs-loader-overview.rst), [model.py L1238](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/swh/model/model.py#L1238-L1245)
- Hound drivers are git, hg, svn, bzr and local, each implementing only Clone, Pull, HeadRev and special/auto-generated files. — [vcs dir](https://github.com/hound-search/hound/tree/cf795f1889a7711cd46bfbef5f865544beeec391/vcs)
- Sonar: "TFVC is not supported. Please convert your project to a Git repository." — [Sonar SCM](https://docs.sonarsource.com/sonarqube-server/analyzing-source-code/scm-integration)

### Inferences

- Native multi-VCS stays cheap (Hound) only when the VCS layer does nothing but "produce a checkout". It turns expensive (OpenGrok) once history, blame and old revisions are needed per VCS. The industry trend is convert-to-git (Sourcegraph, Sonar's advice) or convert-to-a-git-compatible-DAG (SWH).
- For mentalc, "non-git VCS" can be deferred to "convert to git outside mentalc" (git-svn, git-remote-hg, p4-fusion) or "export a directory or zip". No in-tree drivers are needed.

### Gaps

- No bug-rate or maintenance-hours data found for OpenGrok's drivers; issue-title counts are only a proxy.

## What is the minimal interface between source access and analysis?

### Takeaway

Across the efficient tools, the seam is "for a resolved tree, iterate entries (path, mode, blob id) and fetch bytes by blob id", plus a document struct the builder consumes. Sourcegraph and Gitaly expose the same thing as RPCs keyed by `(repo, commit, path)`, plus a whole-tree tar or zip stream.

### Cited Findings

- zoekt: source-specific code builds `index.Document{Name, Content, Branches, SubRepositoryPath, SkipReason}` and calls `builder.Add`. Git, archive and directory front-ends all target the same builder. — [document.go](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/index/document.go#L6-L23), [archive/index.go L145-L160](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/internal/archive/index.go#L145-L160)
- The zoekt archive iterator is `Archive.Next() (*File, error)`, where `File` carries the name, size, mod time and a reader. — [archive.go L30-L110](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/internal/archive/archive.go#L30-L110)
- livegrep: `index_file(tree, path, bytes)`. — [git_indexer.cc L104-L105](https://github.com/livegrep/livegrep/blob/923d5ad71dfe60900e6c2017b2fa4a5ff902ad71/src/git_indexer.cc#L104-L105)
- Sourcegraph gitserver: `ReadFile`, `ReadDir`, `Stat`, `Archive(treeish, format, paths)`. The searcher consumes `FetchTar(repo, commit, paths)`. — [gitserver.proto](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/internal/gitserver/v1/gitserver.proto#L115), [store.go L55-L59](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/searcher/internal/search/store.go#L55-L59)
- Gitaly: `TreeEntry(revision, path)`, `GetBlobs([(revision, path)])`, `ListBlobs(revisions)`. — [blob.proto](https://gitlab.com/gitlab-org/gitaly/-/blob/master/proto/blob.proto)
- Kythe: `FileDataService.Get(FileInfo{path, digest}) -> FileData{content, missing}`. — [analysis.proto L260-L290](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/analysis.proto#L260-L290)
- Sourcegraph's interface for adding a new source kind: `packagesSource.Download(ctx, dir, dep)` (unpack into a dir), plus name and version parsing. The git synthesis is shared. — [packages_syncer.go L48-L58](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/packages_syncer.go#L48-L58)
- SWH `DVCSLoader`: `fetch_data` plus `get_contents`, `get_directories`, `get_revisions`, ... — [vcs-loader-overview.rst](https://github.com/SoftwareHeritage/swh-loader-core/blob/639eb16bbd543ac47747907013325149c7951845/docs/vcs-loader-overview.rst)

### Inferences

- A mentalc-sized interface would have two parts. The source side exposes `resolve(ref) -> tree id`, `entries(tree) -> (path, kind{file, exec, symlink, gitlink}, blob id)` and `read(blob id) -> bytes`. The analysis side consumes `(path, blob id, bytes)`. A zip or a directory implements the same trait by computing git blob IDs on the fly, as SWH does for tarballs.
- Keeping skipped files as name-only placeholders (zoekt `SkipReason`) preserves completeness without reading large or binary blobs.

### Gaps

- None material.

## How are submodules handled when indexing?

### Takeaway

They are off by default or best-effort everywhere. When followed, they are resolved through a separate repo cache keyed by the `.gitmodules` URL, at the gitlink commit, with files path-prefixed into the parent tree and the submodule's version recorded per path. Incremental or delta modes often don't support them.

### Cited Findings

- zoekt: opt-in `Submodules`, which needs `RepoCacheDir`. `.gitmodules` is parsed from the tree. The sub-repo is opened from the cache by URL, and its tree is taken from the gitlink commit and recursed with `SubRepoPath`. Errors are "ignoring submodule error". Without a cache, only the gitlink SHA is recorded per path. Delta builds reject submodules. — [tree.go L148-L210](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/tree.go#L148-L210), [index.go L951-L953](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/index.go#L951-L953)
- zoekt submodule blobs use the slower go-git path rather than cat-file. — [index.go L701-L760](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/index.go#L701-L760)
- livegrep: skips gitlinks unless `walk_submodules`, then uses a child indexer with a prefix. — [git_indexer.cc L110-L127](https://github.com/livegrep/livegrep/blob/923d5ad71dfe60900e6c2017b2fa4a5ff902ad71/src/git_indexer.cc#L110-L127)
- Gitaly GetBlobs returns submodules "as commits and trees", and its own docs discourage it for that reason. — [blob.proto L17-L27](https://gitlab.com/gitlab-org/gitaly/-/blob/master/proto/blob.proto)
- Sonar: submodules are a known cause of "Missing blame information". — [Sonar SCM](https://docs.sonarsource.com/sonarqube-server/analyzing-source-code/scm-integration)
- zoekt `zoekt-repo-index` handles Android `repo` multi-repo manifests over pre-fetched git repos, a manifest-based alternative to submodules. — [main.go L14-L27](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/cmd/zoekt-repo-index/main.go#L14-L27)
- Sourcegraph and Blackbird: no submodule-following found in the material read. Submodules appear to be indexed as separate repos when hosted. — unverified.

### Inferences

- mentalc should record each gitlink as `(path, submodule url, commit)` by default. Recursing should be opt-in and go through the same source layer, with the sub-repo identified by `(url, commit)`. This mirrors zoekt's `subRepoVersions` and keeps the one-repo-at-one-commit model honest.

### Gaps

- Sourcegraph's handling of submodules in search results was not checked in its code.

## How do they identify content for caching and dedup?

### Takeaway

Git blob SHA-1 (and tree and commit IDs) is the de facto content key: Blackbird shards on it, zoekt keys files on `(path, blob)`, and SWH makes all its IDs git-compatible so non-git sources share the space. Kythe uses SHA-256 file digests. Glean and Sourcegraph's searcher key on `(repo, revision)`.

### Cited Findings

- Blackbird: sharded by blob object ID, "avoiding any duplication". The delta tree cut crawling by "over 50%", and unique content is about 28 TB of 115 TB. — [GitHub blog](https://github.blog/engineering/architecture-optimization/the-technology-behind-githubs-new-code-search/)
- zoekt: `fileKey{SubRepoPath, Path, ID}`; identical keys across branches merge their `Branches`. — [tree.go L218-L262](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/gitindex/tree.go#L218-L262)
- SWH: content `sha1_git` equals the git blob hash; directory, revision and release IDs equal git tree, commit and tag hashes. Extra hashes are `sha1`, `sha256` and `blake2s256`. Tarball provenance is kept as an ExtID mapping. — [persistent-identifiers.rst L144-L168](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/docs/persistent-identifiers.rst#L144-L168), [model.py L1629-L1634](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/swh/model/model.py#L1629-L1634)
- Kythe: SHA-256 digest per file; equal digests mean the same file. — [analysis.proto L122-L124, L260-L269](https://github.com/kythe/kythe/blob/69141f022689a611e8a4a1d9b08a3783a2e8a9ed/kythe/proto/analysis.proto#L260-L269)
- Sourcegraph searcher: the cache key is a hash of `(repo, 40-char commit)`. — [store.go L147-L164](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/searcher/internal/search/store.go#L147-L164)
- Sourcegraph package repos use a fixed author, committer and date, so the commit ID is a pure function of the unpacked contents and the version message. — [packages_syncer.go L339-L344](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/packages_syncer.go#L339-L344)
- Glean: DB `<name>/<hash>`, where the hash is often the repo revision; `src.File` is path only. — [Glean databases](https://glean.software/docs/databases/), [src.angle](https://github.com/facebookincubator/Glean/blob/4aeef0241ac56808439f308047d137fc5cb08552/glean/schema/source/src.angle#L6)

### Inferences

- Using git blob IDs (computed with `"blob <len>\0"` hashing for non-git inputs) as mentalc's per-file cache key makes a zip, a directory and a commit with the same file share cache entries for free. Tree IDs give whole-subtree cache hits, which SWH gets for "entire directories".
- Git's SHA-256 object-format transition would need handling. SWH stores extra hashes for that reason (inference).

### Gaps

- Blackbird's exact document key (blob SHA alone, or blob plus path) beyond "sharded by blob object ID" is not stated in the post.

## Archives and package sources: how are tarballs, zips and packages treated?

### Takeaway

Two patterns. (1) Synthetic commit or release: Sourcegraph unpacks a package and commits it into a per-package git repo with one tag per version and fixed metadata; SWH makes a synthetic release pointing at the unpacked directory, with the tarball hash kept as an ExtID. (2) Direct archive iteration as one "branch": zoekt-archive-index, with a caller-supplied branch, commit label and path-strip count. Both treat a package or archive as its own corpus or repo, separate from the code that depends on it.

### Cited Findings

- Sourcegraph: Download and unpack, then `git init/add/commit --date <stable>/tag/push`. One repo per package and one tag per version; obsolete version tags are deleted. — [packages_syncer.go L136-L295](https://github.com/sourcegraph/sourcegraph-public-snapshot/blob/c864f15af264f0f456a6d8a83290b5c940715349/cmd/gitserver/internal/vcssyncer/packages_syncer.go#L136-L295)
- Sourcegraph package repos are added when "referenced in code graph data uploads" (recommended) or listed manually; the feature is experimental. — [package-repos 5.7](https://5.7.sourcegraph.com/admin/code_hosts/package-repos)
- SWH: each package version is a release with `synthetic = true`, message "Synthetic release for ...", and author and date from metadata. The tarball is uncompressed to disk and hashed with `from_disk`; extid `package-manifest-sha256` prevents re-loading. — [package-loader-specifications.rst](https://github.com/SoftwareHeritage/swh-loader-core/blob/639eb16bbd543ac47747907013325149c7951845/docs/package-loader-specifications.rst), [package/loader.py](https://github.com/SoftwareHeritage/swh-loader-core/blob/639eb16bbd543ac47747907013325149c7951845/swh/loader/package/loader.py#L116-L154)
- SWH ExtID can link a directory to the original tarball's hash, with a payload to rebuild it. — [data-model.rst](https://github.com/SoftwareHeritage/swh-model/blob/731ffbf755330a048a02612c19db5188da9188c5/docs/data-model.rst)
- zoekt-archive-index: tar, tgz or zip with `Strip` and `Branch`/`Commit` labels, giving one repo per archive. Zip cannot be streamed (it needs a seekable file). — [archive/index.go L23-L104](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/internal/archive/index.go#L23-L104), [archive.go L83-L97](https://github.com/sourcegraph/zoekt/blob/33ce1d04d373d9c8dcdb8b0a67119dedb6345ccc/internal/archive/archive.go#L83-L97)
- CodeQL inverts this: the database itself contains a zip, directory or bare git "source reference" of what was analysed. — [CodeQL glossary](https://codeql.github.com/docs/codeql-overview/codeql-glossary)
- Kythe: generated files and other trees get a distinct `root` within a corpus; separate repos are separate corpora. — [Kythe storage](https://kythe.io/docs/kythe-storage.html)

### Inferences

- For mentalc, a zip or package can be a source that yields a tree, either kept in memory or written as git objects, with an identity like `(origin url or package@version, tree id)`. Two choices are open. One is converting to git on disk, as Sourcegraph does; it is simpler, but it adds a git-write dependency and a cache directory. The other is in-memory synthetic trees hashed git-style, as SWH does; it needs no writes and keeps one identity scheme. The second fits "no bloat" better. Dependencies would then be separate corpora (Kythe) or separate origins (SWH) referenced from the main model, not merged into its tree.

### Gaps

- How Sourcegraph names package repos (e.g. `npm/<pkg>`) and links them to precise code-intel was not read in code.
