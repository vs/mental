# Verified: sliding versions, git-sourced dependencies and dependency identity (state as of 2026-11-14)

Method. Every page cited below was downloaded whole on 2026-11-14 and the quoted passages were located in the downloaded text by search and read in context; I did not read every line of the longest pages (go.dev/ref/mod, the GitHub workflow syntax page, the POM reference). Nothing is taken from a search-result snippet. Each claim is marked CONFIRMED (verbatim quote, under 40 words, and URL), REFUTED (what the source says instead) or NOT FOUND. Where the only primary source is tool source code rather than documentation, that is said.

Versions observed in the sources on that date: Maven 3.10.0 and 3.9.16 are the GA lines, Maven 4 is at 4.0.0-rc-7; Gradle current is 9.9.1; pnpm documentation refers to v12; Cargo lockfile default is V4.

## Maven: SNAPSHOT, ranges, LATEST/RELEASE, mediation, local repository, Maven 4, maven-lockfile

### Takeaway

Maven has no lockfile in either 3.x or 4.0.0-rc-7; the exact build a `-SNAPSHOT` resolved to is recorded only in repository metadata (remote `maven-metadata.xml`, and its per-repository copy in `~/.m2/repository`). Maven 4.0.0 is not GA. The third-party `maven-lockfile` plugin writes a JSON `lockfile.json` with checksums and a nested dependency tree.

### Cited Findings

