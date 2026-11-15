# Verified: which dependency and build commands execute repository-controlled code, and how tools gate it behind trust

State as of 2026-11-14. Every item is marked CONFIRMED (verbatim quote + URL), REFUTED, or NOT FOUND.

Method caveat. Pages were opened with a fetch tool that returns extracts through a small model; I asked for verbatim quotes only. Three sources were read as raw text and are exact: the Claude Code permissions page, the Claude Code security page, and git's `safe.adoc` / `core.adoc`. For the rest, quotes are verbatim as returned by the extractor; spot-check a quote against the URL before printing it in the specification. Long pages (go.dev/ref/mod, pkg.go.dev/cmd/go, git-config, code.claude.com sandboxing/mcp/permission-modes) were read in several offsets; where a part was not read, the gap is stated.

Pages that could not be opened: `blog.jetbrains.com` (HTTP 403 on three posts; web.archive.org is not reachable from this tool), `docs.swift.org` SwiftPM docs (JavaScript-only, empty body), `bundler.io/man/gemfile.5.html` (empty body; the RubyGems-hosted copy of the same manual was used instead).

## 1. Cargo

### Takeaway
The `cargo metadata` page says nothing about build scripts or proc macros; build scripts are documented as running "just before building the package", and `cargo metadata` does not build. The real exposure is indirect: a repository-supplied `.cargo/config.toml` is read from the current directory and all parents, and `rust-toolchain.toml` selects the toolchain that the `cargo` proxy runs (with auto-install when `RUSTUP_AUTO_INSTALL=1`).

