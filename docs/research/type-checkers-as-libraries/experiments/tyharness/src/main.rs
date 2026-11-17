//! Asks ty, used as a library, what every reference site of a project resolves to.
//!
//! Usage: tyharness <ty.toml> <project-dir> <out.json> <file-or-dir>...
//!
//! For each Python file it walks ruff's syntax tree (the tree ty itself uses) and writes one
//! record per site: calls, attribute accesses (receiver type, load or store), decorators and
//! class bases. Targets are given as (path, qualified name inside the file) so the comparison
//! script can derive module names; ranges are `line:col-line:col`, 1-based, as Pyrefly writes them.

use std::time::Instant;

use anyhow::{Context, Result};
use ruff_db::files::{File, FilePath, system_path_to_file};
use ruff_db::parsed::parsed_module;
use ruff_db::source::{line_index, source_text};
use ruff_db::system::{OsSystem, SystemPath, SystemPathBuf};
use ruff_python_ast::visitor::source_order::{self, SourceOrderVisitor};
use ruff_python_ast::{self as ast, Expr, ExprContext, Stmt};
use ruff_text_size::{Ranged, TextRange, TextSize};
use serde_json::{Value, json};
use ty_project::{ProjectDatabase, ProjectMetadata};
use ty_python_core::ProgramFile;
use ty_python_semantic::{Db as SemDb, HasType, SemanticModel};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 5 {
        anyhow::bail!("usage: tyharness <ty.toml> <project-dir> <out.json> <file-or-dir>...");
    }
    let config = SystemPathBuf::from(args[1].as_str());
    let project = SystemPathBuf::from(args[2].as_str());
    let out = &args[3];
    let started = Instant::now();

    let system = OsSystem::new(&project);
    let mut meta = ProjectMetadata::from_config_file(config, &project, &system)?;
    meta.apply_configuration_files(&system)?;
    let db = ProjectDatabase::fallible(meta, system)?;
    let setup_ms = started.elapsed().as_millis();

    let mut paths = Vec::new();
    for a in &args[4..] {
        collect(&project.join(a), &mut paths)?;
    }
    paths.sort();

    let mut files = Vec::new();
    let mut diag_count = 0usize;
    for p in &paths {
        let file = system_path_to_file(&db, p).with_context(|| format!("no file {p}"))?;
        let pf = db.program_file(file);
        let mut v = Walker { db: &db, file, pf, model: SemanticModel::new(&db, pf), sites: Vec::new() };
        let parsed = parsed_module(&db, pf.python_file(&db)).load(&db);
        v.visit_body(parsed.suite());
        let diags = db.check_file(file);
        diag_count += diags.len();
        let diag_list: Vec<Value> = diags
            .iter()
            .map(|d| json!({"id": d.id().to_string(), "message": d.concise_message().to_string(),
                            "range": d.primary_span().and_then(|s| s.range()).map(|r| v.lc(r))}))
            .collect();
        files.push(json!({"path": p.as_str(), "sites": v.sites, "diagnostics": diag_list}));
    }
    let total_ms = started.elapsed().as_millis();
    let doc = json!({"tool": "ty", "ruff_rev": "4228f4c2168d50dc60df1aec705a1438c75a4888",
                     "setup_ms": setup_ms, "total_ms": total_ms, "diagnostics": diag_count, "files": files});
    std::fs::write(out, serde_json::to_string(&doc)?)?;
    eprintln!("ty: {} files, {} ms", paths.len(), total_ms);
    Ok(())
}

fn collect(p: &SystemPath, out: &mut Vec<SystemPathBuf>) -> Result<()> {
    let std_path = p.as_std_path();
    if std_path.is_file() {
        if p.extension() == Some("py") {
            out.push(p.to_path_buf());
        }
        return Ok(());
    }
    for entry in std::fs::read_dir(std_path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || name == "__pycache__" || name == "node_modules" {
            continue;
        }
        let child = SystemPathBuf::from_path_buf(entry.path()).map_err(|_| anyhow::anyhow!("non-utf8 path"))?;
        collect(&child, out)?;
    }
    Ok(())
}