- CONFIRMED, SNAPSHOT meaning: "The SNAPSHOT value refers to the 'latest' code along a development branch, and provides no guarantee the code is stable or unchanging." — [Maven Getting Started Guide](https://maven.apache.org/guides/getting-started/index.html)
- CONFIRMED, release is fixed: "the code in a 'release' version (any version value without the suffix SNAPSHOT) is unchanging." — [Maven Getting Started Guide](https://maven.apache.org/guides/getting-started/index.html)
- CONFIRMED, timestamped snapshot resolution lives in version-level metadata: "The version level metadata (found in directory of baseVersion) serves the purpose of snapshot timestamped version resolution." and "Exists only in Snapshot Remote repositories." — [Maven Metadata](https://maven.apache.org/repositories/metadata.html)
- CONFIRMED, metadata fields: `versioning/snapshot/timestamp` is "The timestamp when this version was deployed. The timestamp is expressed using UTC in the format yyyyMMdd.HHmmss."; `buildNumber` is "The incremental build number"; `snapshotVersions` is "only the most recent SNAPSHOT for each unique extension/classifier combination." — [Repository Metadata model, Maven 3.10.0](https://maven.apache.org/ref/current/maven-repository-metadata/repository-metadata.html)
- CONFIRMED, artifact-level metadata fields: `latest` is "What the last version added to the directory is, including both releases and snapshots"; `release` is "What the last version added to the directory is, for the releases only". — [Repository Metadata model](https://maven.apache.org/ref/current/maven-repository-metadata/repository-metadata.html)
- CONFIRMED, `updatePolicy`: "The choices are: always, daily (default), interval:X (where X is an integer in minutes) or never (only downloads if not yet existing in the local repository)." — [POM Reference](https://maven.apache.org/pom.html)
- CONFIRMED, new in 3.10.0: "new CLI options for fine-grained update policy control: -UA (force artifact updates only), -UM (force metadata updates only), --artifacts-update-policy=<policy>, --metadata-update-policy=<policy>" — [Maven 3.10.0 Release Notes](https://maven.apache.org/docs/3.10.0/release-notes.html)
- CONFIRMED, download trigger: "Downloading in Maven is triggered by a project declaring a dependency that is not present in the local repository (or for a SNAPSHOT, when the remote repository contains one that is newer)." — [Introduction to Repositories](https://maven.apache.org/guides/introduction/introduction-to-repositories.html)
- CONFIRMED, version ranges: "[1.0,2.0): 1.0 <= x < 2.0; Hard requirement for any version between 1.0 inclusive and 2.0 exclusive." and a bare `1.0` is a "Soft requirement for 1.0. Use 1.0 if no other version appears earlier in the dependency tree." — [POM Reference](https://maven.apache.org/pom.html)
- CONFIRMED, range resolution picks the highest: "Maven picks the highest version of each project that satisfies all the hard requirements of the dependencies on that project. If no version satisfies all the hard requirements, the build fails." — [POM Reference](https://maven.apache.org/pom.html)
- CONFIRMED, a range pitfall: "the version requirement [1.0,2.0) excludes 2.0 but includes version 2.0-rc1, which is contrary to what most people expect. In addition, Gradle interprets it differently" — [POM Reference](https://maven.apache.org/pom.html)
- CONFIRMED, nearest wins: "Maven picks the "nearest definition". That is, it uses the version of the closest dependency to your project in the tree of dependencies." and "if two dependency versions are at the same depth in the dependency tree, the first declaration wins." — [Introduction to the Dependency Mechanism](https://maven.apache.org/guides/introduction/introduction-to-dependency-mechanism.html)
- CONFIRMED, dependencyManagement outranks mediation: "dependency management takes precedence over dependency mediation for transitive dependencies" — [Introduction to the Dependency Mechanism](https://maven.apache.org/guides/introduction/introduction-to-dependency-mechanism.html)
- CONFIRMED with a qualification, `LATEST` and `RELEASE`: for plugins, "Maven 3.x no longer supports usage of these metaversions in the POM." — [Maven 3.x Compatibility Notes](https://cwiki.apache.org/confluence/display/MAVEN/Maven+3.x+Compatibility+Notes). That page speaks only of `<plugin>` declarations. The resolver code still handles both words for any artifact: `if (RELEASE.equals(version))` and `else if (LATEST.equals(version))` select `maven-metadata.xml`, and `version.endsWith(SNAPSHOT)` selects the version-level metadata — [DefaultVersionResolver.java, maven-3.9.x branch](https://raw.githubusercontent.com/apache/maven/maven-3.9.x/maven-resolver-provider/src/main/java/org/apache/maven/repository/internal/DefaultVersionResolver.java) (source code, not documentation).
- CONFIRMED, `_remote.repositories`: the setting `aether.lrm.enhanced.trackingFilename` is the "Filename of the file in which to track the remote repositories." with default `"_remote.repositories"`. — [Resolver configuration](https://maven.apache.org/resolver/configuration.html)
- CONFIRMED, why it exists: the enhanced local repository manager scopes "cached content by its origin and context: if you downloaded an artifact A1 from repository R1 ... but repository R1 is not defined in build, the A1 artifact ... should be handled as not present" — [Resolver: Local Repository](https://maven.apache.org/resolver/local-repository.html)
- CONFIRMED from source only, `resolver-status.properties`: `return metadataPath.getParent().resolve("resolver-status.properties");` and the key suffix `UPDATED_KEY_SUFFIX = ".lastUpdated"` — [DefaultUpdateCheckManager.java](https://raw.githubusercontent.com/apache/maven-resolver/master/maven-resolver-impl/src/main/java/org/eclipse/aether/internal/impl/DefaultUpdateCheckManager.java). I found no documentation page that names this file.
- CONFIRMED from source only, `maven-metadata-<repo>.xml`: the local path composer inserts the repository key before the extension: `result = metadataType.substring(0, idx) + '-' + repositoryKey + metadataType.substring(idx);` — [DefaultLocalPathComposer.java](https://raw.githubusercontent.com/apache/maven-resolver/master/maven-resolver-impl/src/main/java/org/eclipse/aether/internal/impl/DefaultLocalPathComposer.java)
- CONFIRMED, failed lookups are cached too: "a Java Properties file is created/updated (with the same filename as the cached artifact ... but with the additional suffix .lastUpdated) in the local repository." — [Resolver: Local Repository](https://maven.apache.org/resolver/local-repository.html)
- REFUTED, "Maven 4.0.0 is GA": the history page heads the section "Maven 4.0+ (not yet GA)" and says "The Apache Maven team maintains the last version of the last two series of GA releases. These are currently 3.10.0 and 3.9.16." The newest Maven 4 entry is `4.0.0-rc-7`, dated 2026-09-24. — [Maven Releases History](https://maven.apache.org/docs/history.html)
- NOT FOUND, a Maven 4 lockfile: the words "lock" and "lockfile" do not occur in the feature text of [What's new in Maven 4?](https://maven.apache.org/whatsnewinmaven4.html), nor as a feature in the [4.0.0-rc-7](https://maven.apache.org/docs/4.0.0-rc-7/release-notes.html) or [3.10.0](https://maven.apache.org/docs/3.10.0/release-notes.html) release notes. "Reproducible Builds are active by default" in 3.10.0 concerns `project.build.outputTimestamp` (archive timestamps), not dependency resolution.
- CONFIRMED, Maven 4 snapshot timestamps: "all subprojects will now have consistent SNAPSHOT timestamps in their packaged archives in local/remote repositories, while in Maven 3, each subproject had a different one." — [What's new in Maven 4?](https://maven.apache.org/whatsnewinmaven4.html)
- CONFIRMED, Maven 3.10.0 uses Resolver 2: "Maven Resolver 2.x (2.0.24) is used, the same lineage as in Maven 4+, which brings version range filtering, improved remote repository filtering, a transitive dependency manager and user-defined relocations" — [Maven 3.10.0 Release Notes](https://maven.apache.org/docs/3.10.0/release-notes.html)
- CONFIRMED, `maven-lockfile` file name: "`lockfileName` (`-DlockfileName=my-lockfile.json` default="lockfile.json")" — [chains-project/maven-lockfile README](https://raw.githubusercontent.com/chains-project/maven-lockfile/main/README.md)
- CONFIRMED, `maven-lockfile` version 1 fields, from the README example: each dependency has `groupId`, `artifactId`, `version`, `checksumAlgorithm` ("SHA-256"), `checksum`, `scope`, `resolved` (the artifact URL), `repositoryId`, `selectedVersion`, `included`, `id`, `parent`, `children`. "For each artifact, we store the hashes of all transitive dependencies in the `children` field." — [maven-lockfile README](https://raw.githubusercontent.com/chains-project/maven-lockfile/main/README.md)
- CONFIRMED, `maven-lockfile` version 2 keys artifacts by purl: "each artifact's coordinates, checksum and repository are stored once in a top-level `artifacts` table, keyed by [package URL]", for example `"artifact": "pkg:maven/org.junit.jupiter/junit-jupiter-api@5.9.2"`. — [maven-lockfile README](https://raw.githubusercontent.com/chains-project/maven-lockfile/main/README.md)
- CONFIRMED, its lineage: "This is close to the format of the lock file in the npm package-lock.json file." — [maven-lockfile README](https://raw.githubusercontent.com/chains-project/maven-lockfile/main/README.md)

### Inferences

- For a `-SNAPSHOT`, a range, `LATEST` or `RELEASE` in a POM, no committed file in a plain Maven project holds the resolved version. The exact pin is `observed in a local cache` at best: the timestamped file name and `maven-metadata-<repoId>.xml` under `~/.m2/repository/<group path>/<artifact>/<baseVersion>/`.
- A committed `lockfile.json` from `maven-lockfile` is the only "declared in a committed lockfile" case for Maven, and it is opt-in and third-party.
- `_remote.repositories` gives the origin repository id of a cached artifact, which is what a `repository_url` purl qualifier needs.

### Gaps

- I did not find a documentation page that states the on-disk file name of a locally cached timestamped snapshot (for example `foo-1.0-20261114.101500-3.jar` next to `foo-1.0-SNAPSHOT.jar`); the metadata model's `localCopy` field ("Whether to use a local copy instead (with filename that includes the base version)") is the nearest documented statement.
- How `maven-lockfile` records a `-SNAPSHOT` dependency (base version or timestamped version) is not stated in the README.

## Gradle: dynamic versions, changing modules, locking, verification, catalogs, composite builds, cache

### Takeaway

Gradle separates "dynamic versions" (lockable) from "changing modules" such as SNAPSHOTs (explicitly not lockable, not verified). `gradle.lockfile` is a flat list of `group:artifact:version=configurations` lines with no edges and no hashes; a version catalog is a declaration, not a pin.

### Cited Findings

- CONFIRMED, dynamic version forms: prefix range "`1.+`, `1.3.+`" — "Only versions exactly matching the portion before the `+` are included."; "`latest.integration`, `latest.release`" — "Matches the highest version with the specified status." — [Declaring Versions and Ranges](https://docs.gradle.org/current/userguide/dependency_versions.md)
- CONFIRMED, dynamic versus changing: "A dynamic version can be either a version range (e.g., `2.+`) or a placeholder for the latest available version (e.g., `latest.integration`)" and "a Maven `SNAPSHOT` module always points to the latest artifact published, making it a "changing module."" — [Declaring Versions and Ranges](https://docs.gradle.org/current/userguide/dependency_versions.md)
- CONFIRMED, ranges may select snapshots: "Given the published versions `1.0`, `1.1`, `1.2-SNAPSHOT`, and `2.0`, both `[1, 2)` and `1.+` select `1.2-SNAPSHOT`, because it is the highest version that matches." — [Declaring Versions and Ranges](https://docs.gradle.org/current/userguide/dependency_versions.md)
- CONFIRMED, any version can be made changing: "Gradle is flexible enough to treat any version as a changing version." (via `ExternalModuleDependency.setChanging(boolean)`). — [Declaring Versions and Ranges](https://docs.gradle.org/current/userguide/dependency_versions.md)
- CONFIRMED, 24-hour TTL: "By default, Gradle caches these dependencies (including dynamic versions and changing modules) for 24 hours, meaning it does not contact remote repositories for new versions during this time." — [Dependency Caching](https://docs.gradle.org/current/userguide/dependency_caching.md)
- CONFIRMED, the two settings: `resolutionStrategy.cacheDynamicVersionsFor(10, "minutes")` changes "how long Gradle will cache the resolved version for a dynamic version"; `resolutionStrategy.cacheChangingModulesFor(4, "hours")` changes "how long Gradle will cache the metadata and artifacts for a changing module". — [Dependency Caching](https://docs.gradle.org/current/userguide/dependency_caching.md)
- CONFIRMED, lock file name and place: "The lock state is preserved in a file named `gradle.lockfile`, located at the root of each project or subproject directory." The buildscript one "is named `buildscript-gradle.lockfile`." — [Locking Versions](https://docs.gradle.org/current/userguide/dependency_locking.md)
- CONFIRMED, lock file format: "***Each line*** represents a single dependency in the `group:artifact:version` format." and "After the version, the configurations that include the dependency are listed." Example line: `org.springframework:spring-core:5.0.5.RELEASE=compileClasspath, runtimeClasspath`; last line `empty=annotationProcessor`. — [Locking Versions](https://docs.gradle.org/current/userguide/dependency_locking.md)
- CONFIRMED by inspection, no edges and no hashes: the documented format and both example files contain only `group:artifact:version=configurations` lines, three comment lines and the `empty=` line. The page does not use the words "edge" or "hash" for the lockfile; integrity is handled by the separate verification file. — [Locking Versions](https://docs.gradle.org/current/userguide/dependency_locking.md)
- CONFIRMED, warning about changing modules: "Dependency locking is effective with dynamic versions, but it ***should not be used*** with changing versions (e.g., `-SNAPSHOT`), where the coordinates remain the same, but the content may change." and "Gradle will emit a warning when persisting the lock state if changing dependencies are present in the resolution result." — [Locking Versions](https://docs.gradle.org/current/userguide/dependency_locking.md)
- CONFIRMED, how the lock is applied and generated: "Gradle adds locked versions as `strictly` dependency constraints during resolution."; generated with `./gradlew dependencies --write-locks`; "Lockfiles should be checked in to source control." — [Locking Versions](https://docs.gradle.org/current/userguide/dependency_locking.md)
- CONFIRMED, verification file: "This configuration file is located at $PROJECT_ROOT/gradle/verification-metadata.xml." generated with `gradle --write-verification-metadata sha256 help`. — [Dependency verification, Gradle 9.2.0](https://docs.gradle.org/9.2.0/userguide/dependency_verification.html). The `current/` URL for this page returned an empty stub on 2026-11-14, so the 9.2.0 copy was read.
- CONFIRMED, verification is off by default and skips snapshots: "Gradle supports both checksum and signature verification out of the box but performs no dependency verification by default." and "Gradle will not verify changing dependencies (in particular SNAPSHOT dependencies) nor locally produced artifacts" — [Dependency verification, Gradle 9.2.0](https://docs.gradle.org/9.2.0/userguide/dependency_verification.html)
- CONFIRMED, a version catalog is a declaration, not a pin: "Version catalogs declare *requested* versions but do not enforce them." and "version catalogs declare requested versions but do not enforce them during conflict resolution." It lives in "a `libs.versions.toml` file located in the `gradle` subdirectory of the root build" and may hold ranges: `version = { strictly = "[3.8, 4.0[", prefer = "3.9" }`. — [Version Catalogs](https://docs.gradle.org/current/userguide/version_catalogs.md)
- CONFIRMED, composite builds substitute sources for binaries: "Gradle automatically substitutes the external dependency with a local source, so changes to the library are immediately visible in the consuming project." via `includeBuild("../my-utils")` in the settings file. — [Composite Builds](https://docs.gradle.org/current/userguide/composite_builds.md)
- CONFIRMED (2018 source), source dependencies: "when Gradle needs to find a version of the "utilities" library, it will look for a matching tag in the Git repository."; a branch may be requested with `version { branch = 'release' }`; and "Dependency locking does not support source dependencies." — [Gradle blog, Introducing source dependencies, 2018-09-25](https://blog.gradle.org/introducing-source-dependencies)
- CONFIRMED in part, what the modules cache records: "A file-based store of downloaded artifacts ... Artifacts are stored under a checksum, so name clashes will not cause issues." and "A binary store of resolved module metadata, including the results of resolving dynamic versions, module descriptors, and artifacts." — [Dependency Caching](https://docs.gradle.org/current/userguide/dependency_caching.md)
- CONFIRMED, the metadata cache holds the dynamic-version result: "The result of resolving a dynamic version (e.g. `1.+`) to a concrete version (e.g. `1.2`)." and "Every entry in the metadata cache includes a record of the repository that provided the information as well as a timestamp that can be used for cache expiry." — [Dependency Caching](https://docs.gradle.org/current/userguide/dependency_caching.md)
- CONFIRMED, cache directory names: the layout table lists `modules-2`, `files-2.1` and `metadata-2.107` for "Gradle 8.11 and above". — [Dependency Caching](https://docs.gradle.org/current/userguide/dependency_caching.md)
- NOT FOUND, the exact path shape `files-2.1/<group>/<module>/<version>/<sha1>/`: the documentation says only that artifacts are "stored under a checksum"; it names neither SHA-1 nor the directory order.

### Inferences

- The concrete version a dynamic version resolved to is in the binary `metadata-2.x` store, which the documentation describes as binary; it is not readable as text. What a static reader can take from the cache is the set of `<version>` directories present under `files-2.1`, which is evidence a version was downloaded, not evidence it is the one selected.
- For a Gradle project without `gradle.lockfile`, a dynamic version is "resolved now" or unknown; with it, "declared in a committed lockfile" (name and version only, no hash, no origin repository).

### Gaps

- No current (9.9.1) documentation page for Gradle source dependencies was found; the 2018 blog post is the only primary source read, and the feature's present status was not verified.
- The on-disk path layout of `files-2.1` and the hash algorithm in the path were not found in documentation.

## JitPack: a GitHub repository as a Maven coordinate

### Takeaway

JitPack maps `com.github.User:Repo:Version` where Version is a release tag, a commit hash or `branch-SNAPSHOT`; only the first two are pins, and a tag is only as stable as the Git tag.

### Cited Findings

- CONFIRMED, coordinate mapping: "Group: com.github.Username / Artifact: Repository Name / Version: Release tag, commit hash or master-SNAPSHOT" and `implementation 'com.github.User:Repo:Version'`. — [JitPack docs](https://jitpack.io/docs/)
- CONFIRMED, snapshot forms: "You can specify a version for your dependency as: commit hash / branch-SNAPSHOT (replace 'branch' with any branch name, e.g. master)" and "Adding -SNAPSHOT will build the latest commit on the master branch." — [JitPack docs](https://jitpack.io/docs/)
- CONFIRMED, short commit id: "If the project doesn't have any GitHub Releases, you can get the latest snapshot build. In this case, use the short commit id as the version." — [JitPack docs](https://jitpack.io/docs/)
- CONFIRMED, pull requests: "Simply use PR<NR>-SNAPSHOT as the version." — [JitPack docs](https://jitpack.io/docs/)
- CONFIRMED, multi-module: "To get individual artifacts of multi-module builds use com.github.User.Repo as group Id and ModuleName as the artifact Id." — [JitPack: Building](https://docs.jitpack.io/building/)
- CONFIRMED, other hosts: "BitBucket: org.bitbucket.Username:Repo:Tag / GitLab: com.gitlab.Username:Repo:Tag"; a custom domain can replace `com.github.yourcompany`. — [JitPack: Building](https://docs.jitpack.io/building/)
- CONFIRMED, immutability window: "Public repository artifacts on JitPack are immutable after 7 days of publishing." and "JitPack will also keep hosting artifacts after the originating git repository is deleted." — [JitPack: Building](https://docs.jitpack.io/building/)

### Inferences

- A Maven or Gradle coordinate whose group starts with `com.github.`, `com.gitlab.` or `org.bitbucket.` and is served from `https://jitpack.io` can be turned into a repository identity (`pkg:github/user/repo@<version>`) without network access; the commit is known offline only when the version is a commit hash.

### Gaps

- The docs do not say whether a JitPack tag build is rebuilt if the Git tag moves within the 7-day window.

## npm family: git dependencies, lockfile records, workspaces, dist-tags, overrides

### Takeaway

All five lockfiles record the resolved commit of a git dependency; the encoding differs (npm: URL with `#<sha>` in `resolved`; pnpm: `{type: git, repo, commit}` or a host tarball URL; Yarn Berry: `#commit=<sha>`; Yarn classic: `url#<sha>`; Bun: `github:owner/repo#<short sha>`).

### Cited Findings

- CONFIRMED, npm git URL grammar: `<protocol>://[<user>[:<password>]@]<hostname>[:<port>][:][/]<path>[#<commit-ish> | #semver:<semver>]` where "`<protocol>` is one of `git`, `git+ssh`, `git+http`, `git+https`, or `git+file`." — [npm package.json docs (source)](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-json.md)
- CONFIRMED, npm semantics: "If `#<commit-ish>` is provided, it will be used to clone exactly that commit." "If neither `#<commit-ish>` or `#semver:<semver>` is specified, then the default branch is used." Example: `git+ssh://git@github.com:npm/cli#semver:^5.0`. — [npm package.json docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-json.md)
- CONFIRMED, `#semver:` matches tags: "npm will look for any tags or refs matching that range in the remote repository, much as it would for a registry dependency." — [npm package.json docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-json.md)
- CONFIRMED, GitHub shorthand: "you can refer to GitHub URLs as just "foo": "user/foo-project". Just as with git URLs, a `commit-ish` suffix can be included." Also `"package-name": "github:username/repo#branch-name"`. — [npm package.json docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-json.md)
- CONFIRMED, `package-lock.json` records the commit: `resolved` — "In the case of git dependencies, this will be the full git url with commit sha. In the case of link dependencies, this will be the location of the link target." — [npm package-lock.json docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-lock-json.md)
- CONFIRMED, legacy `dependencies` section: "git sources: This is a git specifier with resolved committish. (eg, `git+https://example.com/foo/bar#115311855adb0789a0466714ed48a1499ffea97e`)" and, for `integrity`, "For git dependencies, this is the commit sha." — [npm package-lock.json docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-lock-json.md)
- CONFIRMED, links in the lock: "link: A flag to indicate that this is a symbolic link. If this is present, no other fields are specified, since the link target will also be included in the lockfile." — [npm package-lock.json docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-lock-json.md)
- CONFIRMED, dist-tags slide: "Publishing a package sets the `latest` tag to the published version unless the `--tag` option is used." and "By default, `npm install <pkg>` (without any `@<version>` or `@<tag>` specifier) installs the `latest` tag." A tag is a valid dependency spec: "`tag` A specific version tagged and published as `tag`". — [npm dist-tag docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/commands/npm-dist-tag.md); [npm package.json docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-json.md)
- CONFIRMED, npm `overrides`: "Overrides provide a way to replace a package in your dependency tree with another version, or another package entirely." The value may be "an exact version, a semver range, a dist-tag, or a replacement specifier such as `npm:`, `file:`, or a Git URL." — [npm package.json docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-json.md)
- CONFIRMED, npm workspaces: "The optional `workspaces` field is an array of file patterns that describes locations within the local file system that the install client should look up to find each workspace that needs to be symlinked to the top level `node_modules` folder." — [npm package.json docs](https://raw.githubusercontent.com/npm/cli/latest/docs/lib/content/configuring-npm/package-json.md)
- CONFIRMED, pnpm git specifiers: `pnpm add kevva/is-positive#97edff6f525f192a3f83cea1944765f769ae2678` (commit), `#master` (branch), `#semver:^2.0.0` (range), `#path:/packages/simple-react-app` (subdirectory). — [pnpm package sources (source)](https://raw.githubusercontent.com/pnpm/pnpm.io/main/docs/package-sources.md)
- CONFIRMED, pnpm lock shapes for git (pnpm v12): "the lockfile records one of two shapes: the **host's source archive**, a plain tarball download ... otherwise a **`git` resolution over the canonical HTTPS URL**". "pnpm never records an SSH URL for these hosts." — [pnpm package sources](https://raw.githubusercontent.com/pnpm/pnpm.io/main/docs/package-sources.md)
- CONFIRMED, pnpm treats the specifier as identity: "For repositories on GitHub, GitLab, Bitbucket, and GitHub Gist, the specifier is an **identity**, not a choice of transport." For other hosts, "the URL *is* the identity." — [pnpm package sources](https://raw.githubusercontent.com/pnpm/pnpm.io/main/docs/package-sources.md)
- CONFIRMED from source, pnpm resolution types: `interface GitRepositoryResolution { type: 'git'; repo: string; commit: string; path?: string }`, `interface TarballResolution { type?: undefined; tarball: string; integrity?: string; path?: string ... }`, `interface DirectoryResolution { type: 'directory'; directory: string }`. — [pnpm lockfile types](https://raw.githubusercontent.com/pnpm/pnpm/main/pnpm11/lockfile/types/src/index.ts)
- CONFIRMED, `pnpm-lock.yaml` v9 layout: `packages` is "a mapping of dependency ID (`<pkg_name>@<pkg_version>`) to dependency object"; `snapshots` is "a mapping of all the dependencies of dependencies"; `importers[relativePath][dependencyType][dependencyName]` carries `specifier` and `version`. — [pnpm lockfile spec 9.0](https://raw.githubusercontent.com/pnpm/spec/master/lockfile/9.0.md)
- CONFIRMED, pnpm workspace protocol: "if you set "foo": "workspace:2.0.0" ..." and, on publish, pnpm replaces a `workspace:` dependency by "The corresponding version in the target workspace (if you use workspace:, workspace:*, workspace:~, or workspace:^)". — [pnpm workspaces](https://pnpm.io/workspaces)
- CONFIRMED, Yarn Berry git protocol: "You can explicitly request a tag, commit, branch, or semver tag": `...berry.git#tag=@yarnpkg/cli/2.2.0`, `#commit=a806c88`, `#head=master`; workspaces via `#workspace=@yarnpkg/shell&tag=...`. — [Yarn: Git Protocol](https://yarnpkg.com/protocol/git)
- CONFIRMED from source, Yarn Berry resolves every form to a commit: `resolveUrl` returns `normalizeRepoUrl(`${repo}#${resolve(protocol, request)}`)` where each branch yields `querystring.stringify({...extra, commit: ...})`. — [yarnpkg/berry plugin-git gitUtils.ts](https://raw.githubusercontent.com/yarnpkg/berry/master/packages/plugin-git/sources/gitUtils.ts). I read no documentation page that shows the resulting `yarn.lock` entry.
- CONFIRMED, Yarn Berry local protocols: `link:` "can only point to arbitrary folders ... cannot have a package.json file"; `portal:` target "is assumed to be a package"; `workspace:` "can be used to reference other workspaces in a monorepo setup". — [Yarn link protocol](https://yarnpkg.com/protocol/link), [Yarn portal protocol](https://yarnpkg.com/protocol/portal), [Yarn workspace protocol](https://yarnpkg.com/protocol/workspace)
- CONFIRMED, Yarn `resolutions`: "This field allows you to instruct Yarn to use a specific resolution (specific package version) instead of anything the resolver would normally pick." and "The resolutions field can only be set at the root of the project". — [Yarn manifest](https://yarnpkg.com/configuration/manifest)
- CONFIRMED, Yarn classic lock format: entries are keyed by `name@range` with `version "1.0.3"` and `resolved "https://registry.npmjs.org/package-1/-/package-1-1.0.3.tgz#<hash>"`; "All yarn.lock files should be checked into source control". — [Yarn classic: yarn.lock](https://classic.yarnpkg.com/en/docs/yarn-lock)
- CONFIRMED from source, Yarn classic git entry: the git resolver sets `resolved: `${url}#${commit}`` and `hash: commit`. — [yarnpkg/yarn git-resolver.js](https://raw.githubusercontent.com/yarnpkg/yarn/master/src/resolvers/exotics/git-resolver.js)
- CONFIRMED, Bun lockfile: "bun install creates a lockfile called bun.lock." and "Bun v1.2 changed the default lockfile format to the text-based bun.lock." — [Bun: Lockfile](https://bun.sh/docs/pm/lockfile)
- CONFIRMED, Bun git entry: for a manifest spec `"uNetworking/uWebSockets.js#v20.51.0"` the lock holds `"uWebSocket.js": ["uWebSockets.js@github:uNetworking/uWebSockets.js#6609a88", {}, "uNetworking-uWebSockets.js-6609a88"]`. — [Bun blog: text lockfile](https://bun.sh/blog/bun-lock-text-lockfile)
- CONFIRMED, Bun git protocols and overrides: "Bun supports a variety of protocols, including `github`, `git`, `git+ssh`, and `git+https`."; "Bun supports npm's "overrides" and Yarn's "resolutions" in package.json." — [Bun add docs (source)](https://raw.githubusercontent.com/oven-sh/bun/main/docs/pm/cli/add.mdx); [Bun overrides](https://bun.sh/docs/pm/overrides)

### Inferences

- The Bun blog example shows a 7-character commit prefix for a GitHub dependency. A pin taken from `bun.lock` may therefore be an abbreviated commit, unlike npm, pnpm and Yarn, which the sources show with a full hash or a `commit` field.
- A pnpm v12 lock entry for a GitHub dependency can be a tarball URL rather than a `git` resolution; the commit then has to be parsed out of the URL.
- `workspace:`, `link:`, `portal:` and `file:` entries point inside or beside the repository and have no version pin; they are path identities.

### Gaps

- No documentation page read shows the literal text of a Yarn Berry `yarn.lock` entry for a git dependency (`resolution:` field); only the source code was read.
- Whether `bun.lock` always abbreviates commits or only for the `github:` shorthand was not established.
- The pnpm documentation does not show the tarball URL pattern for host archives.

## Go modules

### Takeaway

`go.mod` is itself the pin (exact versions or pseudo-versions carrying a 12-character commit prefix and commit time); `go.sum` adds hashes; `vendor/modules.txt` repeats the module versions for vendored trees. `replace` to a local path is the one unpinned form.

### Cited Findings

- CONFIRMED, pseudo-version parts: "A revision identifier (abcdefabcdef), which is a 12-character prefix of the commit hash" and "A timestamp (yyyymmddhhmmss), which is the UTC time the revision was created. In Git, this is the commit time, not the author time." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, the three forms: "vX.0.0-yyyymmddhhmmss-abcdefabcdef is used when there is no known base version."; "vX.Y.Z-pre.0.yyyymmddhhmmss-abcdefabcdef is used when the base version is a pre-release version"; "vX.Y.(Z+1)-0.yyyymmddhhmmss-abcdefabcdef is used when the base version is a release version". — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, not unique per commit: "More than one pseudo-version may refer to the same commit by using different base versions." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, `replace`: "If the path on the right side of the arrow is an absolute or relative path (beginning with ./ or ../), it is interpreted as the local file path to the replacement module root directory ... The replacement version must be omitted in this case." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, scope of `replace` and `exclude`: "replace directives only apply in the main module's go.mod file and are ignored in other modules." and "exclude directives only apply in the main module's go.mod file and are ignored in other modules." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, `go.work`: "A workspace can be declared in a go.work file that specifies relative paths to the module directories of each of the modules in the workspace." "a replace in go.work overrides a version-specific replace in a go.mod file" (wildcard form); "It is generally inadvisable to commit go.work files into version control systems". — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, `vendor/modules.txt`: "go mod vendor also creates the file vendor/modules.txt that contains a list of vendored packages and the module versions they were copied from." and "When the go command reads vendor/modules.txt, it checks that the module versions are consistent with go.mod." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED from source, `modules.txt` line shapes: module lines begin `"# "` and may carry `" => "` for a replacement; annotation lines are `"## explicit; go %s\n"`, `"## explicit\n"`, `"## go %s\n"`, and `"## workspace\n"`. — [cmd/go/internal/modcmd/vendor.go](https://raw.githubusercontent.com/golang/go/master/src/cmd/go/internal/modcmd/vendor.go)
- CONFIRMED, vendoring switch: "If the vendor directory is present in the main module's root directory, it will be used automatically if the go version in the main module's go.mod file is 1.14 or higher. To explicitly enable vendoring, invoke the go command with the flag -mod=vendor." — [Go Modules Reference](https://go.dev/ref/mod). The spelling `GOFLAGS=-mod=vendor` was not searched for separately.
- CONFIRMED, proxy protocol: "`$base/$module/@v/list` Returns a list of known versions of the given module in plain text, one per line. This list should not include pseudo-versions." and `$base/$module/@v/$version.info` returns JSON `{Version, Time}`; "this endpoint may be used to find versions for branch names or revision identifiers." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, case encoding: "the $module and $version elements are case-encoded by replacing every uppercase letter with an exclamation mark followed by the corresponding lower-case letter." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, `go.sum`: "Each line in go.sum has three fields separated by spaces: a module path, a version (possibly ending with /go.mod), and a hash." "Currently, SHA-256 (h1) is the only supported hash algorithm." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, checksum database: "The checksum database is served by sum.golang.org, which is run by Google. It is a Transparent Log (or "Merkle Tree") of go.sum line hashes" and it "ensures that the bits associated with a specific version do not change from one day to the next, even if the module's author subsequently alters the tags". — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, `GOPRIVATE`: "list of glob patterns of module path prefixes that should be considered private. Acts as a default value for GONOPROXY and GONOSUMDB." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, module path to repository: "If the module path has a VCS qualifier (one of .bzr, .fossil, .git, .hg, .svn) at the end of a path component, the go command will use everything up to that path qualifier as the repository URL." Otherwise it "sends an HTTP GET request to a URL derived from the module path with a ?go-get=1 query string." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, subdirectory tags: "The version v0.4.0 of that module must have the tag named gopls/v0.4.0 in that repository." (module `golang.org/x/tools/gopls`). — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, major suffix: "If a module is released at major version v2 or higher, its path must have a major version suffix." and its `go.mod` "may be in either foo/bar or foo/bar/v2." `+incompatible` marks "A tag with major version v2 or higher" on a module without a suffix and without `go.mod`. — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED, module cache location: "The default location of the module cache is $GOPATH/pkg/mod. To use a different location, set the GOMODCACHE environment variable." — [Go Modules Reference](https://go.dev/ref/mod)

### Inferences

- A pseudo-version gives a 12-character commit prefix offline; the full 40-character hash needs the cache or the network (the `.info` endpoint's `Origin` data was not checked).
- Mapping module path, version to repository and tag is a pure function for hosts with known layouts (`github.com/owner/repo[/subdir][/vN]` gives tag `[subdir/]vX.Y.Z`), except that the `/vN` element may be either a path suffix only or a real directory, which cannot be told apart without the repository contents.
- A `replace` to a local path has no version; it is a path identity, as with Cargo `path` and npm `file:`.

### Gaps

- The exact file layout of the module cache's download area (`cache/download/<module>/@v/<version>.info`) was not located in the text I searched; treat it as unverified.
- Whether `.info` files in the cache carry the full commit hash was not checked.

## Cargo

### Takeaway

`Cargo.lock` records every package with `name`, `version`, and a `source` string; git packages carry the full commit after `#`, registry packages carry a `checksum`, path packages have no `source` at all.

### Cited Findings

- CONFIRMED, git source line: `source = "git+https://github.com/rust-lang/regex.git#9f9f693768c584971a4d53bc3c586c33ed3a6831"` and "Cargo will take the latest commit and write that information out into your Cargo.lock when you build for the first time." — [Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
- CONFIRMED, default git ref: "Cargo assumes that we intend to use the latest commit on the default branch to build our package" unless `rev`, `tag` or `branch` is given; "Anything that is not a branch or a tag falls under rev key." — [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- CONFIRMED, locking of git commits: "Cargo locks the commits of git dependencies in Cargo.lock file at the time of their addition and checks for updates only when you run cargo update command." — [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- CONFIRMED, the `?rev=`, `?branch=`, `?tag=` query: the package id grammar is `[ kind "+" ] proto "://" hostname-and-path [ "?" query] [ "#" ( pkgname | semver ) ]` with `query = ( "branch" | "tag" | "rev" ) "=" ref` and `kind = "registry" | "git" | "path"`. — [Package ID Specifications](https://doc.rust-lang.org/cargo/reference/pkgid-spec.html)
- CONFIRMED from source, branch encoding changed: "the encoding of `git` dependencies in lock files changed where `branch = "master"` is now encoded with `branch=master` instead of with nothing at all." — [cargo src/resolver/encode.rs](https://raw.githubusercontent.com/rust-lang/cargo/master/src/resolver/encode.rs)
- CONFIRMED from source, path packages: "All packages have a `source` listed to indicate where they come from. For `path` dependencies, however, no `source` is listed." and therefore "all packages with `path` sources must have unique names." — [cargo src/resolver/encode.rs](https://raw.githubusercontent.com/rust-lang/cargo/master/src/resolver/encode.rs)
- CONFIRMED from source, registry checksum: "All packages from registries contain a `checksum` which is a sha256 checksum of the tarball the package is associated with." — [cargo src/resolver/encode.rs](https://raw.githubusercontent.com/rust-lang/cargo/master/src/resolver/encode.rs)
- CONFIRMED from source, edges: "Dependency edges are specified in a compact form where possible where just the name is listed. The version/source on dependency edges are only listed if necessary to disambiguate". — [cargo src/resolver/encode.rs](https://raw.githubusercontent.com/rust-lang/cargo/master/src/resolver/encode.rs)
- CONFIRMED from source, lock versions: V3 "explicitly lists a `version` at the top of the file"; V4 — "`SourceId` URL serialization is aware of URL encoding ... New lockfiles use V4 by default starting in 1.83."; V5 is "Unstable." — [cargo src/resolver/resolve.rs](https://raw.githubusercontent.com/rust-lang/cargo/master/src/resolver/resolve.rs)
- CONFIRMED, default requirement is a range: "Caret requirements are the default version requirement strategy." and "Leaving off the caret is a simplified equivalent syntax to using caret requirements." — [Specifying Dependencies](https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html)
- CONFIRMED, `[patch]`: "The [patch] section of Cargo.toml can be used to override dependencies with other copies." and "[patch] is applicable transitively but can only be defined at the top level". — [Overriding Dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html)
- CONFIRMED, `[replace]`: "[replace] is deprecated. You should use the [patch] table instead." — [Overriding Dependencies](https://doc.rust-lang.org/cargo/reference/overriding-dependencies.html)
- CONFIRMED, workspaces: "All packages share a common Cargo.lock file which resides in the workspace root." — [Workspaces](https://doc.rust-lang.org/cargo/reference/workspaces.html)
- CONFIRMED, alternate registries: `some-crate = { version = "1.0", registry = "my-registry" }` with `my-registry = { index = "https://my-intranet:8080/git/index" }` in config; "crates.io does not accept packages that depend on crates from other registries." — [Registries](https://doc.rust-lang.org/cargo/reference/registries.html)
- CONFIRMED, commit policy: "When in doubt, check Cargo.lock into the version control system (e.g. Git)." — [Cargo.toml vs Cargo.lock](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)

### Inferences

- The specific string `source = "git+https://...?rev=...#<commit>"` is the composition of the documented `#<commit>` example and the documented `?rev=` query grammar; no page read shows one literal line containing both.
- A path package in `Cargo.lock` cannot be told from a workspace member by the lock alone; `Cargo.toml` `[workspace] members` and `path =` keys are needed.

### Gaps

- No documentation page gives a normative grammar of `Cargo.lock`; the facts above come from the guide's example and from source comments.

## Git submodules, subtrees and vendored copies

### Takeaway

A submodule's pin is the gitlink entry in the superproject tree (mode 160000) and is always present in committed data even when the submodule directory is empty. The claim that `git subtree` leaves no pin is partly wrong: its squash and rejoin commits write `git-subtree-dir` and `git-subtree-split` trailers.

### Cited Findings

- CONFIRMED, gitlink holds the commit: "The gitlink entry contains the object name of the commit that the superproject expects the submodule's working directory to be at." — [gitsubmodules](https://git-scm.com/docs/gitsubmodules)
- CONFIRMED, mode: "Notice the 160000 mode for the DbConnector entry." (`create mode 160000 DbConnector`); in the index the object type bits are "1000 (regular file), 1010 (symbolic link) and 1110 (gitlink)". — [Pro Git: Submodules](https://git-scm.com/book/en/v2/Git-Tools-Submodules); [index-format](https://git-scm.com/docs/index-format)
- CONFIRMED, `.gitmodules` `path`: "Defines the path, relative to the top-level directory of the Git working tree, where the submodule is expected to be checked out." — [gitmodules](https://git-scm.com/docs/gitmodules)
- CONFIRMED, `.gitmodules` `url`: "This may be either an absolute URL ready to be passed to git-clone or (if it begins with ./ or ../) a location relative to the superproject's origin repository." — [gitmodules](https://git-scm.com/docs/gitmodules)
- CONFIRMED, `.gitmodules` `branch`: "A remote branch name for tracking updates in the upstream submodule. If the option is not specified, it defaults to the remote HEAD." A value of `.` means the superproject's current branch name. — [gitmodules](https://git-scm.com/docs/gitmodules)
- CONFIRMED, `git submodule status`: "Each SHA-1 will possibly be prefixed with - if the submodule is not initialized, + if the currently checked out submodule commit does not match the SHA-1 found in the index of the containing repository and U if the submodule has merge conflicts." With `--cached` it prints "the SHA-1 recorded in the superproject". — [git-submodule](https://git-scm.com/docs/git-submodule)
- CONFIRMED, empty directory: "The DbConnector directory is there, but empty." and, for a deinitialized submodule, "The directory which is supposed to be the working directory is empty instead." — [Pro Git: Submodules](https://git-scm.com/book/en/v2/Git-Tools-Submodules); [gitsubmodules](https://git-scm.com/docs/gitsubmodules)
- CONFIRMED, subtree needs no special construct: "Unlike submodules, subtrees do not need any special constructions (like '.gitmodules' files or gitlinks) be present in your repository" and "A subtree is just a subdirectory". — [git-subtree documentation](https://raw.githubusercontent.com/git/git/master/contrib/subtree/git-subtree.adoc)
- REFUTED in part, "git subtree leaves no pin": the script writes commit-message trailers `git-subtree-dir: $dir`, `git-subtree-mainline: $latest_old` and `git-subtree-split: $latest_new`, and finds earlier ones with `git log --grep="^git-subtree-dir: $dir/*\$"`. A squash commit subject is `Squashed '$dir/' changes from $oldsub_short..$newsub_short`. — [git-subtree.sh](https://raw.githubusercontent.com/git/git/master/contrib/subtree/git-subtree.sh). No file in the tree records the upstream URL or commit; the trailers exist only in history, and a non-squash `subtree add` carries the upstream commits themselves as merge parents.

### Inferences

- The submodule pin is readable offline from committed data without initialising the submodule: `git ls-tree` on the superproject commit, or `git submodule status --cached`. This is the "declared in committed files" label, with identity from `.gitmodules` `url`.
- A subtree's upstream commit is recoverable from `git log` when the trailers are present, but the upstream repository URL is not recorded by `git subtree`. A plain vendored copy (files copied in) has neither; this last point is an inference, since no source read makes a statement about copied directories.

### Gaps

- The git-ls-tree page was fetched but I did not extract a sentence that states the `160000 commit <sha>` output form; the mode is confirmed from Pro Git and index-format.

## GitHub Actions

### Takeaway

Workflow dependency lockfiles are generally available since 2026-10-28. The file `.github/workflows/actions.lock` pins actions and their transitive action dependencies to commits, records `ref`, `commit`, `owner_id` and `repo_id`, and does not cover reusable-workflow calls or local `./` actions.

### Cited Findings

- CONFIRMED, `uses` forms: "`{owner}/{repo}@{ref}` — You can specify a branch, ref, or SHA in a public GitHub repository."; a subdirectory form "at a specific branch, ref, or SHA"; `./path/to/dir`; `docker://{image}:{tag}` (examples `docker://alpine:3.8`, `docker://ghcr.io/OWNER/IMAGE_NAME`). — [Workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)
- CONFIRMED, new self reference: "The $/ prefix is the self repository reference. It ... resolves to that repository at the running commit". "A $/ reference must not include an @{ref} suffix." "The $/ syntax is not available in GitHub Enterprise Server." — [Workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)
- CONFIRMED, reusable workflows: "`{owner}/{repo}/.github/workflows/{filename}@{ref}` for reusable workflows in public and private repositories."; also `$/.github/workflows/{filename}` and `./.github/workflows/{filename}`. — [Workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax)
- CONFIRMED, guidance: "Using the commit SHA of a released action version is the safest for stability and security." and "a tag can be moved or deleted if a bad actor gains access to the repository storing the action." — [Workflow syntax](https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax); [Secure use reference](https://docs.github.com/en/actions/reference/security/secure-use)
- CONFIRMED, general availability: "You can now use GitHub Actions workflow dependency lockfiles in general availability on GitHub Enterprise Cloud, GitHub Enterprise Cloud with data residency, Team, and Free plans." (October 28, 2026) — [GitHub Changelog: workflow lockfiles are generally available](https://github.blog/changelog/2026-10-28-github-actions-workflow-lockfiles-are-generally-available/)
- CONFIRMED, preview date: "GitHub Actions workflow dependency lockfiles are now in public preview" (October 14, 2026). — [GitHub Changelog: public preview](https://github.blog/changelog/2026-10-14-github-actions-workflow-lockfiles-in-public-preview/)
- CONFIRMED, file and generator: "The lockfile is defined in a repository at .github/workflows/actions.lock. It's generated by gh actions-lock, a GitHub CLI extension, and should never be edited by hand." Install: `gh extension install github/gh-actions-lock`; run `gh actions-lock` from the repository root. — [Workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles); [Locking action dependencies](https://docs.github.com/en/actions/how-tos/secure-your-work/lock-workflow-dependencies)
- CONFIRMED, format: "The lockfile has two sections. workflows records which dependencies each onboarded workflow uses, and dependencies records which commit each of those references resolves to." The example shows `version: 'v0.0.3'`, a `workflows:` map from workflow path to a list such as `'octo-org/deploy-action@v3.1.0'`, and a `dependencies:` map with `ref: 'v3.1.0'`, `commit: 'sha1-...'`, `owner_id: 44036562`, `repo_id: 197814629`. — [Workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles)
- CONFIRMED, field meanings: "ref: The tag or branch the commit was resolved from. commit: The exact commit the reference resolved to. This is the code that will run. owner_id and repo_id: The numeric identity of the source repository." — [Workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles)
- CONFIRMED, coverage and its limit: "A workflow dependency lockfile pins a commit SHA to every action your workflows depend on, including transitive dependencies. It doesn't pin calls to reusable workflows (jobs.<job_id>.uses). To pin a reusable workflow, reference it by its full commit SHA." — [Workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles)
- CONFIRMED, reusable workflows have their own: "A reusable workflow's dependencies are covered by the lockfile of the repository where it's defined, not by the lockfile of the repository that calls it." — [Workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles)
- CONFIRMED, local actions: "The extension never follows or locks a local path action (uses: ./…)." — [Locking action dependencies](https://docs.github.com/en/actions/how-tos/secure-your-work/lock-workflow-dependencies)
- CONFIRMED, opt-in per workflow and enforced at run time: "Onboard workflows individually. GitHub Actions requires each dependency of an onboarded workflow to be in the lockfile and runs its recorded commit." — [GitHub Changelog, GA entry](https://github.blog/changelog/2026-10-28-github-actions-workflow-lockfiles-are-generally-available/)
- CONFIRMED, relation to SHA pinning: "For repositories that adopt a lockfile, SHA pinning within workflow files is no longer necessary." — [Workflow dependency lockfiles](https://docs.github.com/en/actions/concepts/security/workflow-dependency-lockfiles)
- CONFIRMED, immutable releases: "Immutable releases are releases where the assets and associated Git tag cannot be changed after publication." and "Once an immutable release is published, its associated Git tag is locked to a specific commit, cannot be changed, and cannot be deleted while the release exists." — [Immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)
- CONFIRMED, tag reuse blocked: "Even if you delete a repository and create a new one with the same name, you cannot reuse tags that were associated with immutable releases in the original repository." — [Immutable releases](https://docs.github.com/en/code-security/concepts/supply-chain-security/immutable-releases)
- Conflict noted: the how-to page's "Who can use this feature?" box lists only "GitHub Enterprise Cloud", while both changelog entries list Enterprise Cloud, Team and Free plans. — [Locking action dependencies](https://docs.github.com/en/actions/how-tos/secure-your-work/lock-workflow-dependencies); [GA changelog entry](https://github.blog/changelog/2026-10-28-github-actions-workflow-lockfiles-are-generally-available/)

### Inferences

- With `actions.lock` committed, an action reference is "declared in a committed lockfile", and `owner_id`/`repo_id` give a rename-proof identity. Without it, only `uses: owner/repo@<40-hex sha>` is a pin; tags and branches are sliding unless the tag belongs to an immutable release, which cannot be known offline.
- Workflows not listed under `workflows:` are not onboarded, so their action references stay unlocked even when the file exists.
- The lockfile schema version shown is `v0.0.3`, which suggests the format may still change.

### Gaps

- Neither docs page says whether `docker://` actions or container images used by Docker-based actions appear in `actions.lock`; NOT FOUND.
- GitHub Enterprise Server availability of lockfiles is not stated in the pages read.
- The literal encoding of `commit` (the example shows `'sha1-...'`, elided) was not verified against a real file.

## Container images as dependencies

### Takeaway

A tag is mutable and a digest is the pin; both a Dockerfile `FROM` and a Compose `image:` accept `@sha256:...`. No Docker-native lockfile was found in the pages read.

### Cited Findings

- CONFIRMED, tags slide: "Image tags are mutable, meaning a publisher can update a tag to point to a new image." and "if you specify FROM alpine:3.21 in your Dockerfile, 3.21 resolves to the latest patch version for 3.21." — [Docker: Building best practices](https://docs.docker.com/build/building/best-practices/)
- CONFIRMED, digest pin: `FROM alpine:3.21@sha256:a8560b36e8b8210634f77d9f7f9efd7ffa463e380b75e2e74aff4511df3ef88c`; "By pinning your images to a digest, you're guaranteed to always use the same image version". — [Docker: Building best practices](https://docs.docker.com/build/building/best-practices/)
- CONFIRMED, `FROM` grammar and default: `FROM [--platform=<platform>] <image>[:<tag>] [AS <name>]` or `<image>[@<digest>]`; "If you omit either of them, the builder assumes a latest tag by default." — [Dockerfile reference](https://docs.docker.com/reference/dockerfile/)
- CONFIRMED, Compose `image`: "image must follow the Open Container Specification addressable image format, as [<registry>/][<project>/]<image>[:<tag>|@<digest>]." with examples `image: redis:5` and `image: redis@sha256:0ed5d5928d47...`. — [Compose file reference: services](https://docs.docker.com/reference/compose-file/services/)
- NOT FOUND, a lockfile for images: none of the three pages above describes a file that records the digest a tag resolved to. The best-practices page mentions only looking up digests manually and a tool check that "pinned digests in your Dockerfile correspond to the correct version".

### Inferences

- For `image: redis:7.4` nothing committed holds the digest; it is "observed in a local cache" only if a local image store has the tag, or "resolved now" by asking the registry.
- A multi-architecture tag resolves to an index digest, and the per-platform manifest digest differs; the pages read do not discuss which one a pin should hold.

### Gaps

- `docker buildx imagetools`, BuildKit provenance attestations and `docker compose config --lock-image-digests` were not checked; whether any of them counts as a lock record is unverified.

## Identity formats: purl, SCIP, CycloneDX, SPDX

### Takeaway

purl is the identifier every other format in scope embeds: CycloneDX `component.purl`, SPDX 2.3 external reference type `purl`, SPDX 3 `packageUrl`, `maven-lockfile` v2 keys, and `pnpm add` arguments. SCIP uses its own `manager name version` triple.

### Cited Findings

- CONFIRMED, purl shape: "A PURL is a URL composed of seven components: scheme:type/namespace/name@version?qualifiers#subpath". `scheme`, `type` and `name` are Required; `namespace`, `version`, `qualifiers`, `subpath` are Optional. — [purl-spec Clause 5](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/specification/standard/Clause-5-Package-URL-Specification.md)
- CONFIRMED, standardisation: the old `PURL-SPECIFICATION.rst` now says its contents "have been moved to markdown files" and points to "the Package-URL Specification 1st Edition standard at: <https://ecma-international.org/publications-and-standards/standards/ecma-427/>"; the repository also carries "ECMA-427 2nd Edition Release notes". — [PURL-SPECIFICATION.rst](https://raw.githubusercontent.com/package-url/purl-spec/main/PURL-SPECIFICATION.rst); [2nd Edition release notes](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/specification/ECMA-427-2nd_Edition-Release_notes.md)
- CONFIRMED, recommended qualifiers: `checksum`, `download_url` ("A URL for a direct package download URL"), `file_name`, `repository_url`, `vcs_url` ("A URL for a version control system (VCS) location"), `vers` ("A VERS notation that specifies a version range instead of a single version"). — [purl-spec Annex B](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/specification/standard/Annex-B-Recommended-Qualifiers.md)
- CONFIRMED, qualifier ordering: "Tools that build PURLs should sort multiple qualifiers lexicographically by key". — [purl-spec Annex B](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/specification/standard/Annex-B-Recommended-Qualifiers.md)
- CONFIRMED, `pypi`: namespace Prohibited; "The name shall be lowercase." and "Replace runs of consecutive dash '-', underscore '_', or dot '.' characters with a single dash '-'". — [pypi type definition](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/types/definitions/pypi-definition.md)
- CONFIRMED, `npm`: namespace is the scope, case sensitive; "The npm scope @ sign prefix is always percent encoded"; example `pkg:npm/%40angular/animation@12.3.1`; and a git-pinned example `pkg:npm/mypackage@12.4.5?vcs_url=git:%2F%2Fhost.com%2F%2Fpath%2Fto%2Frepo.git%404345abcd34343`. — [npm type definition](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/types/definitions/npm-definition.md)
- CONFIRMED, `maven`: namespace Required, "The group id is the namespace."; qualifiers `classifier` and `type` (default `jar`); "Use the standard repository_url qualifier to point to another repository"; example `pkg:maven/groovy/groovy@1.0?repository_url=https:%2F%2Fmaven.google.com`. — [maven type definition](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/types/definitions/maven-definition.md)
- CONFIRMED, `golang`, with an internal inconsistency: namespace and name are marked "Case Sensitive: Yes" yet carry the notes "The namespace shall be lowercased." and "The name shall be lowercased."; "The version is often empty when a commit is not specified and should be the commit in most cases when available."; and "The current definition predates Go modules and has several practical problems". — [golang type definition](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/types/definitions/golang-definition.md)
- CONFIRMED, `cargo`: namespace Prohibited, name case sensitive, default repository `https://crates.io/`; example `pkg:cargo/rand@0.7.2`. — [cargo type definition](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/types/definitions/cargo-definition.md)
- CONFIRMED, `github`: "The namespace is the user or organization. It is not case sensitive and shall be lowercased."; "The version is a commit or tag."; example `pkg:github/package-url/purl-spec@244fd47e07d1004`. — [github type definition](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/types/definitions/github-definition.md)
- CONFIRMED, `docker`: "The version should be the image id sha256 or a tag. Since tags can be moved, a sha256 image id is preferred."; example `pkg:docker/customer/dockerimage@sha256%3A244fd47e07d10?repository_url=gcr.io`. — [docker type definition](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/types/definitions/docker-definition.md)
- CONFIRMED, `generic`: "this is handy for a plain version control repository such as a bare git repo in combination with a vcs_url"; qualifiers `download_url` and `checksum`; example `pkg:generic/bitwarderl?vcs_url=git%2Bhttps:%2F%2Fgit.fsfe.org%2Fdxtr%2Fbitwarderl%40cc55108da32`. — [generic type definition](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/types/definitions/generic-definition.md)
- New finding, a `git` type exists: `pkg:git/<namespace>/<name>@<version>`, "Git-based source packages"; "The version is a git reference ... Ideally a commit or tag."; example `pkg:git/codeberg.org/forgejo/forgejo@a72d2c07cfca03b55371089de6aa230d8c951fa0#options/locale_readme.md`. — [git type definition](https://raw.githubusercontent.com/package-url/purl-spec/main/docs/types/definitions/git-definition.md)
- CONFIRMED, SCIP package triple: `message Package { string manager = 1; string name = 2; string version = 3; }` with the grammar `<package> ::= <manager> ' ' <package-name> ' ' <version>` and "Use the placeholder '.' to indicate an empty value". — [scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)
- CONFIRMED, SCIP package meaning: "Unit of packaging and distribution. NOTE: This corresponds to a module in Go and JVM languages." — [scip.proto](https://raw.githubusercontent.com/sourcegraph/scip/main/scip.proto)
- CONFIRMED, CycloneDX `bom-ref`: "An optional identifier which can be used to reference the component elsewhere in the BOM. Every bom-ref must be unique within the BOM." — [CycloneDX 1.6 JSON schema](https://raw.githubusercontent.com/CycloneDX/specification/master/schema/bom-1.6.schema.json)
- CONFIRMED, CycloneDX `purl`: "Asserts the identity of the component using package-url (purl). The purl, if specified, must be valid and conform to the specification". Only `type` and `name` are required on a component; `hashes` is "The hashes of the component."; other identity fields are `cpe`, `swid`, `omniborId`, `swhid`. — [CycloneDX 1.6 JSON schema](https://raw.githubusercontent.com/CycloneDX/specification/master/schema/bom-1.6.schema.json). The 1.7 schema file exists in the same repository and has the same `purl`, `bom-ref` and `hashes` properties.
- CONFIRMED, CycloneDX dependency graph: a dependency's `ref` "References a component or service by its bom-ref attribute" and `dependsOn` holds "The bom-ref identifiers of the components or services that are dependencies of this dependency object." — [CycloneDX 1.6 JSON schema](https://raw.githubusercontent.com/CycloneDX/specification/master/schema/bom-1.6.schema.json)
- CONFIRMED, SPDX 2.3 package identity: fields include the package SPDX identifier (`SPDXID: SPDXRef-1`), `PackageDownloadLocation` with VCS forms such as `git+https://git.myproject.org/MyProject@da39a3ee5e6b4b0d3255bfef95601890afd80709#lib/variable.rb`, `PackageChecksum: SHA256: ...`, and external references of the form `<category> <type> <locator>` with category `PACKAGE-MANAGER`. — [SPDX 2.3 package information](https://raw.githubusercontent.com/spdx/spdx-spec/support/2.3/chapters/package-information.md)
- CONFIRMED, SPDX 2.3 purl reference: type `purl` with locator format `scheme:type/namespace/name@version?qualifiers#subpath`. — [SPDX 2.3 Annex F](https://raw.githubusercontent.com/spdx/spdx-spec/support/2.3/chapters/external-repository-identifiers.md)
- CONFIRMED, SPDX 3.0.1: `Package` has properties `downloadLocation`, `homePage`, `packageUrl`, `packageVersion`, `sourceInfo`; `packageUrl` "Provides a place for the SPDX data creator to record the package URL string"; external identifier types include `packageUrl`, `gitoid`, `swhid`. — [SPDX 3.0.1 Package](https://spdx.github.io/spdx-spec/v3.0.1/model/Software/Classes/Package/); [packageUrl](https://spdx.github.io/spdx-spec/v3.0.1/model/Software/Properties/packageUrl/); [ExternalIdentifierType](https://spdx.github.io/spdx-spec/v3.0.1/model/Core/Vocabularies/ExternalIdentifierType/)
- CONFIRMED, purl accepted by a package manager: "`pnpm add` accepts Package URLs for npm, Cargo, and Python packages" (added in v12.5.0). — [pnpm add docs (source)](https://raw.githubusercontent.com/pnpm/pnpm.io/main/docs/cli/add.md)

### Inferences

- "Most widely supported identifier" is not a statement any source makes. What the sources show is that purl is the one identifier present in CycloneDX, SPDX 2.3, SPDX 3, a Maven lockfile format and a package manager CLI, and that it is an Ecma standard (ECMA-427). No comparable cross-format presence was seen for the SCIP triple.
- A git-pinned dependency has three purl encodings in the spec: `pkg:github/owner/repo@<commit>` (GitHub only), `pkg:git/<host>/<path>@<commit>` (any host), or the ecosystem type plus `?vcs_url=git+https://...@<commit>` (keeps the package name). The spec does not rank them.
- The `golang` type is declared by its own definition to predate modules, so a Go module pin maps onto it imperfectly (pseudo-versions and `/vN` suffixes are not addressed).
- The SCIP triple and a purl carry the same three facts (type or manager, name, version); a purl additionally carries origin qualifiers. The mapping from one to the other is an inference, not something either source defines.

### Gaps

- Per-language conventions for the SCIP `manager` value (for example `npm`, `maven`, `gomod`) are not in `scip.proto`; indexer documentation was not read.
- Whether the `git` purl type is part of the published ECMA-427 edition or only of the repository's main branch was not established.
- Tool-support counts for purl versus other identifiers were not measured in any source read.

## Measured data from studies read

### Takeaway

One study gives lockfile commit rates per package manager (Go 99.7%, Cargo 70.9%, npm 53%, pnpm 36%, Gradle 0.9%); one small study gives SHA pinning of GitHub Actions (6.3% of 382 workflows). No measured rate was found for SNAPSHOT, range or git-reference declarations.

### Cited Findings

- Lockfile commit rates, "as of now" column of Table 6, among 4,859 GitHub projects with a dependency specification file: npm CLI 53% (1,021 of 1,916); pnpm 36% (688, same denominator); Cargo 70.9% (772 of 1,089); Poetry 83.8% (263 of 314); Pipenv 86.2% (25 of 29); Gradle 0.9% (3 of 323); Go 99.7% (1,184 of 1,188); total 81.4% (3,956). — [The Design Space of Lockfiles Across Package Managers, arXiv 2505.04834v3](https://arxiv.org/html/2505.04834v3)
- Same study, verbatim: "the Go ecosystem has the highest level of lockfile support, with 99.7% of projects committing the lockfile to the repository." and "the state of lockfile usage in Gradle is poor with only 0.9% projects committing their lockfile." — [arXiv 2505.04834v3](https://arxiv.org/html/2505.04834v3)
- Same study, sample: "we consider repositories created between September 30, 2019, and September 30, 2024, that have at least 42 stars, 10 contributors, and 300 commits." Counts are as of March 28, 2025. For Go the "lockfile" counted is `go.sum`. Maven is not in the study. — [arXiv 2505.04834v3](https://arxiv.org/html/2505.04834v3)
- GitHub Actions pinning: "commit-SHA pinning ... is used in only 6.3% of workflows globally. Java projects show the highest SHA adoption (15.79%), while Python projects show the lowest (2.70%)." Table 11 (percent of workflows with at least one action using the strategy, n=382): SHA 6.3, tag 93.7, branch 12.6, unpinned 3.7. — [An Empirical Study of Complexity, Heterogeneity, and Compliance of GitHub Actions Workflows, arXiv 2507.18062](https://arxiv.org/html/2507.18062)
- npm version constraints: "we ignore non-floating/pinning version constraints as they only occupy 0.83% of version constraints in our dataset" and "the popularity of floating only strategy increased from 2011 to 2018 but has fallen since then, replaced by more adoption of pinning only and mixed strategy". — [Pinning Is Futile, arXiv 2502.06662](https://arxiv.org/html/2502.06662)

### Inferences

- The 0.83% figure bounds how many npm constraints are neither a floating range nor an exact pin (a class that includes git URLs, tags and file paths), but the paper does not break that class down, so it is not a git-dependency rate.
- The Actions figure rests on 382 workflows in three languages; it is too small to generalise and counts workflows with at least one SHA-pinned action, not fully pinned workflows.

### Gaps

- NOT FOUND: any measured rate of `-SNAPSHOT` or version-range declarations in Maven or Gradle projects, of git-sourced dependencies in npm, Cargo or Go manifests, of committed `vendor/` directories, or of submodule use. No study on these was read.
- NOT FOUND: a large-scale, peer-reviewed SHA-pinning rate for GitHub Actions. Search results named industry figures (Datadog, Palo Alto Networks) and other papers (USI ICSA 2026, "Time for Actions" SecDev 2025); none of these was opened, so no number from them is recorded here.
- No adoption data exists yet for `actions.lock`, which became generally available on 2026-10-28.

## Summary table: sliding forms and where the exact pin is recorded

### Takeaway

"Committed" means readable offline from files in the repository; "cache" means readable offline only from a local tool cache; "resolve" means only by asking a remote now.

### Cited Findings

Each row restates findings cited in the sections above; no new claims.

| Ecosystem | Sliding forms | Where the exact pin is recorded, if anywhere | Readable offline from |
|---|---|---|---|
| Maven | `-SNAPSHOT`; ranges `[1.0,2.0)`; `LATEST`, `RELEASE`; transitive versions chosen by nearest-wins | No native lockfile (3.10.0, 4.0.0-rc-7). Timestamped snapshot in remote `maven-metadata.xml` and local `maven-metadata-<repo>.xml`. Opt-in third-party `lockfile.json` | Committed only if `maven-lockfile` is used; otherwise cache (`~/.m2/repository`) or resolve |
| Gradle | `1.+`, `latest.release`, ranges; changing modules (`-SNAPSHOT`, `changing = true`); catalog entries with ranges | `gradle.lockfile` (name and version per configuration, no hashes, no edges); not meant for changing modules. Dynamic result otherwise in the binary metadata cache | Committed when locking is on; otherwise resolve (the cache record is binary) |
| JitPack coordinates | `branch-SNAPSHOT`, `PR<n>-SNAPSHOT`, tags (mutable for 7 days) | The version string itself when it is a commit hash | Committed for commit-hash versions; otherwise resolve |
| npm | semver ranges, dist-tags, `#semver:`, branch refs, default branch | `package-lock.json` `resolved` (git URL with commit sha) and `integrity` | Committed |
| pnpm | same, plus `workspace:` ranges | `pnpm-lock.yaml` `resolution` (`type: git`, `repo`, `commit`) or a host tarball URL | Committed |
| Yarn Berry | same, plus `#head=`, `#tag=`, semver tags | `yarn.lock` resolution ending `#commit=<sha>` (from source code) | Committed |
| Yarn classic | same | `yarn.lock` `resolved "<url>#<commit>"` (from source code) | Committed |
| Bun | same | `bun.lock` package entry such as `github:owner/repo#6609a88` (abbreviated in the example read) | Committed |
| Go | `@latest`, branch queries at `go get` time only; `replace` to a local path | `go.mod` (versions and pseudo-versions with 12-character commit prefix), `go.sum` hashes, `vendor/modules.txt` | Committed; full commit hash for a pseudo-version needs cache or resolve |
| Cargo | caret ranges by default; git with branch, tag or no ref | `Cargo.lock` `source = "git+<url>?<ref>#<commit>"`, registry `checksum`; path packages have no `source` | Committed when `Cargo.lock` is checked in; otherwise resolve |
| Git submodule | `.gitmodules` `branch` (used only by `update --remote`) | Gitlink tree entry, mode 160000, holding the commit | Committed, even when the submodule directory is empty |
| Git subtree | upstream branch at merge time | `git-subtree-split` trailer in squash or rejoin commit messages; no file in the tree; upstream URL not recorded | Committed history only, when trailers exist |
| Vendored copy | n/a | Nothing, unless a tool writes a manifest (Go `vendor/modules.txt`) | Not available |
| GitHub Actions | `@tag`, `@branch`; `docker://image:tag` | `.github/workflows/actions.lock` (`ref`, `commit`, `owner_id`, `repo_id`) for onboarded workflows; or a 40-hex SHA in `uses:`. Reusable-workflow calls and `./` actions are not covered | Committed when locked or SHA-pinned; otherwise resolve |
| Container images | tags (`redis:7.4`), implicit `latest` | `@sha256:` digest written in `FROM` or `image:`; no lockfile found | Committed only when the digest is written; otherwise local image store or resolve |

### Inferences

- Three labels are enough for every row: declared in committed files (lockfile, manifest pin or gitlink), observed in a local cache (Maven local repository, Go module cache, local image store), resolved now. Gradle is the case where a cache record exists but is not readable as text.
- Maven and container images are the two ecosystems in scope with no committed pin by default; Gradle joins them in practice, given the 0.9% lockfile commit rate measured above.

### Gaps

- The table inherits the gaps listed per section: Gradle cache path layout, Yarn lockfile literals read from source code only, Bun commit abbreviation, coverage of `docker://` actions by `actions.lock`, and Docker-side lock mechanisms not checked.
