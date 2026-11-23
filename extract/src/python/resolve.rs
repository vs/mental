//! Sites, with the build unit's roots, Python version and environment, to
//! outcomes, by ty: the only stage that reads across files, and the only
//! code whose reads are not ours. ty reads the environment from disk itself,
//! read-only, through [`system::ReadOnly`].

mod packages;
mod system;

use std::collections::BTreeSet;
use std::fmt;

use mental_model::{BuildUnit, Outcome, Package, Pin, Reason, SiteKind};
use ruff_db::files::{File, system_path_to_file};
use ruff_db::parsed::parsed_module;
use ruff_db::source::source_text;
use ruff_db::system::{SystemPath, SystemPathBuf};
use ruff_python_ast::find_node::covering_node;
use ruff_python_ast::visitor::{self, Visitor};
use ruff_python_ast::{AnyNodeRef, Expr, ExprRef, Stmt};
use ruff_ranged_value::RangedValue;
use ruff_text_size::{Ranged, TextRange, TextSize};
use ty_ide::{NavigationTargets, goto_declaration, goto_definition, goto_type_definition};
use ty_module_resolver::file_to_module;
use ty_project::metadata::options::{EnvironmentOptions, Options};
use ty_project::metadata::python_version::SupportedPythonVersion;
use ty_project::metadata::value::RelativePathBuf;
use ty_project::{ProjectDatabase, ProjectMetadata, SemanticDb};
use ty_python_core::ProgramFile;
use ty_python_semantic::types::ide_support::call_signature_details;
use ty_python_semantic::types::{Type, TypeVarKind};
use ty_python_semantic::{HasType, SemanticModel};

use super::sites::Listed;
use super::{Environment, PythonFile};
use packages::Packages;

/// The version of ty that the pinned ruff commit builds.
pub(super) const TY_VERSION: &str = "0.0.105";

/// Why ty could not be set up for a build unit, or read one of its files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolveError {
    /// The build unit records no environment, or no Python version, so the
    /// gaps of stage 1 are not all closed.
    Unprepared { missing: &'static str },
    /// The project directory is relative or has a `..` segment, which ty
    /// would not match against the roots.
    ProjectDir(String),
    /// The Python version is one ty does not check.
    PythonVersion { version: String, reason: String },
    /// ty could not open the project.
    Project(String),
    /// ty could not find a file `mentalc` read.
    File { path: String, reason: String },
}

impl fmt::Display for ResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResolveError::Unprepared { missing } => write!(
                f,
                "the build unit records no {missing}, which the type checker needs; close \
                 the gaps of the build model first"
            ),
            ResolveError::ProjectDir(dir) => write!(
                f,
                "the project directory `{dir}` is relative or has a `..` segment; pass it \
                 absolute and with no `..`, as the type checker matches it against the roots"
            ),
            ResolveError::PythonVersion { version, reason } => write!(
                f,
                "the type checker does not check Python {version}: {reason}; set a version it \
                 checks in `.python-version`"
            ),
            ResolveError::Project(reason) => write!(
                f,
                "the type checker could not open the project: {reason}; check that the project \
                 directory and its environment are readable"
            ),
            ResolveError::File { path, reason } => write!(
                f,
                "the type checker could not read `{path}`: {reason}; build again once the file \
                 is in place"
            ),
        }
    }
}

impl std::error::Error for ResolveError {}

/// A site's outcome, and whether what it names is a `def` or `class`; an
/// escape must be one of the repository.
pub(super) struct Checked {
    pub(super) outcome: Outcome,
    pub(super) definition: bool,
    /// The site has no declaration, and ty types it with a type nothing of
    /// which can be called, as `__name__` is a `str`: handed to a call, it
    /// hands over no code.
    pub(super) uncallable: bool,
}

pub(super) struct Resolver {
    db: ProjectDatabase,
    project: SystemPathBuf,
    packages: Packages,
}

