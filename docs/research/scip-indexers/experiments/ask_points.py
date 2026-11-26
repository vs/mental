"""Where `extract` asks the checker for each site kind, found with Python's own parser.

The site records keep whole lines, not columns, so the pairing needs the column of the name a
site is about. This walk follows extract/src/python/sites.rs: a call's callee (a decorator when
the call is one), a non-call decorator, each imported name, each positional base, each
attribute assigned to, each type an `except` names, each name or attribute handed to a call.
It returns, per 0-based line, the code-point column of each site's last name with its kind.
"""

import ast
import collections


def _last_name(node):
    """Start (line, byte column) of the last name of a Name or Attribute; None otherwise."""
    if isinstance(node, ast.Name):
        return node.lineno, node.col_offset
    if isinstance(node, ast.Attribute):
        # The attribute's name ends the node; ruff's tree has its own range, ast's has not.
        return node.end_lineno, node.end_col_offset - len(node.attr.encode())
    return None


def ask_points(text):
    """{(line, code-point column): {kind}} for one file; lines and columns 0-based."""
    tree = ast.parse(text)
    lines = text.splitlines()
    points = collections.defaultdict(set)

    def add(kind, pos):
        if pos is None:
            return
        line, byte_col = pos
        raw = lines[line - 1].encode()[:byte_col]
        points[(line - 1, len(raw.decode()))].add(kind)

    def escapes(call):
        for value in [*call.args, *(k.value for k in call.keywords)]:
            if isinstance(value, (ast.Name, ast.Attribute)):
                add("escape", _last_name(value))

    decorator_calls = set()
    for node in ast.walk(tree):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef, ast.ClassDef)):
            for dec in node.decorator_list:
                if isinstance(dec, ast.Call):
                    decorator_calls.add(id(dec))
                    add("decorator", _last_name(dec.func))
                else:
                    add("decorator", _last_name(dec))
        if isinstance(node, ast.ClassDef):
            for base in node.bases:
                add("base-class", _last_name(base))
        elif isinstance(node, ast.Call):
            if id(node) not in decorator_calls:
                add("callee", _last_name(node.func))
            escapes(node)
        elif isinstance(node, ast.Attribute) and isinstance(node.ctx, ast.Store):
            add("write-receiver", _last_name(node))
        elif isinstance(node, ast.ExceptHandler) and node.type is not None:
            types = node.type.elts if isinstance(node.type, ast.Tuple) else [node.type]
            for t in types:
                add("handler-type", _last_name(t))
        elif isinstance(node, (ast.Import, ast.ImportFrom)):
            for alias in node.names:
                if alias.name == "*":
                    continue
                # The whole dotted name and its last segment: indexers place it either way.
                add("import", (alias.lineno, alias.col_offset))
                dot = alias.name.rfind(".")
                if dot >= 0:
                    add("import", (alias.lineno, alias.col_offset + dot + 1))
    return points


def to_encoding(line_text, codepoint_col, encoding):
    """A code-point column in the index's encoding."""
    if encoding == "utf-8":
        return len(line_text[:codepoint_col].encode())
    if encoding in ("utf-16", "unspecified"):  # scip-python is TypeScript: UTF-16 strings
        return len(line_text[:codepoint_col].encode("utf-16-le")) // 2
    return codepoint_col
