// Part J, run 2: what javac's own API answers for each name of a project's main sources.
//
// Usage: java --add-exports jdk.compiler/com.sun.tools.javac.code=ALL-UNNAMED \
//            JavacQuery.java <repository root> <argfile> <out.jsonl>
//
// <argfile> holds one javac option or source file per line. Writes one JSON object per line:
// a "ref" for each name (identifier, member select, `new`, method reference, lambda), an "arg"
// for each name handed to a call, a "method" for each method declaration with what it
// overrides, a "diag" for each diagnostic, and a last "done". Positions are 1-based lines and
// 0-based UTF-16 columns, as part J's pairing uses.

import com.sun.source.tree.*;
import com.sun.source.util.*;
import com.sun.tools.javac.code.Symbol;
import java.io.*;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.*;
import java.util.*;
import javax.lang.model.element.*;
import javax.lang.model.type.*;
import javax.lang.model.util.*;
import javax.tools.*;

public class JavacQuery {
    static String root;
    static Trees trees;
    static Elements elements;
    static Types types;
    static SourcePositions positions;
    static PrintWriter out;

    public static void main(String[] argv) throws Exception {
        root = Path.of(argv[0]).toRealPath().toUri().toString();
        List<String> options = new ArrayList<>();
        List<File> files = new ArrayList<>();
        for (String line : Files.readAllLines(Path.of(argv[1]))) {
            if (line.endsWith(".java")) files.add(new File(line));
            else if (!line.isEmpty()) options.add(line);
        }
        out = new PrintWriter(Files.newBufferedWriter(Path.of(argv[2]), StandardCharsets.UTF_8));
        long start = System.nanoTime();
        JavaCompiler compiler = ToolProvider.getSystemJavaCompiler();
        DiagnosticCollector<JavaFileObject> diagnostics = new DiagnosticCollector<>();
        StandardJavaFileManager fm = compiler.getStandardFileManager(diagnostics, null, StandardCharsets.UTF_8);
        JavacTask task = (JavacTask) compiler.getTask(null, fm, diagnostics, options, null,
                fm.getJavaFileObjectsFromFiles(files));
        trees = Trees.instance(task);
        elements = task.getElements();
        types = task.getTypes();
        positions = trees.getSourcePositions();
        List<CompilationUnitTree> units = new ArrayList<>();
        task.parse().forEach(units::add);
        task.analyze();
        long analyzed = System.nanoTime();
        for (CompilationUnitTree unit : units) new Scanner(unit).scan(unit, null);
        for (Diagnostic<? extends JavaFileObject> d : diagnostics.getDiagnostics()) {
            if (d.getKind() != Diagnostic.Kind.ERROR && d.getKind() != Diagnostic.Kind.MANDATORY_WARNING) continue;
            Map<String, Object> r = record("diag");
            r.put("path", d.getSource() == null ? null : relative(d.getSource().toUri()));
            r.put("line", d.getLineNumber());
            r.put("kind", d.getKind().toString());
            r.put("code", d.getCode());
            r.put("message", d.getMessage(Locale.ROOT).lines().findFirst().orElse(""));
            emit(r);
        }
        Map<String, Object> done = record("done");
        done.put("units", units.size());
        done.put("analyze_ms", (analyzed - start) / 1_000_000);
        done.put("total_ms", (System.nanoTime() - start) / 1_000_000);
        emit(done);
        out.close();
    }

    static final class Scanner extends TreePathScanner<Void, Void> {
        final CompilationUnitTree unit;
        final String path;
        final LineMap lines;
        final CharSequence text;

        Scanner(CompilationUnitTree unit) throws IOException {
            this.unit = unit;
            this.path = relative(unit.getSourceFile().toUri());
            this.lines = unit.getLineMap();
            this.text = unit.getSourceFile().getCharContent(true);
        }