struct Walker<'db> {
    db: &'db ProjectDatabase,
    file: File,
    pf: ProgramFile<'db>,
    model: SemanticModel<'db>,
    sites: Vec<Value>,
}

impl<'db> Walker<'db> {
    fn lc(&self, r: TextRange) -> String {
        lc_in(self.db, self.file, r)
    }

    /// The token a "go to definition" request is made on: the name itself, or the attribute name.
    fn token(e: &Expr) -> Option<TextSize> {
        match e {
            Expr::Name(n) => Some(n.range.start()),
            Expr::Attribute(a) => Some(a.attr.range.start()),
            _ => None,
        }
    }

    fn goto(&self, at: Option<TextSize>) -> Vec<Value> {
        let Some(offset) = at else { return Vec::new() };
        let Some(targets) = ty_ide::goto_definition(self.db, self.pf, offset) else { return Vec::new() };
        targets.value.into_iter().map(|t| target_json(self.db, t.file(), t.focus_range())).collect()
    }

    fn ty(&self, e: &Expr) -> Value {
        let Some(t) = e.inferred_type(&self.model) else { return json!({"display": null}) };
        let env = self.model.program_environment();
        let display = t.display(self.db, &env).to_string();
        let parts: Vec<_> = match t.as_union() {
            Some(u) => u.elements(self.db).to_vec(),
            None => vec![t],
        };
        let mut classes: Vec<Value> = Vec::new();
        for p in &parts {
            // A narrowed intersection such as `Recipe & ~AlwaysFalsy` has no definition of its own,
            // and its positive parts are crate-private: ask the IDE's public "go to type
            // definition", as an editor would. (Type variables such as `Self@m` are left to the
            // comparison script: for them this request lands on the `Self` import.)
            let pd = p.display(self.db, &env).to_string();
            if p.definition(self.db, &env).is_none() && pd.contains(" & ") && !pd.starts_with("Self@") {
                if let Some(nav) = Self::token(e).and_then(|o| ty_ide::goto_type_definition(self.db, self.pf, o)) {
                    let got: Vec<Value> =
                        nav.value.into_iter().map(|t| target_json(self.db, t.file(), t.focus_range())).collect();
                    if !got.is_empty() {
                        classes.extend(got);
                        continue;
                    }
                }
            }
            classes.push(match p.definition(self.db, &env).and_then(|d| d.focus_range(self.db)) {
                Some(fr) => target_json(self.db, fr.file(), fr.range()),
                None => json!({"display": p.display(self.db, &env).to_string()}),
            });
        }
        json!({"display": display, "unknown": t.is_unknown(), "parts": classes})
    }
}

impl<'a, 'db> SourceOrderVisitor<'a> for Walker<'db> {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        match stmt {
            Stmt::FunctionDef(f) => {
                for d in &f.decorator_list {
                    let callee = match &d.expression { Expr::Call(c) => &*c.func, e => e };
                    let s = json!({"kind": "decorator", "range": self.lc(d.expression.range()),
                                   "text": src(self.db, self.file, d.expression.range()),
                                   "targets": self.goto(Self::token(callee))});
                    self.sites.push(s);
                }
            }
            Stmt::ClassDef(c) => {
                for b in c.bases() {
                    let s = json!({"kind": "base", "range": self.lc(b.range()),
                                   "class_range": self.lc(c.name.range),
                                   "text": src(self.db, self.file, b.range()),
                                   "targets": self.goto(Self::token(b)), "type": self.ty(b)});
                    self.sites.push(s);
                }
            }
            _ => {}
        }
        source_order::walk_stmt(self, stmt);
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        match expr {
            Expr::Call(c) => {
                let s = json!({"kind": "call", "range": self.lc(c.range()),
                               "text": src(self.db, self.file, c.func.range()),
                               "callee_type": self.ty(&c.func),
                               "targets": self.goto(Self::token(&c.func))});
                self.sites.push(s);
            }
            Expr::Attribute(a) => {
                let ctx = match a.ctx { ExprContext::Store => "store", ExprContext::Del => "del", _ => "load" };
                let s = json!({"kind": "attr", "ctx": ctx, "range": self.lc(a.range()),
                               "base_range": self.lc(a.value.range()),
                               "text": src(self.db, self.file, a.range()),
                               "base_type": self.ty(&a.value),
                               "targets": self.goto(Some(a.attr.range.start()))});
                self.sites.push(s);
            }
            _ => {}
        }
        source_order::walk_expr(self, expr);
    }
}