impl Resolver {
    pub(super) fn new(
        unit: &BuildUnit,
        pins: &[Pin],
        environment: &Environment,
        files: &[PythonFile],
    ) -> Result<Resolver, ResolveError> {
        let project = SystemPathBuf::from(environment.project_dir);
        if !project.is_absolute() || project.components().any(|part| part.as_str() == "..") {
            return Err(ResolveError::ProjectDir(environment.project_dir.to_owned()));
        }
        let python = unit.python.as_ref().ok_or(ResolveError::Unprepared {
            missing: "Python version",
        })?;
        let dir = unit
            .environment
            .as_deref()
            .ok_or(ResolveError::Unprepared {
                missing: "environment",
            })?;
        let version = version(&python.version)?;
        let texts = files
            .iter()
            .map(|file| (project.join(file.path), file.text.to_owned()))
            .collect();
        let system = system::ReadOnly::new(&project, texts);
        let mut metadata = ProjectMetadata::new(unit.id.as_str(), project.clone());
        metadata.set_override_options(Options {
            environment: Some(EnvironmentOptions {
                root: Some(unit.roots.iter().map(RelativePathBuf::cli).collect()),
                python: Some(RelativePathBuf::cli(dir)),
                python_version: Some(RangedValue::cli(version)),
                ..EnvironmentOptions::default()
            }),
            ..Options::default()
        });
        let db = ProjectDatabase::fallible(metadata, system)
            .map_err(|error| ResolveError::Project(error.to_string()))?;
        Ok(Resolver {
            db,
            project,
            packages: Packages::new(&unit.id, pins, environment.records),
        })
    }

    /// The outcome of each site of `file`, in the order listed.
    pub(super) fn check(
        &self,
        file: &PythonFile,
        sites: &[Listed],
    ) -> Result<Vec<Checked>, ResolveError> {
        let path = self.project.join(file.path);
        let found = system_path_to_file(&self.db, &path).map_err(|error| ResolveError::File {
            path: file.path.to_owned(),
            reason: error.to_string(),
        })?;
        let program = self.db.program_file(found);
        let model = SemanticModel::new(&self.db, program);
        Ok(sites
            .iter()
            .map(|site| self.site(program, &model, site))
            .collect())
    }

    fn site(&self, program: ProgramFile, model: &SemanticModel, site: &Listed) -> Checked {
        let Some(ask) = site.ask else {
            return unresolved(Reason::Dynamic);
        };
        let found = match site.kind {
            SiteKind::WriteReceiver => {
                let receiver = site
                    .receiver
                    .expect("a write receiver's site records the object written to");
                if let Some(reason) = self.untyped(program, model, receiver) {
                    return unresolved(reason);
                }
                self.class_of(program, model, receiver, ask)
            }
            SiteKind::Callee | SiteKind::Decorator => match self.called(program, model, site.range)
            {
                Called::Class(callee) => self.class_of(program, model, callee, ask),
                Called::Untyped(reason) => {
                    if self
                        .targets(goto_declaration(&self.db, program, ask))
                        .is_empty()
                    {
                        BTreeSet::new()
                    } else {
                        return unresolved(reason);
                    }
                }
                Called::Runs(runs) if !runs.is_empty() => runs,
                Called::Runs(_) | Called::NotACall => {
                    self.targets(goto_declaration(&self.db, program, ask))
                }
            },
            SiteKind::Import | SiteKind::BaseClass | SiteKind::Escape | SiteKind::HandlerType => {
                self.targets(goto_declaration(&self.db, program, ask))
            }
        };
        let mut found = found.into_iter();
        match (found.next(), found.next()) {
            (Some((outcome, definition)), None) => Checked {
                outcome,
                definition,
                uncallable: false,
            },
            (Some(_), Some(_)) => unresolved(Reason::Ambiguous),
            (None, _) => Checked {
                uncallable: site.kind == SiteKind::Escape
                    && !may_be_called(&self.db, program, model, site.range),
                ..unresolved(self.no_answer(program, model, site))
            },
        }
    }