        /** A record at a source position; null for a tree javac made up (an implicit `super()`). */
        Map<String, Object> at(String type, long pos) {
            if (pos < 0) return null;
            Map<String, Object> r = record(type);
            long line = lines.getLineNumber(pos);
            r.put("path", path);
            r.put("line", line);
            r.put("col", pos - lines.getStartPosition(line));
            return r;
        }

        long start(Tree t) { return positions.getStartPosition(unit, t); }

        long end(Tree t) { return positions.getEndPosition(unit, t); }

        /** Where the last name of a name, a type or a member select starts. */
        long nameStart(Tree t) {
            switch (t.getKind()) {
                case MEMBER_SELECT: return end(t) - ((MemberSelectTree) t).getIdentifier().length();
                case PARAMETERIZED_TYPE: return nameStart(((ParameterizedTypeTree) t).getType());
                case ANNOTATED_TYPE: return nameStart(((AnnotatedTypeTree) t).getUnderlyingType());
                default: return start(t);
            }
        }

        void ref(String tree, long pos, TreePath p) {
            Map<String, Object> r = at("ref", pos);
            if (r == null) return;
            r.put("tree", tree);
            describe(r, "", trees.getElement(p));
            TypeMirror type = trees.getTypeMirror(p);
            r.put("type", typeName(type));
            emit(r);
        }

        @Override public Void visitIdentifier(IdentifierTree t, Void v) {
            ref("identifier", start(t), getCurrentPath());
            return super.visitIdentifier(t, v);
        }

        @Override public Void visitMemberSelect(MemberSelectTree t, Void v) {
            ref("select", nameStart(t), getCurrentPath());
            return super.visitMemberSelect(t, v);
        }

        @Override public Void visitImport(ImportTree t, Void v) {
            if (t.isStatic() && t.getQualifiedIdentifier() instanceof MemberSelectTree select) {
                // javac keeps no element on a static import's member: the members it imports are
                // those of the qualifier's class with that name, which may be several overloads.
                Map<String, Object> r = at("static-import", nameStart(select));
                if (r == null) return super.visitImport(t, v);
                Element owner = trees.getElement(new TreePath(new TreePath(getCurrentPath(), select), select.getExpression()));
                String name = select.getIdentifier().toString();
                List<Element> members = new ArrayList<>();
                if (owner instanceof TypeElement type) {
                    for (Element m : elements.getAllMembers(type)) {
                        if (m.getSimpleName().contentEquals(name) && m.getModifiers().contains(Modifier.STATIC)) members.add(m);
                    }
                }
                r.put("members", members.size());
                describe(r, "", members.isEmpty() ? null : members.get(0));
                emit(r);
            }
            return super.visitImport(t, v);
        }

        @Override public Void visitNewClass(NewClassTree t, Void v) {
            ref("new", nameStart(t.getIdentifier()), getCurrentPath());
            arguments(t.getArguments(), trees.getElement(getCurrentPath()));
            return super.visitNewClass(t, v);
        }

        @Override public Void visitMethodInvocation(MethodInvocationTree t, Void v) {
            arguments(t.getArguments(), trees.getElement(new TreePath(getCurrentPath(), t.getMethodSelect())));
            return super.visitMethodInvocation(t, v);
        }

        void arguments(List<? extends ExpressionTree> args, Element callee) {
            for (ExpressionTree a : args) {
                if (a.getKind() != Tree.Kind.IDENTIFIER && a.getKind() != Tree.Kind.MEMBER_SELECT) continue;
                TreePath p = new TreePath(getCurrentPath(), a);
                Map<String, Object> r = at("arg", nameStart(a));
                if (r == null) continue;
                describe(r, "callee_", callee);
                TypeMirror type = trees.getTypeMirror(p);
                r.put("type", typeName(type));
                if (type != null && type.getKind() != TypeKind.ERROR) {
                    Element te = types.asElement(type);
                    r.put("type_origin", te == null ? type.getKind().isPrimitive() ? "primitive" : type.getKind().toString().toLowerCase() : origin(te));
                }
                emit(r);
            }
        }

