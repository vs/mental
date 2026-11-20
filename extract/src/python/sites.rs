//! One file's syntax tree to its reference sites, each with the scope that
//! holds it, the name it refers to as written, and where to ask the type
//! checker what that name is. Every name or attribute handed to a call is
//! listed as an escape candidate; resolution keeps the escapes among them.

use mental_model::SiteKind;
use ruff_python_ast::visitor::{self, Visitor};
use ruff_python_ast::{Arguments, Decorator, ExceptHandler, Expr, Stmt};
use ruff_text_size::{Ranged, TextRange, TextSize};

/// A site before it has an ID or an outcome.
pub(super) struct Listed {
    pub(super) kind: SiteKind,
    /// The qualified name of the innermost module, class or function that
    /// holds the site; a site in the header of a `def` or `class` belongs
    /// to it.
    pub(super) scope: String,
    pub(super) name: String,
    pub(super) range: TextRange,
    /// Where the checker is asked: the start of the last name the site is
    /// written with. `None` when it is written with no name, as in
    /// `handlers[0]()` or a star import.
    pub(super) ask: Option<TextSize>,
    /// The object the asked name is an attribute of, whose type says why
    /// the checker has no answer; for a write receiver, the object written to.
    pub(super) receiver: Option<TextRange>,
    /// For `from m import x`: where to ask whether `m` exists.
    pub(super) module: Option<TextSize>,
    /// For an escape candidate: the listed site of the call it is handed to.
    pub(super) call: Option<usize>,
}

pub(super) fn sites(module: &str, body: &[Stmt]) -> Vec<Listed> {
    let mut walk = Walk {
        scope: vec![module.to_owned()],
        sites: Vec::new(),
    };
    walk.visit_body(body);
    walk.sites
}

struct Walk {
    scope: Vec<String>,
    sites: Vec<Listed>,
}

impl Walk {
    /// Lists a site written as `expr`, asking for its last name; returns its index.
    fn list(&mut self, kind: SiteKind, expr: &Expr, range: TextRange) -> usize {
        let (ask, receiver) = asked(expr);
        self.push(Listed {
            kind,
            scope: self.scope.join("."),
            name: written(expr),
            range,
            ask,
            receiver,
            module: None,
            call: None,
        })
    }

    fn push(&mut self, site: Listed) -> usize {
        self.sites.push(site);
        self.sites.len() - 1
    }

    /// Lists each name or attribute in `arguments` as an escape candidate of `call`.
    fn candidates(&mut self, call: usize, arguments: &Arguments) {
        let values = arguments
            .args
            .iter()
            .chain(arguments.keywords.iter().map(|keyword| &keyword.value));
        for value in values {
            if matches!(value, Expr::Name(_) | Expr::Attribute(_)) {
                let index = self.list(SiteKind::Escape, value, value.range());
                self.sites[index].call = Some(call);
            }
        }
    }

    fn import(&mut self, name: String, alias: &ruff_python_ast::Alias, module: Option<TextSize>) {
        let ask = (alias.name.as_str() != "*").then(|| last_segment(&alias.name));
        self.push(Listed {
            kind: SiteKind::Import,
            scope: self.scope.join("."),
            name,
            range: alias.range(),
            ask,
            receiver: None,
            module,
            call: None,
        });
    }
}

impl<'a> Visitor<'a> for Walk {
    fn visit_stmt(&mut self, stmt: &'a Stmt) {
        if let Stmt::FunctionDef(def) = stmt {
            self.scope.push(def.name.to_string());
            visitor::walk_stmt(self, stmt);
            self.scope.pop();
        } else if let Stmt::ClassDef(class) = stmt {
            self.scope.push(class.name.to_string());
            for base in class.bases() {
                self.list(SiteKind::BaseClass, base, base.range());
            }
            visitor::walk_stmt(self, stmt);
            self.scope.pop();
        } else if let Stmt::Import(import) = stmt {
            for alias in &import.names {
                self.import(alias.name.to_string(), alias, None);
            }
        } else if let Stmt::ImportFrom(import) = stmt {
            let module = import.module.as_ref();
            let written = module.map_or("", |module| module.as_str());
            let asked = module.map(last_segment);
            for alias in &import.names {
                self.import(format!("{written}.{}", alias.name), alias, asked);
            }
        } else {
            visitor::walk_stmt(self, stmt);
        }
    }