### Cited Findings
- NOT FOUND (explicit statement): the `cargo metadata` page has no sentence on build scripts or proc macros. Its description is only: "Output JSON to stdout containing information about the workspace members and resolved dependencies of the current package." — [cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- CONFIRMED (build scripts run at build time): "Placing a file named `build.rs` in the root of a package will cause Cargo to compile that script and execute it just before building the package." — [Build Scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
- CONFIRMED `--no-deps`: "Output information only about the workspace members and don't fetch dependencies." — [cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- CONFIRMED `--locked`: "Asserts that the exact same dependencies and versions are used as when the existing `Cargo.lock` file was originally generated." Cargo exits with an error when "The lock file is missing." or "Cargo attempted to change the lock file due to a different dependency resolution." — [cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- CONFIRMED `--offline`: "Prevents Cargo from accessing the network for any reason." and "Cargo will restrict itself to crates that are downloaded locally". — [cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- CONFIRMED `--frozen`: "Equivalent to specifying both `--locked` and `--offline`." — [cargo metadata](https://doc.rust-lang.org/cargo/commands/cargo-metadata.html)
- CONFIRMED (repository config is read): "It looks for configuration files in the current directory and all parent directories." — [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- REFUTED (`[alias]` hijacking `cargo metadata`): "Aliases are not allowed to redefine existing built-in commands." An alias can only add new subcommand names. — [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- CONFIRMED (config can name programs to execute): "Some Cargo commands invoke external programs, which can be configured as a path and some number of arguments." — [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- CONFIRMED `build.rustc`: "Sets the executable to use for `rustc`." `build.rustc-wrapper`: "Sets a wrapper to execute instead of `rustc`." — [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- CONFIRMED (credential providers are a path plus arguments): for `registry.credential-provider`, "If specified as a string, path and arguments will be split on spaces." The page does not literally say "a process is executed". — [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- CONFIRMED `net.git-fetch-with-cli`: "If this is `true`, then Cargo will use the `git` executable to fetch registry indexes and git dependencies." — [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- CONFIRMED (config can redirect sources): `[source]` "If set, replace this source with the given named source or named registry."; `paths` "An array of paths to local packages which are to be used as overrides for dependencies." — [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html)
- CONFIRMED (`rust-toolchain.toml` selects the toolchain): "the toolchain can be named in the project's directory in a file called `rust-toolchain.toml`"; precedence: "The toolchain is chosen in the order listed above, using the first one that is specified." — [rustup overrides](https://rust-lang.github.io/rustup/overrides.html)
- CONFIRMED (custom toolchain path in the file): "The `path` setting allows a custom toolchain to be used. The value is an absolute path string." — [rustup overrides](https://rust-lang.github.io/rustup/overrides.html)
- CONFIRMED (auto-install is controlled by an environment variable): `RUSTUP_AUTO_INSTALL`: "When set to `1`, installs the active toolchain when it is absent." and "Set this value to `0` to disable automatic installation." — [rustup environment variables](https://rust-lang.github.io/rustup/environment-variables.html)
- CONFIRMED (`RUSTUP_TOOLCHAIN` outranks the file): "If set, will override the toolchain used for all rust tool invocations." — [rustup environment variables](https://rust-lang.github.io/rustup/environment-variables.html)

### Inferences
- `cargo metadata` does not compile, so `build.rs` and proc macros do not run; this follows from the build-script page ("just before building the package") plus the metadata description, not from an explicit sentence.
- Whether `cargo metadata` itself invokes `rustc` (and therefore a repository-configured `build.rustc` / `rustc-wrapper`) is not stated on either page. Treat it as a possible execution path until tested: safest is to run with a controlled `CARGO_HOME`, `RUSTUP_TOOLCHAIN` set explicitly, `RUSTUP_AUTO_INSTALL=0`, and `--config` overrides, or to parse `Cargo.toml` / `Cargo.lock` directly.
- Without `--no-deps` and without `--offline`, `cargo metadata` may download registry index and crates; with a repository `[source]` replacement or credential provider this contacts a repository-chosen host and may run a repository-chosen program.

### Gaps
- No primary sentence found that says "cargo metadata does not run build scripts" or that says whether it invokes `rustc` for target info.
- The rustup page does not state the default value of auto-install, nor whether `path` in `rust-toolchain.toml` is restricted for safety.

## 2. Go

### Takeaway
The `go` command's documented design keeps module download free of module-code execution, but two things are repository-controlled: the `go` / `toolchain` lines in `go.mod` can make the `go` command download and run a different Go toolchain (default `GOTOOLCHAIN=auto`), and direct-mode downloads run `git`/`hg`. `go list -export` produces "up-to-date export information", which means building.

### Cited Findings
- CONFIRMED (toolchain switch): "If no such program is found, the `go` command downloads and runs the specified Go toolchain." — [Go Toolchains](https://go.dev/doc/toolchain)
- CONFIRMED (triggered by the repository's `go.mod`): "When a command encounters a module requiring a newer Go version and `GOTOOLCHAIN` permits running different toolchains (it is one of the `auto` or `path` forms), the `go` command chooses and switches to an appropriate newer toolchain". — [Go Toolchains](https://go.dev/doc/toolchain)
- CONFIRMED (`toolchain` line): "The `toolchain` line declares a suggested toolchain to use with the module or workspace." — [Go Toolchains](https://go.dev/doc/toolchain)
- CONFIRMED (default is auto): "In standard Go toolchains, the `$GOROOT/go.env` file sets the default `GOTOOLCHAIN=auto`, but repackaged Go toolchains may change this value." — [Go Toolchains](https://go.dev/doc/toolchain)
- CONFIRMED (how to disable): "When `GOTOOLCHAIN` is set to `local`, the `go` command always runs the bundled Go toolchain." — [Go Toolchains](https://go.dev/doc/toolchain)
- CONFIRMED (downloads are verified): "Toolchains are downloaded like any other module, meaning that toolchain downloads can be proxied by setting `GOPROXY` and have their checksums checked by the Go checksum database." — [Go Toolchains](https://go.dev/doc/toolchain)
- CONFIRMED `go mod download`: "The `go mod download` command downloads the named modules into the module cache." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED `go list -m`: "The `-m` flag causes `go list` to list modules instead of packages." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED `-mod=mod`: "tells the `go` command to ignore the vendor directory and to automatically update `go.mod`". `-mod=readonly`: "tells the `go` command to ignore the `vendor` directory and to report an error if `go.mod` needs to be updated." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED `-mod=vendor`: "In this mode, the `go` command will not use the network or the module cache." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED `GOPROXY=off`: "`off` indicates that no communication should be attempted." Default: "`https://proxy.golang.org,direct`". — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED (direct mode runs VCS tools): "The `go` command runs version control tools like `git` when downloading modules directly." — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED (Go's own statement of the risk): "It is also a potential security problem if a malicious server finds a way to cause the invoked version control command to run unintended code." Default rule: "`public:git|hg,private:all`". — [Go Modules Reference](https://go.dev/ref/mod)
- CONFIRMED `GOFLAGS`: "A space-separated list of -flag=value settings to apply to go commands by default." — [cmd/go](https://pkg.go.dev/cmd/go)
- CONFIRMED `GOAUTH` runs commands: "Executes the given command (a space-separated argument list) and attaches the provided headers to HTTPS requests." — [cmd/go](https://pkg.go.dev/cmd/go)
- CONFIRMED `go list` flags: "-deps flag causes list to iterate over not just the named packages but also all their dependencies"; "-export flag causes list to set the Export field to the name of a file containing up-to-date export information"; "-compiled flag causes list to set CompiledGoFiles to the Go source files presented to the compiler." — [cmd/go](https://pkg.go.dev/cmd/go)
- CONFIRMED (`go generate` is never implicit): "Go generate is never run automatically by go build, go test, and so on." — [cmd/go](https://pkg.go.dev/cmd/go)
- CONFIRMED (cgo flag allow-list): "For security reasons, only a limited set of flags are allowed, notably -D, -U, -I, and -l." — [cmd/cgo](https://pkg.go.dev/cmd/cgo)
- CONFIRMED (cgo can invoke pkg-config): "CPPFLAGS and LDFLAGS may be obtained via the pkg-config tool using a '#cgo pkg-config:' directive". — [cmd/cgo](https://pkg.go.dev/cmd/cgo)
- CONFIRMED (checksum verification): "If the computed hash does not match, the `go` command reports a security error and does not install the file in the module cache." — [Go Modules Reference](https://go.dev/ref/mod)

### Inferences
- `go list -m all`, `go list -json -deps` and `go mod download` do not compile or run package code; this is the design implied by the docs (no build step is described for them), but I found no single sentence stating "does not execute module code".
- `go list -export` and `-compiled` require running the compiler and, for cgo packages, the cgo tool and the C compiler (`CC`: "The command to use to compile C code."). Avoid both flags for an untrusted repository; `CGO_ENABLED=0` removes the C toolchain path.
- A repository cannot set `GOFLAGS`, `GOTOOLCHAIN`, `GOPROXY` or `GOAUTH` through `go.mod`; they come from the environment or the user's `go env -w` file. The repository-controlled lever is the `go`/`toolchain` line. Running with `GOTOOLCHAIN=local`, `GOFLAGS=-mod=readonly` (or `-mod=mod` never), `GOPROXY=off` and `GOVCS=*:off` gives an offline, no-switch, no-VCS run.

### Gaps
- NOT FOUND: an explicit Go statement that module download never executes downloaded code.
- The last 15k characters of go.dev/ref/mod and the `-toolexec` text were not read.

## 3. Maven

### Takeaway
Any `mvn` invocation in a checkout reads the repository's `.mvn/` directory: `extensions.xml` loads core extensions (arbitrary JVM code, fetched by coordinates), `maven.config` injects command-line options, `jvm.config` injects JVM options. POM-declared build extensions and plugins add more. The wrapper downloads and runs whatever `distributionUrl` names.

### Cited Findings
- CONFIRMED (`.mvn` is repository content): "This directory is part of the project and may be checked into version control." The files "contain project specific configuration for running Maven." — [Configuring Maven](https://maven.apache.org/configure.html)
- CONFIRMED (`.mvn/extensions.xml` loads code by coordinates): "all transitive dependencies of those extensions will automatically being downloaded from your repository." — [Configuring Maven](https://maven.apache.org/configure.html)
- CONFIRMED (extensions add classes to Maven itself): "Extensions are a way to add classes to either the Core Classloader (Core Extensions) or the Project Classloader (Build Extensions)." and "There are core extensions which are loaded early and build extensions which are loaded late." — [Using Extensions](https://maven.apache.org/guides/mini/guide-using-extensions.html)
- CONFIRMED (POM-declared build extensions): "Registered as build extension in project->build->extensions->extension" and "Registered via project->build->plugins->plugin with element `extensions` being set to `true`." — [Using Extensions](https://maven.apache.org/guides/mini/guide-using-extensions.html)
- CONFIRMED (`.mvn/maven.config`): "defining `.mvn/maven.config` file which contains the configuration options for the `mvn` command line." — [Configuring Maven](https://maven.apache.org/configure.html)
- CONFIRMED (`.mvn/jvm.config`): "Starting with Maven 3.3.1+ you can define JVM configuration via `.mvn/jvm.config` file which means you can define the options for your build on a per project base." — [Configuring Maven](https://maven.apache.org/configure.html)
- CONFIRMED (wrapper downloads and runs a distribution): "if the user doesn't have the necessary version of Maven specified in `.mvn/wrapper/maven-wrapper.properties` it will be downloaded for the user first, installed and then used." — [Maven Wrapper](https://maven.apache.org/tools/wrapper/)
- CONFIRMED (`distributionUrl` is the lever): "you can change its version by setting the `distributionUrl` in `.mvn/wrapper/maven-wrapper.properties`". — [Maven Wrapper](https://maven.apache.org/tools/wrapper/)
- CONFIRMED (wrapper may compile and run repository Java): "The download is attempted via curl, wget and, as last resort, by compiling the `.mvn/wrapper/MavenWrapperDownloader.java` file and executing the resulting class." — [Maven Wrapper](https://maven.apache.org/tools/wrapper/)
- CONFIRMED (optional checksums live in the same repository file): "The property for validating the _maven-wrapper.jar_ file is named `wrapperSha256Sum` whereas the distribution file property is named `distributionSha256Sum`." — [Maven Wrapper](https://maven.apache.org/tools/wrapper/)
- CONFIRMED (`dependency:tree` needs the project): "Displays the dependency tree for this project." Attribute: "Requires a Maven project to be executed." — [dependency:tree](https://maven.apache.org/plugins/maven-dependency-plugin/tree-mojo.html)
- CONFIRMED (new in 3.10): "Starting with Maven 3.10.0, Maven also loads the extensions listed in `${user.home}/.m2/extensions.xml`". — [Configuring Maven](https://maven.apache.org/configure.html)

### Inferences
- `mvn dependency:tree` and `dependency:build-classpath` load the project model, so core extensions from `.mvn/extensions.xml` and POM build extensions run inside the Maven JVM before the goal does. `.mvn/jvm.config` can carry options such as `-javaagent:` and `.mvn/maven.config` can carry `-D`, `-s <settings>` or extra goals; the docs describe the files as generic option carriers and do not restrict their content.
- `mvnw` is a repository-supplied shell script, so running it is running repository code by definition; the checksum properties do not help because the same checkout supplies them.

### Gaps
- NOT FOUND: a Maven security page that states the trust model in one sentence.
- NOT FOUND on the goal page: whether `dependency:tree` forks a lifecycle phase (the page lists no such attribute, which suggests it does not).

## 4. Gradle

### Takeaway
Confirmed from Gradle's lifecycle docs: every build first runs init scripts, then evaluates the settings file, then evaluates the build files of every participating project; `buildSrc` is compiled automatically. The Tooling API executes builds and fails model fetch when a script fails to evaluate. The wrapper downloads and invokes the distribution in `distributionUrl`.

### Cited Findings
- CONFIRMED (initialization): "Gradle first runs any init scripts, then evaluates the settings file" and "Gradle detects the set of projects (root and subprojects) and included builds participating in the build." — [Build Lifecycle](https://docs.gradle.org/current/userguide/build_lifecycle.html)
- CONFIRMED (configuration): "Gradle evaluates the build files of every participating project". — [Build Lifecycle](https://docs.gradle.org/current/userguide/build_lifecycle.html)
- CONFIRMED (`buildSrc`): "Gradle automatically compiles the code in `buildSrc` and includes it in the classpath of all build scripts." and "Gradle automatically treats it as an included build". — [Sharing build logic](https://docs.gradle.org/current/userguide/sharing_build_logic_between_subprojects.html)
- CONFIRMED (init scripts): "Initialization scripts (*init scripts*) are Gradle scripts that run during the **initialization phase**, **before** the build's `settings.gradle(.kts)` and project build scripts are evaluated." — [Initialization Scripts](https://docs.gradle.org/current/userguide/init_scripts.html)
- REFUTED in part (init scripts from the checkout): the four documented locations are `-I`, `GRADLE_USER_HOME/init.gradle(.kts)`, `GRADLE_USER_HOME/init.d/` and `GRADLE_HOME/init.d/`; none is the project directory. A checkout supplies an init script only indirectly, through a wrapper distribution it names. — [Initialization Scripts](https://docs.gradle.org/current/userguide/init_scripts.html)
- CONFIRMED (wrapper): the wrapper "invokes a declared version of Gradle, downloading it beforehand if necessary"; `distributionUrl` is "The full URL pointing to the Gradle distribution ZIP file." — [Gradle Wrapper](https://docs.gradle.org/current/userguide/gradle_wrapper.html)
- CONFIRMED (Gradle's own warning): "The Wrapper JAR is a binary file that will be executed on the computers of developers and build servers." and "treat it as untrustworthy until proven otherwise". — [Gradle Wrapper](https://docs.gradle.org/current/userguide/gradle_wrapper.html)
- CONFIRMED (Tooling API runs builds): "This API allows you to execute and monitor builds and to query Gradle about the details of a build." and "The Tooling API can download and install the appropriate Gradle version, similar to the wrapper." — [Tooling API](https://docs.gradle.org/current/userguide/tooling_api.html)
- CONFIRMED (model fetch evaluates scripts): `getModel` fails "because the target project's configuration script fails to evaluate". — [Tooling API](https://docs.gradle.org/current/userguide/tooling_api.html)

### Inferences
- `gradle dependencies` is a task, so it runs in the execution phase after initialization and configuration; settings, build scripts, `buildSrc` and included builds are therefore all evaluated first. There is no Gradle mode that reads dependencies without evaluating repository scripts.

### Gaps
- NOT FOUND: the literal sentence "every invocation" in the lifecycle page (the page describes the phases of "a build" without that wording).
- Gradle's separate security page was not opened.

## 5. npm, pnpm, Yarn, Bun

### Takeaway
npm 12 has shipped (GA 2026-07-08, docs show 12.2.2 as Latest) and blocks dependency install scripts, git dependencies and remote-URL dependencies by default, with an `allowScripts` allow-list in `package.json`. The root project's own lifecycle scripts still run on a bare `npm install`, and npm 12 adds a new root-level code hook, `.npm-extension.mjs`. pnpm blocks dependency build scripts since 10.0; the setting is `allowBuilds` (10.26+, the only one in 11+), earlier `onlyBuiltDependencies`. Yarn's `yarnPath` executes a repository-named binary.

### Cited Findings
npm
- CONFIRMED (npm 12 is GA): "npm v12 is now generally available and tagged `latest`." (published July 8, 2026) — [GitHub Changelog](https://github.blog/changelog/2026-07-08-npm-install-time-security-and-gat-bypass2fa-deprecation/)
- CONFIRMED (new default): "`allowScripts` defaults to off"; dependency lifecycle scripts "(i.e., `preinstall`, `install`, `postinstall`)" "no longer run unless explicitly allowed". — [GitHub Changelog](https://github.blog/changelog/2026-07-08-npm-install-time-security-and-gat-bypass2fa-deprecation/)
- CONFIRMED (scope of the block, announced June 9, 2026): "`npm install` will no longer execute `preinstall`, `install`, or `postinstall` scripts from dependencies"; "`prepare` scripts from git, file, and link dependencies are blocked the same way." — [GitHub Changelog](https://github.blog/changelog/2026-06-09-upcoming-breaking-changes-for-npm-v12/)
- CONFIRMED (git and URL deps): "`--allow-git` defaults to `none`" and "`--allow-remote` defaults to `none`". — [GitHub Changelog](https://github.blog/changelog/2026-07-08-npm-install-time-security-and-gat-bypass2fa-deprecation/)
- CONFIRMED (docs): "Dependency install scripts are blocked by default." The command "Manages the `allowScripts` field in your project's `package.json`". — [npm approve-scripts (v12)](https://docs.npmjs.com/cli/v12/commands/npm-approve-scripts)
- CONFIRMED (config names, npm "Version 12.2.2 (Latest)"): `allow-scripts`, `strict-allow-scripts`, `allow-scripts-pending`, `dangerously-allow-all-scripts` ("bypass the `allowScripts` policy entirely and run every dependency install script"). — [npm config (v12)](https://docs.npmjs.com/cli/v12/using-npm/config)
- CONFIRMED (the allow-list can also come from `.npmrc`): "For team-wide policy in a project, use the `allowScripts` field in `package.json` (which also supports explicit denials), or configure it in `.npmrc`." — [npm config (v12)](https://docs.npmjs.com/cli/v12/using-npm/config)
- CONFIRMED (root scripts still run): "These run on a bare `npm install` in a local project (no package arguments)." followed by `preinstall`, `install`, `postinstall`, `prepublish`, `preprepare`, `prepare`, `postprepare`. — [npm scripts (v12)](https://docs.npmjs.com/cli/v12/using-npm/scripts)
- CONFIRMED (implicit node-gyp): "npm will default the `install` command to compile using node-gyp via `node-gyp rebuild`" when a `binding.gyp` exists and no install script is defined. — [npm scripts (v12)](https://docs.npmjs.com/cli/v12/using-npm/scripts)
- CONFIRMED `ignore-scripts`: "If true, npm does not run scripts specified in package.json files." Default false. — [npm config (v12)](https://docs.npmjs.com/cli/v12/using-npm/config)
- CONFIRMED (new root code hook in npm 12): `ignore-extension`: "If true, npm does not import or execute a root `.npm-extension.mjs` / `.npm-extension.cjs` file"; "`ignore-scripts` implies `ignore-extension`, since both disable root-owned install-time code." — [npm config (v12)](https://docs.npmjs.com/cli/v12/using-npm/config)
- CONFIRMED (`.npm-extension` runs before resolution; not under `npm ci`): it "lets a project imperatively repair the manifests of third-party dependencies"; "`npm ci` does **not** import or execute `.npm-extension`." — [.npm-extension (v12)](https://docs.npmjs.com/cli/v12/configuring-npm/npm-extension)
- CONFIRMED (project `.npmrc` is read): "per-project config file (`/path/to/my/project/.npmrc`)"; "this file is not read in global mode". The page lists no setting that is barred at project level. — [npmrc](https://docs.npmjs.com/cli/v11/configuring-npm/npmrc)
- CONFIRMED (security-relevant keys exist as ordinary config): `registry` "The base URL of the npm registry."; `script-shell` "The shell to use for scripts run with the `npm exec`, `npm run` and `npm init <package-spec>` commands."; `git` "The command to use for git commands."; `node-gyp` "This is the location of the \"node-gyp\" bin." — [npm config (v11.21.2, Legacy)](https://docs.npmjs.com/cli/v11/using-npm/config)
- CONFIRMED `npm ls`: "This command will print to stdout all the versions of packages that are installed"; with `--package-lock-only` "the output will be based on the tree described by the `package-lock.json`". The page mentions no script execution. — [npm ls (v12)](https://docs.npmjs.com/cli/v12/commands/npm-ls)
- CONFIRMED `package-lock-only`: "If set to true, the current operation will only use the `package-lock.json`, ignoring `node_modules`." — [npm config (v12)](https://docs.npmjs.com/cli/v12/using-npm/config)

pnpm
- CONFIRMED (10.0 changed the default): "Lifecycle scripts of dependencies are not executed during installation by default!" and "they should be listed in the `pnpm.onlyBuiltDependencies` field of `package.json`". — [pnpm v10.0.0 release](https://github.com/pnpm/pnpm/releases/tag/v10.0.0)
- CONFIRMED (pnpm 10 setting names): `onlyBuiltDependencies` ("A list of package names that are allowed to execute \"preinstall\", \"install\", and/or \"postinstall\" scripts during installation."), `onlyBuiltDependenciesFile`, `neverBuiltDependencies`, `ignoredBuiltDependencies` (10.1.0), `strictDepBuilds` (10.3.0, default false), `dangerouslyAllowAllBuilds` (10.9.0), `allowBuilds` (10.26.0). — [pnpm 10.x settings](https://pnpm.io/10.x/settings)
- CONFIRMED (pnpm 11 and later): "The following settings have been removed in v11 and replaced by `allowBuilds`"; `strictDepBuilds` now "Default: **true**". — [pnpm build settings](https://pnpm.io/settings/build)
- CONFIRMED (current statement): "pnpm does not run the build scripts of dependencies unless they are allowed in allowBuilds." — [pnpm supply chain security](https://pnpm.io/supply-chain-security)
- CONFIRMED `ignoreScripts`: "Do not execute any scripts defined in the project `package.json` and its dependencies." — [pnpm 10.x settings](https://pnpm.io/10.x/settings)
- CONFIRMED (pnpmfile is repository code that pnpm loads): "pnpm lets you hook directly into the installation process via special functions (hooks)."; "By default, `.pnpmfile.mjs` should be located in the same directory as the lockfile."; `ignorePnpmfile` is "Useful together with `--ignore-scripts` when you want to make sure that no script gets executed during install." — [pnpmfile](https://pnpm.io/pnpmfile)
- CONFIRMED (`pnpm list` can load the pnpmfile): `--find-by`: "Use a finder function defined in `.pnpmfile.mjs` to match dependencies by properties other than name."; `--lockfile-only`: "Read package information from the lockfile instead of checking the actual `node_modules` directory." — [pnpm list](https://pnpm.io/cli/list)

Yarn and Corepack
- CONFIRMED (`yarnPath` executes a repository-named binary): "This binary will be executed instead of any other (including the global one) for any command run within the directory covered by the rc file." and "If the file extension ends with `.js` it will be required, and will be spawned in any other case." — [.yarnrc.yml](https://yarnpkg.com/configuration/yarnrc)
- CONFIRMED (opt-out): `ignorePath`: "If true, whatever Yarn version is being executed will keep running rather than looking at the value of `yarnPath` to decide." — [.yarnrc.yml](https://yarnpkg.com/configuration/yarnrc)
- CONFIRMED (current Yarn default for dependency scripts): "If false (the default), Yarn will not execute the `postinstall` scripts from third-party packages when installing the project"; workspaces "will still see their postinstall scripts evaluated". — [.yarnrc.yml](https://yarnpkg.com/configuration/yarnrc)
- CONFIRMED (Yarn lifecycle): "the `preinstall` and `install` scripts, if presents, are called right before running the `postinstall` script"; for git sources Yarn "will automatically run `yarn install` and `yarn pack`". — [Yarn lifecycle scripts](https://yarnpkg.com/advanced/lifecycle-scripts)
- CONFIRMED (other repository-controlled Yarn hooks): `injectEnvironmentFiles` "Array of .env files which will get injected into any subprocess spawned by Yarn."; "JavaScript constraints will always be read from the `yarn.config.cjs` file." — [.yarnrc.yml](https://yarnpkg.com/configuration/yarnrc)
- CONFIRMED (Corepack `packageManager`): "`packageManager@x.y.z` is required. The hash is optional but strongly recommended as a security practice."; "Corepack will download and cache the latest compatible version." — [Corepack README](https://github.com/nodejs/corepack/blob/main/README.md)
- CONFIRMED (Corepack URLs): "You can also provide a URL to a `.js` file (which will be interpreted as a CommonJS module) or a `.tgz` file."; `COREPACK_ENABLE_UNSAFE_CUSTOM_URLS` "can be set to `1` to allow use of custom URLs to load a package manager known by Corepack." — [Corepack README](https://github.com/nodejs/corepack/blob/main/README.md)
- CONFIRMED (Corepack off-switches): `COREPACK_ENABLE_PROJECT_SPEC` "can be set to `0` to prevent Corepack from checking if the package manager corresponds to the one defined for the current project."; `COREPACK_ENABLE_NETWORK` "can be set to `0` to prevent Corepack from accessing the network." — [Corepack README](https://github.com/nodejs/corepack/blob/main/README.md)

Bun
- CONFIRMED: "Because running arbitrary code is a security risk, Bun does not execute arbitrary lifecycle scripts by default, unlike other `npm` clients." — [Bun lifecycle scripts](https://bun.sh/docs/pm/lifecycle)
- CONFIRMED (allow-list is repository-controlled): "To allow lifecycle scripts for a particular package, add its name to the `trustedDependencies` array in your `package.json`."; "A curated list of popular npm packages with lifecycle scripts is allowed by default." — [Bun lifecycle scripts](https://bun.sh/docs/pm/lifecycle)
- CONFIRMED: "To disable lifecycle scripts for all packages, use the `--ignore-scripts` flag." — [Bun lifecycle scripts](https://bun.sh/docs/pm/lifecycle)

### Inferences
- The allow-lists in npm 12 (`allowScripts` in `package.json`), pnpm (`allowBuilds`) and Bun (`trustedDependencies`) are all stored in the repository. For a developer's own project that is the trust decision; for `mentalc` analysing an arbitrary checkout it is repository-controlled input and must not count as the user's trust decision. Likewise the root package's own lifecycle scripts run on every bare install in all four managers unless `--ignore-scripts` is passed.
- For npm, `--ignore-scripts` now also disables `.npm-extension`; a "no execution" invocation of npm 12 is `npm install --package-lock-only --ignore-scripts` or `npm ci --ignore-scripts`, plus a neutral `--userconfig`/registry if the project `.npmrc` must not redirect traffic.
- `npm ls`, `npm ls --package-lock-only` and `pnpm list --lockfile-only` are read-only according to their pages (no script mentioned). `pnpm list --find-by` loads `.pnpmfile`; whether plain `pnpm list` loads it is not stated.
- Running `yarn` in a checkout with `yarnPath` executes a file from the repository unless `ignorePath` is set from outside the repository (for example `YARN_IGNORE_PATH=1`).

### Gaps
- The npm 12 scripts page does not restate the dependency-script block; the block is documented on the approve-scripts and config pages and in the changelog.
- NOT FOUND: whether `npm install --package-lock-only` runs root lifecycle scripts (not stated; pass `--ignore-scripts`).
- NOT FOUND: Bun's handling of the root package's own scripts on `bun install` (page is silent).
- The Yarn version in which `enableScripts` became `false` by default was not determined; the current configuration page states it as the default.
- Corepack's distribution status in recent Node.js releases was not verified.

## 6. Python

### Takeaway
Confirmed: sdists need the build backend to run; PEP 517 hooks are called from `pyproject.toml`-named backends (which can live in-tree). `.pth` `import` lines are still executed at startup in Python 3.15 but are now deprecated. uv states that build backends "may run arbitrary Python code"; Poetry auto-installs project-required plugins.

### Cited Findings
- CONFIRMED (`.pth`): "For backward compatibility, lines starting with `import` (followed by space or tab) are executed with `exec()`." (Python 3.15.2 docs) — [site module](https://docs.python.org/3/library/site.html)
- CONFIRMED (every startup): "Executable lines (`import` lines in `_name_.pth` files and `_name_.start` file entry points) are always run at Python startup (unless `-S` is given to disable the `site.py` module entirely)". — [site module](https://docs.python.org/3/library/site.html)
- CONFIRMED (new, deprecation): "Changed in version 3.15: `.pth` file lines starting with `import` are deprecated." and they "will be silently ignored in Python 3.18 and 3.19." — [site module](https://docs.python.org/3/library/site.html)
- CONFIRMED (sdist needs the backend): "installers like pip cannot install from sdists directly"; "they need to first build a wheel, by invoking the build backend that the sdist specifies". — [PyPA package formats](https://packaging.python.org/en/latest/discussions/package-formats/)
- CONFIRMED (PEP 517): "Frontends should call each hook in a fresh subprocess"; "Projects can specify that their backend code is hosted in-tree by including the `backend-path` key in `pyproject.toml`." — [PEP 517](https://peps.python.org/pep-0517/)
- CONFIRMED (uv): `no-build`: "Don't build source distributions."; "uv will also still build editable requirements, and their build backends may run arbitrary Python code." — [uv settings](https://docs.astral.sh/uv/reference/settings/)
- CONFIRMED (uv installs the project): "uv will install the project (and other workspace members) as _editable_ packages"; "`--no-install-project`: Do not install the current project". — [uv locking and syncing](https://docs.astral.sh/uv/concepts/projects/sync/)
- CONFIRMED (uv lock flags): `--locked`: "If the lockfile is not up-to-date, uv will raise an error instead of updating the lockfile."; `--frozen`: "To use the lockfile without checking if it is up-to-date". — [uv locking and syncing](https://docs.astral.sh/uv/concepts/projects/sync/)
- CONFIRMED (uv may download interpreters): `python-downloads` default `"automatic"`: "Automatically download managed Python installations when needed". — [uv settings](https://docs.astral.sh/uv/reference/settings/)
- CONFIRMED (Poetry project plugins): "If the plugin is not installed in Poetry's own environment when running `poetry install`" "it will be installed only for the current project under `.poetry/plugins`"; "Installed plugin packages are automatically loaded when Poetry starts up." — [Poetry plugins](https://python-poetry.org/docs/plugins/)

### Inferences
- `uv sync` in a checkout with a `[build-system]` runs the repository's build backend (editable install), even with `no-build`; `--no-install-project` plus `--frozen` avoids that. `[tool.uv]` in `pyproject.toml` can change indexes, which redirects network traffic to repository-chosen hosts.
- A Poetry project can make `poetry install` fetch and load a plugin named in `[tool.poetry.requires-plugins]`; that is repository-controlled code execution inside Poetry.

### Gaps
- NOT FOUND: a PyPA sentence that literally says "setup.py executes arbitrary code"; the setup.py-deprecation page and the pip download page do not say it.
- The last quarter of the uv settings page was not read.

## 7. Ruby, PHP, Swift

### Takeaway
Gemfile-as-Ruby and Composer scripts/plugins are confirmed from official docs. Composer has a real trust gate for plugins (`allow-plugins`, since 2.2.0). For SwiftPM the manifest-execution sentence could not be read from official docs; only the sandbox statement in SE-0303 was confirmed.

### Cited Findings
- CONFIRMED (Bundler): "A `Gemfile` is evaluated as Ruby code". — [gemfile(5) on RubyGems Guides](https://guides.rubygems.org/gemfile/)
- CONFIRMED (Bundler includes other files): "The `eval_gemfile` method reads the contents of another file as if they appeared in the current Gemfile." — [gemfile(5)](https://guides.rubygems.org/gemfile/)
- CONFIRMED (Bundler's own warning): "These protocols are unauthenticated, so a man-in-the-middle attacker can deliver malicious code and compromise your system." (about `http://` and `git://`) — [gemfile(5)](https://guides.rubygems.org/gemfile/)
- CONFIRMED (Composer scripts are root-only): "Only scripts defined in the root package's `composer.json` are executed." — [Composer scripts](https://getcomposer.org/doc/articles/scripts.md)
- CONFIRMED (Composer script events on install): "pre-install-cmd: occurs before the install command is executed with a lock file present."; "post-autoload-dump: occurs after the autoloader has been dumped, either during install/update, or via the dump-autoload command." — [Composer scripts](https://getcomposer.org/doc/articles/scripts.md)
- CONFIRMED (Composer plugins): "Plugin packages are automatically loaded as soon as they are installed"; "You may pass the --no-plugins option to Composer commands to disable all installed plugins." — [Composer plugins](https://getcomposer.org/doc/articles/plugins.md)
- CONFIRMED (Composer trust gate): "As of Composer 2.2.0, the `allow-plugins` option adds a layer of security"; "Defaults to `{}` which does not allow any plugins to be loaded."; "If you run Composer interactively it will prompt you to decide if you want to execute the plugin or not." — [Composer config](https://getcomposer.org/doc/06-config.md)
- CONFIRMED (SwiftPM manifest parsing is sandboxed): "Package plugins will be run in a sandbox that prevents network access"; "This is the same sandbox that manifest parsing uses, except that it allows writing to a limited set of output directories." — [SE-0303](https://github.com/swiftlang/swift-evolution/blob/main/proposals/0303-swiftpm-extensible-build-tools.md)
- NOT FOUND (SwiftPM "Package.swift is compiled and run"): docs.swift.org returned an empty body; the older PackageDescription page shows the manifest as Swift source (`import PackageDescription`, `let package = Package(...)`) but has no sentence about compilation or execution. — [PackageDescription](https://docs.swift.org/package-manager/PackageDescription/PackageDescription.html)

### Inferences
- Composer's `allow-plugins` lives in `composer.json`, so like the JavaScript allow-lists it is repository-controlled when analysing someone else's checkout; `--no-plugins` and `--no-scripts` are the external switches (the `--no-scripts` flag was not on the scripts page I read).
- The existence of a sandbox for "manifest parsing" in SE-0303 implies the manifest is executed, but that is an inference.

### Gaps
- The Bundler `plugin` directive and `gemspec` evaluation text were not found on the page read.
- SwiftPM: no directly quotable official sentence; the JetBrains AppCode post that describes it returned 403.

## 8. Docker Compose

### Takeaway
`docker compose config` is documented only as "Parse, resolve and render compose file in canonical format"; no execution is documented. It does read repository-controlled files and interpolates host shell variables into its output, so it can leak host environment values, and `include` can reference remote sources.

### Cited Findings
- CONFIRMED (what it does): "Parse, resolve and render compose file in canonical format". — [docker compose config](https://docs.docker.com/reference/cli/docker/compose/config/)
- CONFIRMED (opt-outs exist, so the defaults interpolate and read env files): `--no-interpolate` "Don't interpolate environment variables"; `--no-env-resolution` "Don't resolve service env files"; `--no-path-resolution` "Don't resolve file paths". — [docker compose config](https://docs.docker.com/reference/cli/docker/compose/config/)
- CONFIRMED (network-touching options are opt-in): `--resolve-image-digests` "Pin image tags to digests"; `--lock-image-digests` "Produces an override file with image digests". — [docker compose config](https://docs.docker.com/reference/cli/docker/compose/config/)
- CONFIRMED (host environment is an interpolation source, first in precedence): "Variables from your shell environment", then "Variables from a file set by `--env-file`", then "variables from an `.env` file in the project directory". — [Compose interpolation](https://docs.docker.com/compose/how-tos/environment-variables/variable-interpolation/)
- CONFIRMED (`config` prints resolved values): `docker compose config` "prints your resolved application config to the terminal". — [Compose interpolation](https://docs.docker.com/compose/how-tos/environment-variables/variable-interpolation/)
- CONFIRMED (`include`): "Each path listed in the `include` section is loaded as an individual Compose application model"; "`env_file` defines an environment file(s) to use to define default values when interpolating variables". — [Compose file: include](https://docs.docker.com/reference/compose-file/include/)
- CONFIRMED (remote include): "`include` allows you to reference Compose files from remote sources, such as OCI artifacts or Git repositories." — [Compose include how-to](https://docs.docker.com/compose/how-tos/multiple-compose-files/include/)

### Inferences
- A compose file containing `${AWS_SECRET_ACCESS_KEY}` gets the host value substituted into `docker compose config` output. `mentalc` should either parse the YAML itself or run with `--no-interpolate --no-env-resolution` and a scrubbed environment.
- With a remote `include`, `docker compose config` must fetch from a repository-chosen registry or Git URL (network, and a `git` subprocess for Git sources); the docs opened do not describe a prompt.

### Gaps
- NOT FOUND: an explicit statement that `docker compose config` never executes anything, and how remote includes behave under `config`. `extends` was not read.

## 9. Git

### Takeaway
Git's own manual states the model precisely: cloning untrusted content is generally safe, but running Git inside an untrusted `.git` directory is not, because config and hooks can run arbitrary commands. `safe.directory` (added April 2022 for CVE-2022-24765) only covers repositories owned by another user.

### Cited Findings
- CONFIRMED: "Some configuration options and hook files may cause Git to run arbitrary shell commands. Because configuration and hooks are not copied using `git clone`, it is generally safe to clone remote repositories with untrusted content". — [git(1) SECURITY](https://git-scm.com/docs/git)
- CONFIRMED: "it is not safe to run Git commands in a `.git` directory (or the working tree that surrounds it) when that `.git` directory itself comes from an untrusted source." — [git(1) SECURITY](https://git-scm.com/docs/git)
- CONFIRMED (limit of the ownership check): "you can also acquire untrusted repositories that are owned by you (for example, if you extract a zip file or tarball from an untrusted source)." — [git(1) SECURITY](https://git-scm.com/docs/git)
- CONFIRMED (`safe.directory`, exact source text): "By default, Git will refuse to even parse a Git config of a repository owned by someone else, let alone run its hooks, and this config setting allows users to specify exceptions". — [safe.adoc](https://raw.githubusercontent.com/git/git/master/Documentation/config/safe.adoc), rendered at [git-config](https://git-scm.com/docs/git-config)
- CONFIRMED (cannot be set by the repository): "This config setting is only respected in protected configuration (see <<SCOPES>>). This prevents untrusted repositories from tampering with this value." — [safe.adoc](https://raw.githubusercontent.com/git/git/master/Documentation/config/safe.adoc)
- CONFIRMED (opt-out): "To completely opt-out of this security check, set `safe.directory` to the string `*`." — [safe.adoc](https://raw.githubusercontent.com/git/git/master/Documentation/config/safe.adoc)
- CONFIRMED (protected configuration): "Protected configuration refers to the _system_, _global_, and _command_ scopes. For security reasons, certain options are only respected when they are specified in protected configuration, and ignored otherwise." — [git-config](https://git-scm.com/docs/git-config)
- CONFIRMED (`core.fsmonitor` is a command): "Otherwise, this variable contains the pathname of the \"fsmonitor\" hook command." — [core.adoc](https://raw.githubusercontent.com/git/git/master/Documentation/config/core.adoc)
- CONFIRMED (`safe.bareRepository`): `explicit` "will protect you from attacks that involve cloning a repository that contains a bare repository and running a Git command within that directory"; "This will be the default in Git 3.0." — [safe.adoc](https://raw.githubusercontent.com/git/git/master/Documentation/config/safe.adoc)
- CONFIRMED (2022 origin; published April 12, 2022): "Since some configuration variables (such as core.fsmonitor) cause Git to execute arbitrary commands" "this can lead to arbitrary command execution when working on a shared machine."; fix: "upgrade to Git v2.35.2". — [GitHub blog, CVE-2022-24765](https://github.blog/open-source/git/git-security-vulnerability-announced/)
- CONFIRMED (`git submodule update` clones and fetches): "Update the registered submodules to match what the superproject expects by cloning missing submodules, fetching missing commits in submodules". — [git-submodule](https://git-scm.com/docs/git-submodule)
- CONFIRMED (URLs come from the repository): "a `.gitmodules` file in the top-level directory of the containing repository is used to find the URL of each submodule." — [git-submodule](https://git-scm.com/docs/git-submodule)
- CONFIRMED (custom update command is not accepted from `.gitmodules`): "Note that this mechanism is not supported in the `.gitmodules` file or on the command line." — [git-submodule](https://git-scm.com/docs/git-submodule)

### Inferences
- A fresh `git clone` over the network carries no hooks or config, so reading it with Git is safe by Git's own statement. A checkout that arrived as an archive, a copied directory, or one containing an embedded bare repository can carry `core.fsmonitor`, `core.hooksPath`, `core.sshCommand` and similar; `git status` there can execute them. `mentalc` should not shell out to `git` in a repository of unknown provenance, or should neutralise config (`-c core.fsmonitor=false -c core.hooksPath=/dev/null`, `safe.bareRepository=explicit`).
- `git submodule update --init` contacts repository-chosen URLs; it is a network action under repository control and belongs behind the trust decision.

### Gaps
- `protocol.file.allow` and the full list of command-running config keys were not read (the git-config page is 450k characters; only the first part and the source files for `safe.*` and `core.*` were read).

## 10. Trust gates in IDEs

### Takeaway
IntelliJ's Safe Mode disables build-tool import, startup scripts, VCS support, Groovy DSL scripts, File Watchers and database connections. VS Code Restricted Mode disables or limits AI agents, terminal, tasks, debugging, workspace settings and extensions. The JetBrains "command line carries the same risk" sentence could not be opened at source.

### Cited Findings
- CONFIRMED (IntelliJ options): "Preview in Safe Mode", "Trust Project", "Don't Open". Trust Project "opens and initializes the project, resolves project plugins, adds dependencies, and enables all IntelliJ IDEA features." — [IntelliJ project security](https://www.jetbrains.com/help/idea/project-security.html)
- CONFIRMED (Safe Mode, build tools): "for a project that contains any build tool, the importing process is disabled."; "no build scripts are executed and no dependencies are resolved." — [IntelliJ project security](https://www.jetbrains.com/help/idea/project-security.html)
- CONFIRMED (Safe Mode, other): "any scripts or tasks that are executed during the opening process are disabled."; "VCS support is fully disabled."; "any Groovy DSL scripts will not be executed."; "Database connections are disabled." — [IntelliJ project security](https://www.jetbrains.com/help/idea/project-security.html)
- CONFIRMED (headless is trusted by default): "When you launch an instance of IntelliJ IDEA in the headless mode using a command-line launcher, the project opens in the trusted mode by default." — [IntelliJ project security](https://www.jetbrains.com/help/idea/project-security.html)
- NOT FOUND at source (JetBrains "same risk" statement): the help page has no such sentence. The blog post <https://blog.jetbrains.com/idea/2021/03/intellij-idea-2020-3-3/> returned HTTP 403. A search-result summary attributes to it: "building or running a Maven or Gradle project from the command line carries the same security risks as importing it into an IDE". This is a snippet, not a read page; verify by hand before quoting.
- CONFIRMED (VS Code purpose): Workspace Trust "lets you decide whether code in your project folder can be executed by VS Code and extensions without your explicit approval." — [VS Code Workspace Trust](https://code.visualstudio.com/docs/editing/workspaces/workspace-trust)
- CONFIRMED (VS Code Restricted Mode): it "tries to prevent automatic code execution by disabling or limiting the operation of several VS Code features:" "AI agents, terminal, tasks, debugging, workspace settings, and extensions." — [VS Code Workspace Trust](https://code.visualstudio.com/docs/editing/workspaces/workspace-trust)
- CONFIRMED (VS Code details): "Opening a workspace in restricted mode disables agents in that workspace."; "opening a terminal is blocked by default when a folder is open in Restricted Mode." — [VS Code Workspace Trust](https://code.visualstudio.com/docs/editing/workspaces/workspace-trust)

### Inferences
- Both IDEs treat "resolve dependencies through the build tool" as the action that needs trust, and both keep browsing and editing available without it. That matches the proposed `mentalc` rule: static reading by default, build-tool invocation behind a trust decision.

### Gaps
- The JetBrains blog posts (IntelliJ 2020.3.3, RubyMine and AppCode "Trusted Projects") could not be opened.

## 11. Coding agents as the trust gate

### Takeaway
In Claude Code, permission rules and prompts apply to the tool calls Claude makes (the Bash command text Claude writes, or an MCP tool call), not to processes those tools spawn. The Bash sandbox does cover child processes of a Bash command, but local MCP servers run outside it "with your full access". So commands that `mentalc` spawns are never prompted for individually; under Bash they inherit the sandbox if one is on, under MCP they inherit nothing.

### Cited Findings
Claude Code
- CONFIRMED (what permissions cover): "**Permissions** control which tools Claude Code can use and which files or domains it can access. They apply to Bash, Read, Edit, WebFetch, MCP, and every other tool". — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (Bash prompts): Bash commands: approval required "Yes, except a built-in set of read-only commands". — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (rule order): "Rules are evaluated in order: deny, then ask, then allow. The first match in that order determines the outcome". — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (Bash rules match text Claude writes, not behaviour): "A Bash rule matches the command text Claude writes"; "a deny or ask rule covers the invocation Claude usually produces and isn't a security boundary around the program." — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (runners execute their arguments): "Because these tools execute their arguments as a command, a rule like `Bash(devbox run *)` matches whatever comes after `run`". — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (permission modes): `default` "Prompts for permission on first use of each tool"; `acceptEdits`; `plan`; `auto` "a background classifier checks that they align with your request"; `dontAsk` "Auto-denies every call that would otherwise prompt"; `bypassPermissions` "Skips permission prompts". — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (starting mode today): "Auto mode is the built-in starting permission mode for interactive terminal and VS Code sessions." — [Claude Code security](https://code.claude.com/docs/en/security)
- CONFIRMED (MCP rules name tools only): "To match every tool from an MCP server, write `mcp__<server>__*`. To match one tool, write its full name, `mcp__<server>__<tool>`."; an `mcp__` rule with parentheses is skipped when settings load. — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (sandbox covers child processes of shell commands): "**Sandboxing** provides OS-level enforcement that restricts shell commands' filesystem and network access. It applies to Bash, PowerShell, and Monitor tool commands and their child processes." — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (MCP servers are outside the sandbox): "**Other processes Claude Code starts**: command hooks, local MCP servers, plugin monitors, LSP servers, and helper commands such as your status line command and `apiKeyHelper` run with your full access". — [Claude Code sandboxing](https://code.claude.com/docs/en/sandboxing)
- CONFIRMED (sandbox mechanism and defaults): "**macOS**: uses Seatbelt for sandbox enforcement"; "**Linux**: uses bubblewrap for isolation"; network: "No direct route out. Connections go through a proxy on your machine that checks each host against your allowed domains, which start empty." — [Claude Code sandboxing](https://code.claude.com/docs/en/sandboxing)
- CONFIRMED (default reads are broad): "**Default read behavior**: read access to the entire computer, except certain denied directories. This default still allows reading credential files". — [Claude Code sandboxing](https://code.claude.com/docs/en/sandboxing)
- CONFIRMED (auto-allow removes the prompt): "Claude Code approves a command automatically, with no prompt, when the command runs inside the sandbox." — [Claude Code sandboxing](https://code.claude.com/docs/en/sandboxing)
- CONFIRMED (escape hatch): Claude "may retry the command with the `dangerouslyDisableSandbox` parameter"; "You can disable the unsandboxed retry by setting `\"allowUnsandboxedCommands\": false`". — [Claude Code sandboxing](https://code.claude.com/docs/en/sandboxing)
- CONFIRMED (not a hard boundary): "Sandboxing reduces risk but is not a complete isolation boundary." — [Claude Code sandboxing](https://code.claude.com/docs/en/sandboxing)
- CONFIRMED (an approved Bash command is unconstrained without the sandbox): "The boundary is a permission prompt, so a Bash command you approve can still write anywhere your user account can". — [Claude Code security](https://code.claude.com/docs/en/security)
- CONFIRMED (stdio MCP servers): "Stdio servers run as local processes on your machine." — [Claude Code MCP](https://code.claude.com/docs/en/mcp)
- CONFIRMED (project `.mcp.json` gate, and its hole): "Claude Code prompts for approval in interactive sessions before using project-scoped servers from `.mcp.json` files."; in `claude -p`, SDK and some cloud sessions "it loads project-scoped servers without asking." — [Claude Code MCP](https://code.claude.com/docs/en/mcp)
- CONFIRMED (workspace trust gates repository-supplied allow rules): "`permissions.allow` rules and `permissions.additionalDirectories` entries in a project's `.claude/settings.json` grant capability, so Claude Code applies them only after you accept the workspace trust dialog". — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (no trust dialog headless): "Claude Code shows the trust dialog in interactive terminal sessions, but not in `claude -p` or SDK sessions." — [Claude Code permissions](https://code.claude.com/docs/en/permissions)
- CONFIRMED (MCP trust is the user's job): Anthropic "does not security-audit or manage any MCP server." — [Claude Code security](https://code.claude.com/docs/en/security)
- CONFIRMED (per-call consent exists for MCP tools): for a tool marked `requiresUserInteraction`, "Claude Code shows that tool's permission prompt on every call, even in `acceptEdits`, `auto`, and `bypassPermissions` permission modes". — [Claude Code MCP](https://code.claude.com/docs/en/mcp)
- CONFIRMED (auto-mode classifier sees tool calls, not results): "the classifier sees user messages, tool calls other than read-only lookups such as file reads and searches, and your CLAUDE.md content. Tool results are stripped from those requests". — [Claude Code permission modes](https://code.claude.com/docs/en/permission-modes)

OpenAI Codex
- Note: `developers.openai.com/codex/security`, `/codex/sandbox` and `/codex/agent-approvals-security` now answer with HTTP 308 to `learn.chatgpt.com/docs/...`; the openai/codex repository's `docs/sandbox.md` still links to the old `developers.openai.com/codex/security` address. Quotes below are from the redirect target.
- CONFIRMED (two controls): sandbox mode is "What Codex can do technically (for example, where it can write and whether it can reach the network)."; approval policy is "When Codex must ask you before it executes an action." — [Codex: Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)
- CONFIRMED (modes): read-only: "can read files and run commands within the read-only sandbox."; workspace-write: "can read files, make edits, and run commands in the working directory automatically."; danger-full-access: "No sandbox; no approvals _(not recommended)_." — [Codex: Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)
- CONFIRMED (defaults): version-controlled folders get "`Auto` (workspace write + on-request approvals)"; "By default, the agent runs with network access turned off." — [Codex: Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)
- CONFIRMED (mechanism): "OS-level mechanisms enforce sandbox policies."; macOS "uses Seatbelt policies and runs commands using `sandbox-exec`."; Linux uses `bwrap` plus `seccomp`. — [Codex: Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)
- CONFIRMED (children of sandboxed commands are filtered): the network proxy "filters scripts, programs, and child processes that run inside the local command sandbox." — [Codex: Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)
- REFUTED (approval policy list): the page says Codex "no longer support[s] `approval_policy = \"untrusted\"`"; the remaining policies named are `on-request` ("commands allowed by the sandbox can run without approval.") and `never`. — [Codex: Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)
- CONFIRMED (project trust): Codex "may also start in `read-only` until you explicitly trust the working directory."; a project `trust_level = "untrusted"` "also disables project-local configuration." — [Codex: Agent approvals & security](https://learn.chatgpt.com/docs/agent-approvals-security)
- NOT FOUND (Codex MCP servers and the sandbox): the page does not say whether MCP server processes run inside the sandbox; it says only that the command network proxy does not filter "MCP server connections". The Codex MCP page address I tried returned 404.

### Inferences
- Answer to the design question, for Claude Code. (a) `mentalc` run through the Bash tool: the user or classifier approves the `mentalc ...` command line once; anything `mentalc` spawns (`cargo`, `go`, `mvn`, `npm`) gets no prompt of its own. If the Bash sandbox is enabled, those children are inside it (no network unless a host is allowed, writes limited to the working directory); if it is off, they run with the user's full access. (b) `mentalc` as a local stdio MCP server: the prompt is per MCP tool call by tool name; the server process and everything it spawns run outside the sandbox with the user's full access. The agent is therefore not a sufficient trust gate for `mentalc`'s internal subprocesses.
- Consequences for the specification: make execution a separate, explicitly named surface so the agent's gate can see it. Options: a distinct MCP tool for "resolve with build tool" (optionally annotated `requiresUserInteraction`) apart from the static-extraction tool; or have `mentalc` never spawn build tools and instead print the exact command for the agent to run through its own Bash tool, where the permission prompt and sandbox apply.
- A repository-supplied allow-list must not pre-authorise `mentalc` execution: Claude Code itself withholds repository `permissions.allow` rules until workspace trust, and in headless sessions there is no dialog at all, so `mentalc` needs its own default-deny.
- Codex follows the same shape (sandbox on spawned commands and their children, approval on the command), so the same reasoning applies; MCP coverage is unverified.

### Gaps
- No Claude Code sentence says in so many words "subprocesses of an MCP server are not subject to permission rules"; the conclusion combines "local MCP servers ... run with your full access" with the fact that rules match tool names and command text.
- Codex: whether MCP servers are sandboxed is not documented on the page read.

## 12. Supply-chain incidents that used install-time execution

### Takeaway
Install-time lifecycle scripts were the execution vector in the Nx compromise (August 2025), the Shai-Hulud worm (September 2025) and continue to be in 2026 (keyv, August 2026). These incidents are what GitHub cites for npm 12's new default.

### Cited Findings
- CONFIRMED (Nx, advisory published Aug 27, 2025): "Malicious versions of the nx package, as well as some supporting plugin packages, were published to npm"; "The compromised package contained a `postinstall` script that scanned user's file system for text files". — [GHSA-cxm3-wv7p-598c](https://github.com/nrwl/nx/security/advisories/GHSA-cxm3-wv7p-598c)
- CONFIRMED (Shai-Hulud, GitHub, Sept 22, 2025): "On September 14, 2025, we were notified of the Shai-Hulud attack."; "a self-replicating worm that infiltrated the npm ecosystem via compromised maintainer accounts"; "injecting malicious post-install scripts into popular JavaScript packages."; "Immediate removal of 500+ compromised packages from the npm registry." — [GitHub: Our plan for a more secure npm supply chain](https://github.blog/security/supply-chain-security/our-plan-for-a-more-secure-npm-supply-chain/)
- CONFIRMED (Shai-Hulud, CISA, Sept 23, 2025): "has compromised over 500 packages"; it "targeted GitHub Personal Access Tokens (PATs) and application programming interface (API) keys for cloud services". — [CISA alert](https://www.cisa.gov/news-events/alerts/2025/09/23/widespread-supply-chain-compromise-impacting-npm-ecosystem)
- CONFIRMED (keyv, first-hand vendor analysis, Aug 4, 2026): "The malicious releases add a `preinstall` hook that runs an obfuscated loader before application code starts"; "keyv 6.0.0 and ten related npm releases shipped install-time malware."; advisory "SNYK-JS-KEYV-18515941". — [Snyk](https://snyk.io/blog/inside-keyv-npm-compromise-preinstall-malware-trusted-provenance-ide-hooks/)
- CONFIRMED (the vendor response): "This major release turns on the install-time security defaults we announced in June". — [GitHub Changelog, 2026-07-08](https://github.blog/changelog/2026-07-08-npm-install-time-security-and-gat-bypass2fa-deprecation/)

### Inferences
- The keyv compromise happened about four weeks after npm 12 became `latest`; install hooks still pay off against clients on older npm versions, other package managers, or projects that have approved the package. A default-deny in `mentalc` is in line with where npm, pnpm, Yarn and Bun have all moved.

### Gaps
- Other 2026 incidents (jscrambler, July 2026; Mastra, June 2026; a `@redhat-cloud-services` campaign, June 2026) appeared only in search results from secondary sites and were not opened; do not cite them from these notes.
- The November 2025 "Shai-Hulud 2.0" wave was not researched.

## Summary table

| Tool or command | Executes repository-controlled code | Needs network | Evidence URL |
| :- | :- | :- | :- |
| `cargo metadata` | Conditional: no build scripts or proc macros (inferred); repository `.cargo/config.toml` can name `rustc`, wrappers, credential providers; `rust-toolchain.toml` picks the toolchain | Yes unless `--offline`/`--frozen`; `--no-deps` skips fetching | https://doc.rust-lang.org/cargo/commands/cargo-metadata.html |
| `cargo build` / `check` | Yes: `build.rs`, proc macros | Yes unless offline | https://doc.rust-lang.org/cargo/reference/build-scripts.html |
| `cargo` via rustup proxy | Conditional: `rust-toolchain.toml` selects (and with `RUSTUP_AUTO_INSTALL=1` installs) a toolchain | Conditional | https://rust-lang.github.io/rustup/environment-variables.html |
| `go list -m all`, `go list -json -deps`, `go mod download` | Conditional: no package code (inferred); `go`/`toolchain` line switches to a downloaded toolchain unless `GOTOOLCHAIN=local`; direct mode runs `git`/`hg` | Yes unless `GOPROXY=off` / `-mod=vendor` | https://go.dev/doc/toolchain |
| `go list -export` / `-compiled` | Conditional: compiles; cgo runs the C compiler and `pkg-config` | As above | https://pkg.go.dev/cmd/go |
| `mvn dependency:tree`, `dependency:build-classpath` | Yes: `.mvn/extensions.xml`, POM build extensions, `.mvn/maven.config`, `.mvn/jvm.config` | Yes (plugins, extensions, dependencies) unless `-o` with a warm cache | https://maven.apache.org/configure.html |
| `./mvnw` | Yes: repository script; downloads and runs `distributionUrl` | Yes on first use | https://maven.apache.org/tools/wrapper/ |
| `gradle dependencies`, Tooling API model fetch | Yes: settings, build scripts, `buildSrc`, included builds | Yes unless `--offline` with a warm cache | https://docs.gradle.org/current/userguide/build_lifecycle.html |
| `./gradlew` | Yes: repository wrapper JAR; downloads and runs `distributionUrl` | Yes on first use | https://docs.gradle.org/current/userguide/gradle_wrapper.html |
| `npm install` / `npm ci` (npm 12) | Conditional: root lifecycle scripts and `.npm-extension` yes (not under `npm ci` for the extension); dependency scripts only if in `allowScripts`; nothing with `--ignore-scripts` | Yes | https://docs.npmjs.com/cli/v12/using-npm/config |
| `npm install` (npm 11 and earlier) | Yes: root and dependency lifecycle scripts unless `--ignore-scripts` | Yes | https://docs.npmjs.com/cli/v11/using-npm/scripts |
| `npm ls`, `npm ls --package-lock-only` | No (none documented) | No | https://docs.npmjs.com/cli/v12/commands/npm-ls |
| `pnpm install` (10+) | Conditional: root scripts and `.pnpmfile` yes; dependency builds only if allowed (`allowBuilds`, formerly `onlyBuiltDependencies`) | Yes | https://pnpm.io/settings/build |
| `pnpm list` | Conditional: `--find-by` loads `.pnpmfile`; otherwise none documented | No | https://pnpm.io/cli/list |
| `yarn` (Berry) in a checkout | Yes if `.yarnrc.yml` sets `yarnPath` (unless `ignorePath`); `yarn.config.cjs` constraints; workspace postinstall | Yes for install | https://yarnpkg.com/configuration/yarnrc |
| Corepack shim (`yarn`, `pnpm`) | Conditional: downloads and runs the version in `packageManager`; URLs only with `COREPACK_ENABLE_UNSAFE_CUSTOM_URLS=1` | Yes unless cached / `COREPACK_ENABLE_NETWORK=0` | https://github.com/nodejs/corepack/blob/main/README.md |
| `bun install` | Conditional: dependency scripts only for `trustedDependencies` and the default list; none with `--ignore-scripts` | Yes | https://bun.sh/docs/pm/lifecycle |
| pip / uv building an sdist or editable project | Yes: build backend from `pyproject.toml` (may be in-tree) | Yes (build requirements) | https://docs.astral.sh/uv/reference/settings/ |
| `uv sync` | Conditional: yes when the project has a build system; avoid with `--no-install-project --frozen` | Yes | https://docs.astral.sh/uv/concepts/projects/sync/ |
| `poetry install` | Conditional: project-required plugins are installed and loaded | Yes | https://python-poetry.org/docs/plugins/ |
| Python interpreter start in an environment | Yes: `.pth` `import` lines, `sitecustomize` (unless `-S`) | No | https://docs.python.org/3/library/site.html |
| `bundle install` / any Bundler command | Yes: `Gemfile` is evaluated as Ruby | Yes for install | https://guides.rubygems.org/gemfile/ |
| `composer install` | Conditional: root `scripts` yes; plugins only if in `allow-plugins` (or approved at the prompt); `--no-plugins` disables | Yes | https://getcomposer.org/doc/06-config.md |
| SwiftPM (`swift package ...`) | Yes, sandboxed (inferred from "the same sandbox that manifest parsing uses") | Conditional | https://github.com/swiftlang/swift-evolution/blob/main/proposals/0303-swiftpm-extensible-build-tools.md |
| `docker compose config` | No execution documented; reads `.env`, env files, `include`; interpolates host environment into output | Conditional: remote `include`, `--resolve-image-digests` | https://docs.docker.com/reference/cli/docker/compose/config/ |
| `git status` / `git log` in a checkout | Conditional: no for a fresh network clone; yes if `.git` itself is untrusted (config such as `core.fsmonitor`, hooks) | No | https://git-scm.com/docs/git |
| `git submodule update --init` | No code from `.gitmodules` (custom command not accepted there); fetches repository-chosen URLs | Yes | https://git-scm.com/docs/git-submodule |
| Subprocess of a Claude Code Bash command | Not prompted individually; inside the Bash sandbox when enabled | Per sandbox allow-list | https://code.claude.com/docs/en/sandboxing |
| Subprocess of a local MCP server under Claude Code | Not prompted; outside the sandbox, "with your full access" | Unrestricted | https://code.claude.com/docs/en/sandboxing |