        @Override public Void visitMemberReference(MemberReferenceTree t, Void v) {
            boolean ctor = t.getName().contentEquals("<init>");
            ref("method-ref", ctor ? start(t) : end(t) - t.getName().length(), getCurrentPath());
            return super.visitMemberReference(t, v);
        }

        @Override public Void visitLambdaExpression(LambdaExpressionTree t, Void v) {
            ref("lambda", start(t), getCurrentPath());
            return super.visitLambdaExpression(t, v);
        }

        @Override public Void visitMethod(MethodTree t, Void v) {
            Element e = trees.getElement(getCurrentPath());
            if (e instanceof ExecutableElement m && e.getKind() == ElementKind.METHOD && start(t) >= 0) {
                Tree after = t.getReturnType() != null ? t.getReturnType() : t.getModifiers();
                int from = (int) end(after);
                String name = t.getName().toString();
                int pos = indexOfName(text, name, from);
                Map<String, Object> r = at("method", pos);
                describe(r, "", m);
                List<Map<String, Object>> overridden = new ArrayList<>();
                TypeElement owner = (TypeElement) m.getEnclosingElement();
                for (TypeElement s : supertypes(owner)) {
                    for (Element o : s.getEnclosedElements()) {
                        if (o instanceof ExecutableElement other && other.getSimpleName().equals(m.getSimpleName())
                                && elements.overrides(m, other, owner)) {
                            Map<String, Object> d = new LinkedHashMap<>();
                            describe(d, "", other);
                            overridden.add(d);
                        }
                    }
                }
                r.put("overrides", overridden);
                emit(r);
            }
            return super.visitMethod(t, v);
        }
    }

    /** The erased type, `<error>` when javac could not attribute it, or the kind of a non-type. */
    static String typeName(TypeMirror type) {
        if (type == null) return null;
        return switch (type.getKind()) {
            case ERROR -> "<error>";
            case PACKAGE, MODULE, EXECUTABLE, NONE, VOID, OTHER -> type.getKind().toString().toLowerCase();
            default -> types.erasure(type).toString();
        };
    }

    static int indexOfName(CharSequence text, String name, int from) {
        String s = text.toString();
        for (int i = s.indexOf(name, from); i >= 0; i = s.indexOf(name, i + 1)) {
            boolean before = i == 0 || !Character.isJavaIdentifierPart(s.charAt(i - 1));
            int j = i + name.length();
            boolean after = j >= s.length() || !Character.isJavaIdentifierPart(s.charAt(j));
            if (before && after) return i;
        }
        return from;
    }

    static List<TypeElement> supertypes(TypeElement type) {
        List<TypeElement> found = new ArrayList<>();
        Deque<TypeMirror> todo = new ArrayDeque<>(types.directSupertypes(type.asType()));
        Set<String> seen = new HashSet<>();
        while (!todo.isEmpty()) {
            TypeMirror t = todo.pop();
            if (types.asElement(t) instanceof TypeElement e && seen.add(e.getQualifiedName().toString())) {
                found.add(e);
                todo.addAll(types.directSupertypes(t));
            }
        }
        return found;
    }

    /** The element's symbol, in SCIP's descriptor form without overload numbers, its kind and origin. */
    static void describe(Map<String, Object> r, String prefix, Element e) {
        if (e == null) {
            r.put(prefix + "symbol", null);
            return;
        }
        boolean error = e.getKind() == ElementKind.OTHER || e.asType().getKind() == TypeKind.ERROR;
        r.put(prefix + "symbol", error ? "<error>" : symbol(e));
        r.put(prefix + "kind", e.getKind().toString());
        if (!error) r.put(prefix + "origin", origin(e));
    }