    fn visit_decorator(&mut self, decorator: &'a Decorator) {
        let expression = &decorator.expression;
        if let Expr::Call(call) = expression {
            let site = self.list(SiteKind::Decorator, &call.func, expression.range());
            self.candidates(site, &call.arguments);
            self.visit_expr(&call.func);
            self.visit_arguments(&call.arguments);
        } else {
            self.list(SiteKind::Decorator, expression, expression.range());
            self.visit_expr(expression);
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Call(call) = expr {
            let site = self.list(SiteKind::Callee, &call.func, call.range());
            self.candidates(site, &call.arguments);
        } else if let Expr::Attribute(attribute) = expr
            && attribute.ctx.is_store()
        {
            let index = self.list(SiteKind::WriteReceiver, expr, attribute.range());
            let (ask, _) = asked(&attribute.value);
            self.sites[index].ask = ask;
            self.sites[index].receiver = Some(attribute.value.range());
        }
        visitor::walk_expr(self, expr);
    }

    fn visit_except_handler(&mut self, handler: &'a ExceptHandler) {
        let ExceptHandler::ExceptHandler(clause) = handler;
        if let Some(caught) = &clause.type_ {
            let types = if let Expr::Tuple(tuple) = &**caught {
                &tuple.elts[..]
            } else {
                std::slice::from_ref(&**caught)
            };
            for caught in types {
                self.list(SiteKind::HandlerType, caught, caught.range());
            }
        }
        visitor::walk_except_handler(self, handler);
    }
}

/// Where the last segment of a dotted name starts: `b` in `import a.b`.
fn last_segment(name: &ruff_python_ast::Identifier) -> TextSize {
    let last = name.as_str().rfind('.').map_or(0, |dot| dot + 1);
    name.range.start() + TextSize::try_from(last).expect("a dotted name is shorter than 4 GiB")
}

/// Where to ask what `expr` names, and the object it is an attribute of:
/// a name is asked at its start, an attribute at the start of its last part.
fn asked(expr: &Expr) -> (Option<TextSize>, Option<TextRange>) {
    if let Expr::Name(name) = expr {
        (Some(name.range.start()), None)
    } else if let Expr::Attribute(attribute) = expr {
        (
            Some(attribute.attr.range.start()),
            Some(attribute.value.range()),
        )
    } else {
        (None, None)
    }
}

/// An expression as its name is written: names and attributes as they
/// are, a call as `()` and a subscript as `[]` after what they apply to,
/// and nothing for any other expression, such as the literal of `"".join`.
fn written(expr: &Expr) -> String {
    match expr {
        Expr::Name(name) => name.id.to_string(),
        Expr::Attribute(attribute) => format!("{}.{}", written(&attribute.value), attribute.attr),
        Expr::Call(call) => format!("{}()", written(&call.func)),
        Expr::Subscript(subscript) => format!("{}[]", written(&subscript.value)),
        Expr::BoolOp(_)
        | Expr::Named(_)
        | Expr::BinOp(_)
        | Expr::UnaryOp(_)
        | Expr::Lambda(_)
        | Expr::If(_)
        | Expr::Dict(_)
        | Expr::Set(_)
        | Expr::ListComp(_)
        | Expr::SetComp(_)
        | Expr::DictComp(_)
        | Expr::Generator(_)
        | Expr::Await(_)
        | Expr::Yield(_)
        | Expr::YieldFrom(_)
        | Expr::Compare(_)
        | Expr::FString(_)
        | Expr::TString(_)
        | Expr::StringLiteral(_)
        | Expr::BytesLiteral(_)
        | Expr::NumberLiteral(_)
        | Expr::BooleanLiteral(_)
        | Expr::NoneLiteral(_)
        | Expr::EllipsisLiteral(_)
        | Expr::Starred(_)
        | Expr::List(_)
        | Expr::Tuple(_)
        | Expr::Slice(_)
        | Expr::IpyEscapeCommand(_) => String::new(),
    }
}

#[cfg(test)]
mod tests;
