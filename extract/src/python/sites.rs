//! One file's syntax tree to its reference sites, each with the scope that
//! holds it and the name it refers to, as written.

use mental_model::SiteKind;
use ruff_python_ast::visitor::{self, Visitor};
use ruff_python_ast::{Decorator, ExceptHandler, Expr, Stmt};
use ruff_text_size::{Ranged, TextRange};

/// A site before it has an ID.
pub(super) struct Listed {
    pub(super) kind: SiteKind,
    /// The qualified name of the innermost module, class or function that
    /// holds the site; a site in the header of a `def` or `class` belongs
    /// to it.
    pub(super) scope: String,
    pub(super) name: String,
    pub(super) range: TextRange,
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
    fn list(&mut self, kind: SiteKind, name: String, range: TextRange) {
        self.sites.push(Listed {
            kind,
            scope: self.scope.join("."),
            name,
            range,
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
                self.list(SiteKind::BaseClass, written(base), base.range());
            }
            visitor::walk_stmt(self, stmt);
            self.scope.pop();
        } else if let Stmt::Import(import) = stmt {
            for alias in &import.names {
                self.list(SiteKind::Import, alias.name.to_string(), alias.range());
            }
        } else if let Stmt::ImportFrom(import) = stmt {
            let module = import.module.as_ref().map_or("", |module| module.as_str());
            for alias in &import.names {
                let name = format!("{module}.{}", alias.name);
                self.list(SiteKind::Import, name, alias.range());
            }
        } else {
            visitor::walk_stmt(self, stmt);
        }
    }

    fn visit_decorator(&mut self, decorator: &'a Decorator) {
        let expression = &decorator.expression;
        if let Expr::Call(call) = expression {
            self.list(SiteKind::Decorator, written(&call.func), expression.range());
            self.visit_expr(&call.func);
            self.visit_arguments(&call.arguments);
        } else {
            self.list(SiteKind::Decorator, written(expression), expression.range());
            self.visit_expr(expression);
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr) {
        if let Expr::Call(call) = expr {
            self.list(SiteKind::Callee, written(&call.func), call.range());
        } else if let Expr::Attribute(attribute) = expr
            && attribute.ctx.is_store()
        {
            self.list(SiteKind::WriteReceiver, written(expr), attribute.range());
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
                self.list(SiteKind::HandlerType, written(caught), caught.range());
            }
        }
        visitor::walk_except_handler(self, handler);
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