fn lc_in(db: &ProjectDatabase, file: File, r: TextRange) -> String {
    let text = source_text(db, file);
    let idx = line_index(db, file);
    let a = idx.line_column(r.start(), &text);
    let b = idx.line_column(r.end(), &text);
    format!("{}:{}-{}:{}", a.line.get(), a.column.get(), b.line.get(), b.column.get())
}

fn src(db: &ProjectDatabase, file: File, r: TextRange) -> String {
    let text = source_text(db, file);
    let s = &text[r];
    if s.len() > 80 { format!("{}…", &s[..s.floor_char_boundary(80)]) } else { s.to_string() }
}

/// A definition as (path, dotted names of the enclosing classes and functions plus its own).
fn target_json(db: &ProjectDatabase, file: File, focus: TextRange) -> Value {
    let path = match file.path(db) {
        FilePath::System(p) => p.as_str().to_string(),
        FilePath::SystemVirtual(p) => format!("virtual:{}", p.as_str()),
        FilePath::Vendored(p) => format!("vendored:{}", p.as_str()),
    };
    let parsed = parsed_module(db, db.program_file(file).python_file(db)).load(db);
    let mut chain = Vec::new();
    let mut kind = "module";
    find_chain(parsed.suite(), focus, &mut chain, &mut kind);
    json!({"path": path, "name": chain.join("."), "def": kind, "line": lc_in(db, file, focus)})
}

fn find_chain(body: &[Stmt], focus: TextRange, chain: &mut Vec<String>, kind: &mut &'static str) -> bool {
    for stmt in body {
        if !stmt.range().contains_range(focus) {
            continue;
        }
        match stmt {
            Stmt::FunctionDef(f) => {
                chain.push(f.name.id.to_string());
                if f.name.range == focus {
                    *kind = "function";
                } else if !find_chain(&f.body, focus, chain, kind) {
                    *kind = "variable"; // a parameter
                }
                return true;
            }
            Stmt::ClassDef(c) => {
                chain.push(c.name.id.to_string());
                *kind = "class";
                if c.name.range == focus {
                    return true;
                }
                find_chain(&c.body, focus, chain, kind);
                return true;
            }
            Stmt::If(i) => {
                if find_chain(&i.body, focus, chain, kind) {
                    return true;
                }
                for clause in &i.elif_else_clauses {
                    if find_chain(&clause.body, focus, chain, kind) {
                        return true;
                    }
                }
            }
            Stmt::Try(t) => {
                for b in [&t.body, &t.orelse, &t.finalbody] {
                    if find_chain(b, focus, chain, kind) {
                        return true;
                    }
                }
                for h in &t.handlers {
                    let ast::ExceptHandler::ExceptHandler(h) = h;
                    if find_chain(&h.body, focus, chain, kind) {
                        return true;
                    }
                }
            }
            Stmt::With(w) => {
                if find_chain(&w.body, focus, chain, kind) {
                    return true;
                }
            }
            _ => {
                // An assignment, import or other statement: name it by its text.
                let text = source_text_of(stmt, focus);
                *kind = if matches!(stmt, Stmt::Import(_) | Stmt::ImportFrom(_)) { "import" } else { "variable" };
                chain.push(text);
                return true;
            }
        }
    }
    false
}

fn source_text_of(stmt: &Stmt, _focus: TextRange) -> String {
    match stmt {
        Stmt::Assign(a) => a.targets.first().map(name_of).unwrap_or_else(|| "<assign>".into()),
        Stmt::AnnAssign(a) => name_of(&a.target),
        Stmt::Import(_) | Stmt::ImportFrom(_) => "<import>".into(),
        _ => "<stmt>".into(),
    }
}

fn name_of(e: &Expr) -> String {
    match e {
        Expr::Name(n) => n.id.to_string(),
        Expr::Attribute(a) => a.attr.id.to_string(),
        _ => "<target>".into(),
    }
}