    /// The outcomes of what ty found, each once.
    fn targets(
        &self,
        found: Option<ty_ide::RangedValue<NavigationTargets>>,
    ) -> BTreeSet<(Outcome, bool)> {
        found
            .into_iter()
            .flat_map(|targets| targets.value)
            .map(|target| self.target(target.file(), target.focus_range()))
            .collect()
    }

    /// What runs when the call at `range` is made: the definition of each
    /// signature ty matches it with, as `__call__` for `Session()` where
    /// `Session` is a variable. When the callee is a class, the class is what
    /// is called, not the `__init__` or `__new__` it runs.
    fn called(&self, program: ProgramFile, model: &SemanticModel, range: TextRange) -> Called {
        let parsed = parsed_module(&self.db, program.python_file(&self.db)).load(&self.db);
        let Some(ExprRef::Call(call)) = covering_node(parsed.syntax().into(), range)
            .node()
            .as_expr_ref()
        else {
            return Called::NotACall;
        };
        if let Some(Type::ClassLiteral(_) | Type::GenericAlias(_) | Type::SubclassOf(_)) =
            call.func.inferred_type(model)
        {
            return Called::Class(call.func.range());
        }
        if let Some(reason) = self.untyped(program, model, call.func.range()) {
            return Called::Untyped(reason);
        }
        let runs = call_signature_details(model, call)
            .into_iter()
            .filter_map(|signature| signature.definition)
            .map(|definition| {
                let module =
                    parsed_module(&self.db, definition.python_file(&self.db)).load(&self.db);
                let at = definition.focus_range(&self.db, &module);
                self.target(at.file(), at.range())
            })
            .collect();
        Called::Runs(runs)
    }

    /// Why ty found nothing a site names.
    fn no_answer(&self, program: ProgramFile, model: &SemanticModel, site: &Listed) -> Reason {
        if site.kind == SiteKind::Import {
            let module_found = site
                .module
                .and_then(|module| goto_definition(&self.db, program, module))
                .is_some_and(|targets| targets.value.into_iter().next().is_some());
            return if module_found {
                Reason::NotFound
            } else {
                Reason::MissingModule
            };
        }
        site.receiver
            .and_then(|receiver| self.untyped(program, model, receiver))
            .unwrap_or(Reason::NotFound)
    }

    /// Why the object at `receiver` has no type to look a name up in: none
    /// ty can name (`Unknown`, `Any`) or `Never`. `None` when it has one;
    /// asked for its definition, ty would name the special form instead.
    fn untyped(
        &self,
        program: ProgramFile,
        model: &SemanticModel,
        receiver: TextRange,
    ) -> Option<Reason> {
        match type_at(&self.db, program, model, receiver) {
            Some(Type::Dynamic(_) | Type::Divergent(_)) | None => Some(Reason::NoType),
            Some(Type::Never) => Some(Reason::Unreachable),
            Some(_) => None,
        }
    }

    /// The class of the object at `at`, or the class it is: the class a
    /// `Self` stands for, else ty's "go to type definition" asked at `ask`.
    fn class_of(
        &self,
        program: ProgramFile,
        model: &SemanticModel,
        at: TextRange,
        ask: TextSize,
    ) -> BTreeSet<(Outcome, bool)> {
        match self.self_class(program, model, at) {
            Some(class) => BTreeSet::from([class]),
            None => self.targets(goto_type_definition(&self.db, program, ask)),
        }
    }

