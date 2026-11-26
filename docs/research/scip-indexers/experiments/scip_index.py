"""Read a SCIP index into plain Python data, and parse SCIP symbols.

The protobuf module is generated from scip.proto into <work-dir>/gen (see README); this module
adds that folder to the path when it is loaded with `load(work, path)`.
"""

import collections
import dataclasses
import sys

LOCAL = "local"
ROLE_DEFINITION, ROLE_IMPORT, ROLE_WRITE, ROLE_READ = 0x1, 0x2, 0x4, 0x8
ENCODINGS = {0: "unspecified", 1: "utf-8", 2: "utf-16", 3: "utf-32"}


@dataclasses.dataclass(frozen=True)
class Occurrence:
    line: int  # 0-based
    start: int  # in the document's encoding
    end_line: int
    end: int
    symbol: str
    roles: int
    diagnostics: tuple  # (severity, code, message)


@dataclasses.dataclass
class Index:
    tool: str
    documents: dict  # relative path -> (encoding name, [Occurrence])
    symbols: dict  # symbol -> SymbolInformation message (documents and external symbols)


def _range(occ):
    kind = occ.WhichOneof("typed_range")
    if kind == "single_line_range":
        r = occ.single_line_range
        return r.line, r.start_character, r.line, r.end_character
    if kind == "multi_line_range":
        r = occ.multi_line_range
        return r.start_line, r.start_character, r.end_line, r.end_character
    r = list(occ.range)
    if len(r) == 3:
        return r[0], r[1], r[0], r[2]
    assert len(r) == 4, f"a SCIP range has three or four elements, not {r}"
    return tuple(r)


def load(work, path):
    sys.path.insert(0, str(work / "gen"))
    import scip_pb2  # noqa: PLC0415 - generated at run time from the pinned scip.proto

    index = scip_pb2.Index()
    index.ParseFromString(path.read_bytes())
    documents, symbols = {}, {}
    for doc in index.documents:
        occurrences = []
        for occ in doc.occurrences:
            line, start, end_line, end = _range(occ)
            diagnostics = tuple((d.severity, d.code, d.message) for d in occ.diagnostics)
            occurrences.append(Occurrence(line, start, end_line, end, occ.symbol,
                                          occ.symbol_roles, diagnostics))
        documents[doc.relative_path] = (ENCODINGS[doc.position_encoding], occurrences)
        for info in doc.symbols:
            symbols[info.symbol] = info
    for info in index.external_symbols:
        symbols.setdefault(info.symbol, info)
    tool = f"{index.metadata.tool_info.name} {index.metadata.tool_info.version}"
    return Index(tool, documents, symbols)


@dataclasses.dataclass(frozen=True)
class Symbol:
    manager: str
    package: str
    version: str
    descriptors: tuple  # (name, suffix): suffix is one of / # . ( [ : !

    @property
    def namespace(self):
        return ".".join(n for n, s in self.descriptors if s == "/")

    @property
    def path(self):
        """The descriptors after the namespaces, joined with dots; parameters left out."""
        return ".".join(n for n, s in self.descriptors if s in "#.(:!")

    @property
    def last_suffix(self):
        return self.descriptors[-1][1] if self.descriptors else ""


def _name(text, i):
    """A descriptor name at text[i:]: a simple identifier or a backtick-quoted one."""
    if text[i] == "`":
        out, i = [], i + 1
        while True:
            if text[i] == "`":
                if i + 1 < len(text) and text[i + 1] == "`":
                    out.append("`")
                    i += 2
                    continue
                return "".join(out), i + 1
            out.append(text[i])
            i += 1
    j = i
    while j < len(text) and (text[j].isalnum() or text[j] in "_+-$"):
        j += 1
    return text[i:j], j


def parse_symbol(symbol):
    """None for a local symbol; else the package and descriptors of a global one."""
    if not symbol or symbol.startswith(LOCAL + " "):
        return None
    parts, i = [], 0
    for _ in range(4):  # scheme, manager, package name, version: space separated, "  " escapes
        j = i
        while j < len(symbol):
            if symbol[j] == " ":
                if j + 1 < len(symbol) and symbol[j + 1] == " ":
                    j += 2
                    continue
                break
            j += 1
        parts.append(symbol[i:j].replace("  ", " "))
        i = j + 1
    rest, descriptors = symbol[i:], []
    k = 0
    while k < len(rest):
        if rest[k] == "(":  # a parameter: (name)
            name, k = _name(rest, k + 1)
            assert rest[k] == ")", symbol
            descriptors.append((name, "p"))
            k += 1
            continue
        if rest[k] == "[":  # a type parameter: [name]
            name, k = _name(rest, k + 1)
            assert rest[k] == "]", symbol
            descriptors.append((name, "t"))
            k += 1
            continue
        name, k = _name(rest, k)
        suffix = rest[k]
        if suffix == "(":  # a method: name(disambiguator).
            close = rest.index(").", k)
            descriptors.append((name, "("))
            k = close + 2
            continue
        assert suffix in "/#.:!", f"unknown descriptor suffix {suffix!r} in {symbol}"
        descriptors.append((name, suffix))
        k += 1
    _, manager, package, version = parts
    return Symbol(manager, package, version, tuple(descriptors))


def by_line(index):
    """Per document, the occurrences by their first line."""
    out = {}
    for path, (encoding, occurrences) in index.documents.items():
        lines = collections.defaultdict(list)
        for occ in occurrences:
            lines[occ.line].append(occ)
        out[path] = (encoding, lines)
    return out