    static String symbol(Element e) {
        StringBuilder b = new StringBuilder();
        for (Element x = e; x != null; x = x.getEnclosingElement()) {
            switch (x.getKind()) {
                case PACKAGE -> {
                    String p = ((PackageElement) x).getQualifiedName().toString();
                    b.insert(0, p.isEmpty() ? "" : p.replace('.', '/') + "/");
                    return b.toString();
                }
                case MODULE -> { return b.toString(); }
                case LOCAL_VARIABLE, PARAMETER, EXCEPTION_PARAMETER, RESOURCE_VARIABLE, TYPE_PARAMETER,
                     BINDING_VARIABLE -> { return "local"; }
                case METHOD, CONSTRUCTOR, STATIC_INIT, INSTANCE_INIT -> {
                    if (x != e) return "local";  // something declared inside a method body
                    b.insert(0, x.getSimpleName() + "().");
                }
                case FIELD, ENUM_CONSTANT, RECORD_COMPONENT -> b.insert(0, x.getSimpleName() + ".");
                default -> {
                    if (x instanceof TypeElement t) {
                        if (t.getNestingKind() == NestingKind.ANONYMOUS || t.getNestingKind() == NestingKind.LOCAL) return "local";
                        b.insert(0, t.getSimpleName() + "#");
                    } else {
                        b.insert(0, x.getSimpleName() + ".");
                    }
                }
            }
        }
        return b.toString();
    }

    /** source (this compilation), repository (another module's classes), jdk, or the jar's URI. */
    static String origin(Element e) {
        Element x = e;
        while (x != null && !(x instanceof TypeElement) && !(x instanceof PackageElement)) x = x.getEnclosingElement();
        if (x == null) return e.getKind().toString().toLowerCase();  // a module: no class or package
        if (x instanceof PackageElement) return "package";
        TypeElement outer = (TypeElement) x;
        while (outer.getEnclosingElement() instanceof TypeElement t) outer = t;
        if (trees.getPath(outer) != null) return "source";
        JavaFileObject file = ((Symbol.ClassSymbol) outer).classfile;
        if (file == null) return "unknown";
        String uri = file.toUri().toString();
        if (uri.startsWith("jrt:")) return "jdk";
        String inner = uri.startsWith("jar:") ? uri.substring(4) : uri;
        if (inner.startsWith(root)) return "repository";
        return inner.contains("!/") ? inner.substring(0, inner.indexOf("!/")) : inner;
    }

    static String relative(URI uri) {
        String s = uri.toString();
        return s.startsWith(root) ? s.substring(root.length()) : s;
    }

    static Map<String, Object> record(String type) {
        Map<String, Object> r = new LinkedHashMap<>();
        r.put("t", type);
        return r;
    }

    static void emit(Map<String, Object> r) {
        StringBuilder b = new StringBuilder("{");
        for (Map.Entry<String, Object> en : r.entrySet()) {
            if (b.length() > 1) b.append(',');
            b.append(json(en.getKey())).append(':').append(value(en.getValue()));
        }
        out.println(b.append('}'));
    }

    @SuppressWarnings("unchecked")
    static String value(Object v) {
        if (v == null) return "null";
        if (v instanceof Number || v instanceof Boolean) return v.toString();
        if (v instanceof List<?> list) {
            StringJoiner j = new StringJoiner(",", "[", "]");
            for (Object o : list) j.add(value(o));
            return j.toString();
        }
        if (v instanceof Map<?, ?> m) {
            StringJoiner j = new StringJoiner(",", "{", "}");
            for (Map.Entry<String, Object> en : ((Map<String, Object>) m).entrySet()) j.add(json(en.getKey()) + ":" + value(en.getValue()));
            return j.toString();
        }
        return json(v.toString());
    }

    static String json(String s) {
        StringBuilder b = new StringBuilder("\"");
        for (char c : s.toCharArray()) {
            switch (c) {
                case '"' -> b.append("\\\"");
                case '\\' -> b.append("\\\\");
                case '\n' -> b.append("\\n");
                case '\r' -> b.append("\\r");
                case '\t' -> b.append("\\t");
                default -> {
                    if (c < 0x20) b.append(String.format("\\u%04x", (int) c));
                    else b.append(c);
                }
            }
        }
        return b.append('"').toString();
    }
}