    /// The class `Self` stands for, when the object at `at` is `Self` or
    /// `type[Self]`, as `self` and `cls` are in a method. ty's "go to type
    /// definition" finds nothing for it: the `Self` of a method is made up
    /// by ty and has no definition.
    ///
    /// shortcut: ty keeps the class of a `Self` crate-private, so this takes
    /// the class of the method that holds `at`, which is wrong only for the
    /// `Self` of an outer method used inside a class nested in it; upgrade
    /// when ty makes the class public.
    fn self_class(
        &self,
        program: ProgramFile,
        model: &SemanticModel,
        at: TextRange,
    ) -> Option<(Outcome, bool)> {
        let typed = type_at(&self.db, program, model, at)
            .expect("a class is asked for only where ty typed the object");
        let bound = if let Type::TypeVar(bound) = typed {
            bound
        } else if let Type::SubclassOf(class) = typed {
            class.into_type_var()?
        } else {
            return None;
        };
        if !matches!(
            bound.typevar(&self.db).kind(&self.db),
            TypeVarKind::TypingSelf { .. }
        ) {
            return None;
        }
        let parsed = parsed_module(&self.db, program.python_file(&self.db)).load(&self.db);
        let mut method = MethodClass {
            at,
            in_class_body: None,
            class: None,
        };
        method.visit_body(&parsed.syntax().body);
        Some(self.target(program.file(&self.db), method.class?))
    }

    /// The outcome a definition ty found stands for, and whether it is a
    /// `def` or `class`. ty names the definition's module and says whether
    /// it lies in a root, the standard library or `site-packages`.
    fn target(&self, file: File, focus: TextRange) -> (Outcome, bool) {
        let (name, definition) = qualified(&self.db, file, focus);
        let not_found = (
            Outcome::Unresolved {
                reason: Reason::NotFound,
            },
            false,
        );
        let resolver_file = self.db.program_file(file).resolver_file(&self.db);
        let Some(found) = file_to_module(&self.db, resolver_file) else {
            return not_found;
        };
        let module = found.name(&self.db).to_string();
        let search_path = found
            .search_path(&self.db)
            .expect("a module found from a file is not a namespace package, which has no file");
        if search_path.is_first_party() {
            (Outcome::Resolved { module, name }, definition)
        } else if search_path.is_standard_library() {
            let package = Package::Stdlib;
            (
                Outcome::External {
                    package,
                    module,
                    name,
                },
                definition,
            )
        } else if search_path.is_site_packages() {
            let path = file
                .path(&self.db)
                .as_system_path()
                .expect("a file in site-packages is on disk");
            let site_packages = search_path
                .as_system_path()
                .expect("site-packages is a directory on disk");
            let installed = relative(path, site_packages)
                .expect("a module found on a search path lies under it");
            let Some(pin) = self.packages.pin(installed) else {
                return not_found;
            };
            let package = Package::Pin(pin.clone());
            (
                Outcome::External {
                    package,
                    module,
                    name,
                },
                definition,
            )
        } else {
            not_found
        }
    }
}

/// What a call site calls, as far as its syntax and the callee's type tell.
enum Called {
    /// A class, which is the target itself, by the callee at this range.
    Class(TextRange),
    /// A callee of no type ty can name, or of type `Never`: a value of no
    /// type when ty declares the name, else a name nothing defines.
    Untyped(Reason),
    /// The definitions of the signatures ty matched the call with.
    Runs(BTreeSet<(Outcome, bool)>),
    /// The site is a decorator that is not a call.
    NotACall,
}

/// ty's type of the expression at `at`.
fn type_at<'db>(
    db: &'db ProjectDatabase,
    program: ProgramFile,
    model: &SemanticModel<'db>,
    at: TextRange,
) -> Option<Type<'db>> {
    let parsed = parsed_module(db, program.python_file(db)).load(db);
    covering_node(parsed.syntax().into(), at)
        .node()
        .as_expr_ref()
        .and_then(|expr| expr.inferred_type(model))
}

