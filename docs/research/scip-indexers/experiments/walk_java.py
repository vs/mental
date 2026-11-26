"""Part J: list the reference sites of Java files with tree-sitter's Java grammar.

The site kinds are the specification's (6.1), mapped to Java as the plan's table says. Each site
is returned with where its last name starts (0-based line, byte column), the name, and a
`form` that tells apart shapes the index may answer differently:

- import: `single`, `static` or `on-demand` (`import p.*;`, asked at the package's last name);
- decorator: an annotation;
- callee: `method` (`x.h()`), `new` (`new C<>()`, asked at the type), `this` or `super`
  (`this(…)`, `super(…)`, asked at the keyword);
- base-class: a type after `extends` or `implements`;
- write-receiver: `field` (`this.f = …`, `x.f = …`) or `bare` (`g = …`: a field or a local
  variable; only the index says which);
- handler-type: a type a `catch` names, one per element of `A | B`;
- escape: `method-ref` (`this::k`), `lambda` (no name: asked at its start), `argument` (a name or
  field handed to a call), `override` (a method declaration: an escape when it overrides a
  method of outside code, which only the index's relationships say).
"""

import dataclasses

import tree_sitter
import tree_sitter_java

PARSER = tree_sitter.Parser(tree_sitter.Language(tree_sitter_java.language()))
CALLS = {"method_invocation", "object_creation_expression", "explicit_constructor_invocation"}


@dataclasses.dataclass(frozen=True)
class Site:
    kind: str
    form: str
    line: int
    column: int  # bytes
    name: str


def _last_identifier(node):
    """The node that names a type or a dotted name: its last identifier."""
    if node.type in ("identifier", "type_identifier", "this", "super"):
        return node
    if node.type == "generic_type":
        return _last_identifier(node.named_children[0])
    if node.type in ("scoped_identifier", "scoped_type_identifier"):
        return _last_identifier(node.named_children[-1])
    if node.type in ("annotated_type", "array_type"):
        named = [c for c in node.named_children if "annotation" not in c.type]
        return _last_identifier(named[0])
    if node.type == "field_access":
        return node.child_by_field_name("field")
    return None


def sites(source: bytes):
    tree = PARSER.parse(source)
    out = []

    def add(kind, form, node):
        if node is not None:
            out.append(Site(kind, form, node.start_point[0], node.start_point[1],
                            node.text.decode(errors="replace")))

    stack = [tree.root_node]
    while stack:
        node = stack.pop()
        stack.extend(reversed(node.children))
        t = node.type
        if t == "import_declaration":
            name = next(c for c in node.named_children
                        if c.type in ("identifier", "scoped_identifier"))
            texts = [c.type for c in node.children]
            form = ("on-demand" if "asterisk" in texts
                    else "static" if "static" in texts else "single")
            add("import", form, _last_identifier(name))
        elif t in ("annotation", "marker_annotation"):
            add("decorator", "annotation", _last_identifier(node.child_by_field_name("name")))
        elif t == "method_invocation":
            add("callee", "method", node.child_by_field_name("name"))
        elif t == "object_creation_expression":
            add("callee", "new", _last_identifier(node.child_by_field_name("type")))
        elif t == "explicit_constructor_invocation":
            ctor = node.child_by_field_name("constructor")
            add("callee", ctor.type, ctor)
        elif t == "superclass":
            add("base-class", "extends", _last_identifier(node.named_children[0]))
        elif t in ("super_interfaces", "extends_interfaces"):
            for type_list in node.named_children:
                for ty in type_list.named_children:
                    add("base-class", "implements", _last_identifier(ty))
        elif t in ("assignment_expression", "update_expression"):
            target = (node.child_by_field_name("left") if t == "assignment_expression"
                      else node.named_children[0])
            if target is not None and target.type == "field_access":
                add("write-receiver", "field", target.child_by_field_name("field"))
            elif target is not None and target.type == "identifier":
                add("write-receiver", "bare", target)
        elif t == "catch_type":
            for ty in node.named_children:
                add("handler-type", "catch", _last_identifier(ty))
        elif t == "argument_list" and node.parent is not None and node.parent.type in CALLS:
            for arg in node.named_children:
                if arg.type == "method_reference":
                    # `Foo::new` is one occurrence from the type on; `this::k` is asked at `k`.
                    last = arg.children[-1]
                    add("escape", "method-ref", arg if last.type == "new" else last)
                elif arg.type == "lambda_expression":
                    out.append(Site("escape", "lambda", arg.start_point[0], arg.start_point[1],
                                    ""))
                elif arg.type in ("identifier", "field_access"):
                    add("escape", "argument", _last_identifier(arg))
        elif t == "method_declaration":
            add("escape", "override", node.child_by_field_name("name"))
    return out