/// Whether the argument at `at` may be called: ty upcasts its type to a
/// callable, as it does for a function, a class, an instance with
/// `__call__` and a value of no type it can name.
///
/// shortcut: ty keeps that upcast crate-private, so this asks the public
/// signatures of the call that holds the argument with the argument as its
/// callee, which ty types by the argument's node; upgrade when ty makes the
/// upcast public.
fn may_be_called(
    db: &ProjectDatabase,
    program: ProgramFile,
    model: &SemanticModel,
    at: TextRange,
) -> bool {
    let parsed = parsed_module(db, program.python_file(db)).load(db);
    let covering = covering_node(parsed.syntax().into(), at);
    let call = covering
        .ancestors()
        .find_map(|node| {
            if let AnyNodeRef::ExprCall(call) = node {
                Some(call)
            } else {
                None
            }
        })
        .expect("an escape candidate is handed to a call");
    let argument = call
        .arguments
        .args
        .iter()
        .chain(call.arguments.keywords.iter().map(|keyword| &keyword.value))
        .find(|value| value.range() == at)
        .expect("an escape candidate is an argument of its call");
    let mut asked = call.clone();
    *asked.func = argument.clone();
    !call_signature_details(model, &asked).is_empty()
}

fn unresolved(reason: Reason) -> Checked {
    Checked {
        outcome: Outcome::Unresolved { reason },
        definition: false,
        uncallable: false,
    }
}

/// `path` relative to `base`, with `/` between folders.
fn relative<'p>(path: &'p SystemPath, base: &SystemPath) -> Option<&'p str> {
    path.strip_prefix(base).ok().map(SystemPath::as_str)
}

/// ty checks a minor version; a patch, as in `3.13.1`, changes no rule.
fn version(stated: &str) -> Result<SupportedPythonVersion, ResolveError> {
    let minor = stated.splitn(3, '.').take(2).collect::<Vec<_>>().join(".");
    minor.parse().map_err(
        |error: <SupportedPythonVersion as std::str::FromStr>::Err| ResolveError::PythonVersion {
            version: stated.to_owned(),
            reason: error.to_string(),
        },
    )
}

/// The qualified name of what stands at `focus` in `file`, and whether it
/// is the name of a `def` or `class`. An empty focus is the module itself.
fn qualified(db: &ProjectDatabase, file: File, focus: TextRange) -> (String, bool) {
    if focus.is_empty() {
        return (String::new(), false);
    }
    let parsed = parsed_module(db, db.program_file(file).python_file(db)).load(db);
    let mut enclosing = Enclosing {
        focus,
        scopes: Vec::new(),
        definition: false,
    };
    enclosing.visit_body(&parsed.syntax().body);
    let text = source_text(db, file);
    enclosing.scopes.push(text[focus].to_owned());
    (enclosing.scopes.join("."), enclosing.definition)
}

/// The `def`s and `class`es that hold a range, outermost first.
struct Enclosing {
    focus: TextRange,
    scopes: Vec<String>,
    definition: bool,
}

impl<'a> Visitor<'a> for Enclosing {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if !stmt.range().contains_range(self.focus) {
            return;
        }
        let name = if let Stmt::FunctionDef(def) = stmt {
            Some(&def.name)
        } else if let Stmt::ClassDef(class) = stmt {
            Some(&class.name)
        } else {
            None
        };
        if let Some(name) = name {
            if name.range == self.focus {
                self.definition = true;
                return;
            }
            self.scopes.push(name.to_string());
        }
        visitor::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, _expr: &'a Expr) {}
}

/// The name of the class whose method holds a range: the class of the
/// innermost `def` that holds the range and sits directly in a class body,
/// not in another `def`.
struct MethodClass {
    at: TextRange,
    in_class_body: Option<TextRange>,
    class: Option<TextRange>,
}

impl<'a> Visitor<'a> for MethodClass {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if !stmt.range().contains_range(self.at) {
            return;
        }
        if let Stmt::ClassDef(class) = stmt {
            self.in_class_body = Some(class.name.range);
        } else if let Stmt::FunctionDef(_) = stmt {
            self.class = self.in_class_body.take().or(self.class);
        }
        visitor::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, _expr: &'a Expr) {}
}

#[cfg(test)]
mod tests;
