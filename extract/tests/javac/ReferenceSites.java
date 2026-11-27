// The reference sites of a Maven project's main sources, as javac answers
// them: every site of the Java site rules (docs/specification.md, 6.7) with
// the element javac binds it to, written as the model's canonical site.jsonl
// and the sites' lines of anchors.jsonl. tests/java_reference.rs runs it.
//
// Usage: java ReferenceSites.java <project> <unit id> <dependency list> <out dir>
// where the dependency list is the output of
// `mvn dependency:list -DoutputAbsoluteArtifactFilename=true`.

import com.sun.source.tree.AnnotationTree;
import com.sun.source.tree.AssignmentTree;
import com.sun.source.tree.CatchTree;
import com.sun.source.tree.ClassTree;
import com.sun.source.tree.CompilationUnitTree;
import com.sun.source.tree.CompoundAssignmentTree;
import com.sun.source.tree.ExpressionTree;
import com.sun.source.tree.IdentifierTree;
import com.sun.source.tree.ImportTree;
import com.sun.source.tree.LambdaExpressionTree;
import com.sun.source.tree.MemberReferenceTree;
import com.sun.source.tree.MemberSelectTree;
import com.sun.source.tree.MethodInvocationTree;
import com.sun.source.tree.MethodTree;
import com.sun.source.tree.NewClassTree;
import com.sun.source.tree.ParameterizedTypeTree;
import com.sun.source.tree.Tree;
import com.sun.source.tree.UnaryTree;
import com.sun.source.tree.UnionTypeTree;
import com.sun.source.tree.VariableTree;
import com.sun.source.util.JavacTask;
import com.sun.source.util.SourcePositions;
import com.sun.source.util.TreePath;
import com.sun.source.util.TreePathScanner;
import com.sun.source.util.Trees;
import java.io.IOException;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayDeque;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import java.util.stream.Collectors;
import javax.lang.model.element.Element;
import javax.lang.model.element.ElementKind;
import javax.lang.model.element.ExecutableElement;
import javax.lang.model.element.Modifier;
import javax.lang.model.element.PackageElement;
import javax.lang.model.element.TypeElement;
import javax.lang.model.type.TypeKind;
import javax.lang.model.type.TypeMirror;
import javax.lang.model.util.Elements;
import javax.lang.model.util.Types;
import javax.tools.Diagnostic;
import javax.tools.DiagnosticCollector;
import javax.tools.JavaFileObject;
import javax.tools.StandardJavaFileManager;
import javax.tools.StandardLocation;
import javax.tools.ToolProvider;

public class ReferenceSites {

    record Site(String path, long start, String base, String kind, String outcome, int first, int last) {
    }

    final Path project;
    final String unit;
    final Map<Path, String> coordinates;
    final StandardJavaFileManager files;
    Trees trees;
    Elements elements;
    Types types;
    final List<Site> sites = new ArrayList<>();
    final Map<String, List<String>> sourceLines = new HashMap<>();

    ReferenceSites(Path project, String unit, Map<Path, String> coordinates, StandardJavaFileManager files) {
        this.project = project;
        this.unit = unit;
        this.coordinates = coordinates;
        this.files = files;
    }

    public static void main(String[] args) throws IOException {
        Path project = Path.of(args[0]).toRealPath();
        Map<Path, String> coordinates = new HashMap<>();
        List<Path> classpath = new ArrayList<>();
        for (String line : Files.readAllLines(Path.of(args[2]))) {
            String[] parts = line.strip().split(" -- ", 2)[0].split(":");
            if (parts.length < 6) {
                continue;
            }
            Path jar = Path.of(parts[parts.length - 1]).toRealPath();
            coordinates.put(jar, parts[0] + ":" + parts[1]);
            String scope = parts[parts.length - 2];
            if (!scope.equals("runtime") && !scope.equals("test")) {
                classpath.add(jar);
            }
        }
        List<Path> sources;
        try (var walk = Files.walk(project.resolve("src/main/java"))) {
            sources = walk.filter(path -> path.toString().endsWith(".java")).sorted().toList();
        }
        var javac = ToolProvider.getSystemJavaCompiler();
        var files = javac.getStandardFileManager(null, null, StandardCharsets.UTF_8);
        files.setLocationFromPaths(StandardLocation.CLASS_PATH, classpath);
        var diagnostics = new DiagnosticCollector<JavaFileObject>();
        var task = (JavacTask) javac.getTask(null, files, diagnostics,
                List.of("-proc:none", "-implicit:none", "-encoding", "UTF-8"), null,
                files.getJavaFileObjectsFromPaths(sources));
        Iterable<? extends CompilationUnitTree> units = task.parse();
        task.analyze();
        var errors = diagnostics.getDiagnostics().stream()
                .filter(d -> d.getKind() == Diagnostic.Kind.ERROR).toList();
        if (!errors.isEmpty()) {
            errors.forEach(System.err::println);
            System.err.println("The reference is of code that compiles: fix the errors above.");
            System.exit(1);
        }
        var walk = new ReferenceSites(project, args[1], coordinates, files);
        walk.trees = Trees.instance(task);
        walk.elements = task.getElements();
        walk.types = task.getTypes();
        for (CompilationUnitTree cu : units) {
            walk.scan(cu);
        }
        walk.write(Path.of(args[3]));
    }

    void scan(CompilationUnitTree cu) throws IOException {
        Path file = Path.of(cu.getSourceFile().toUri()).toRealPath();
        String path = project.relativize(file).toString();
        sourceLines.put(path, List.of(Files.readString(file).split("\n", -1)));
        SourcePositions positions = trees.getSourcePositions();
        new TreePathScanner<Void, Void>() {

            TreePath at(Tree tree) {
                return new TreePath(getCurrentPath(), tree);
            }

            void add(TreePath where, Tree spanned, String kind, String name, Element target) {
                long start = positions.getStartPosition(cu, spanned);
                long end = positions.getEndPosition(cu, spanned);
                if (start < 0 || end < 0) {
                    return;
                }
                add(where, start, end, kind, name, target);
            }

            void add(TreePath where, long start, long end, String kind, String name, Element target) {
                var lines = cu.getLineMap();
                String base = slug(String.join(" ", List.of(scope(where), kind, name).stream()
                        .filter(part -> !part.isEmpty()).toList()));
                sites.add(new Site(path, start, base, kind, outcome(target),
                        (int) lines.getLineNumber(start), (int) lines.getLineNumber(end - 1)));
            }

            String scope(TreePath where) {
                for (TreePath p = where; p != null; p = p.getParentPath()) {
                    if (p.getLeaf() instanceof MethodTree method) {
                        TypeElement owner = (TypeElement) trees.getElement(p).getEnclosingElement();
                        return owner.getQualifiedName() + "." + method.getName();
                    }
                    if (p.getLeaf() instanceof ClassTree) {
                        return ((TypeElement) trees.getElement(p)).getQualifiedName().toString();
                    }
                }
                String name = cu.getSourceFile().getName();
                String stem = Path.of(name).getFileName().toString().replaceFirst("\\.java$", "");
                return cu.getPackageName() == null ? stem : cu.getPackageName() + "." + stem;
            }

            Element typeOf(TreePath path) {
                TypeMirror type = trees.getTypeMirror(path);
                return type == null ? null : types.asElement(type);
            }

            @Override
            public Void visitImport(ImportTree tree, Void unused) {
                Tree named = tree.getQualifiedIdentifier();
                String name = named.toString();
                if (name.endsWith(".*")) {
                    named = ((MemberSelectTree) named).getExpression();
                }
                add(getCurrentPath(), tree, "import", name, trees.getElement(at(named)));
                return null;
            }

            @Override
            public Void visitAnnotation(AnnotationTree tree, Void unused) {
                add(getCurrentPath(), tree, "decorator", written(tree.getAnnotationType()),
                        trees.getElement(at(tree.getAnnotationType())));
                return super.visitAnnotation(tree, unused);
            }

            @Override
            public Void visitMethod(MethodTree tree, Void unused) {
                var method = (ExecutableElement) trees.getElement(getCurrentPath());
                if (elements.getOrigin(method) != Elements.Origin.EXPLICIT) {
                    return null;
                }
                ExecutableElement overridden = outsideOverridden(method);
                if (overridden != null) {
                    long from = tree.getReturnType() == null ? positions.getStartPosition(cu, tree)
                            : positions.getEndPosition(cu, tree.getReturnType());
                    String name = tree.getName().toString();
                    long start = String.join("\n", sourceLines.get(path)).indexOf(name, (int) from);
                    add(getCurrentPath(), start, start + name.length(), "escape", name, overridden);
                }
                return super.visitMethod(tree, unused);
            }

            @Override
            public Void visitClass(ClassTree tree, Void unused) {
                List<Tree> bases = new ArrayList<>();
                if (tree.getExtendsClause() != null) {
                    bases.add(tree.getExtendsClause());
                }
                bases.addAll(tree.getImplementsClause());
                for (Tree base : bases) {
                    add(at(base), base, "base-class", written(base), typeOf(at(base)));
                }
                return super.visitClass(tree, unused);
            }

            @Override
            public Void visitMethodInvocation(MethodInvocationTree tree, Void unused) {
                Element callee = trees.getElement(getCurrentPath());
                add(getCurrentPath(), tree, "callee", written(tree.getMethodSelect()), callee);
                escapes(tree.getArguments(), callee);
                return super.visitMethodInvocation(tree, unused);
            }

            @Override
            public Void visitNewClass(NewClassTree tree, Void unused) {
                Tree parent = getCurrentPath().getParentPath().getLeaf();
                boolean enumConstant = parent instanceof VariableTree
                        && trees.getElement(getCurrentPath().getParentPath()).getKind()
                        == ElementKind.ENUM_CONSTANT;
                if (!enumConstant) {
                    Element constructor = trees.getElement(getCurrentPath());
                    add(getCurrentPath(), tree, "callee", written(tree.getIdentifier()), constructor);
                    escapes(tree.getArguments(), constructor);
                }
                return super.visitNewClass(tree, unused);
            }

            void escapes(List<? extends ExpressionTree> arguments, Element callee) {
                if (callee == null || inRepository(callee)) {
                    return;
                }
                for (ExpressionTree argument : arguments) {
                    TreePath path = at(argument);
                    if (argument instanceof LambdaExpressionTree) {
                        add(path, argument, "escape", "", functionalMethod(typeOf(path)));
                    } else if (argument instanceof MemberReferenceTree) {
                        add(path, argument, "escape", written(argument), trees.getElement(path));
                    } else if (argument instanceof MemberSelectTree select
                            && select.getIdentifier().contentEquals("class")) {
                        Element type = trees.getElement(at(select.getExpression()));
                        if (type != null && inRepository(type)) {
                            add(path, argument, "escape", written(argument), type);
                        }
                    } else if (argument instanceof IdentifierTree || argument instanceof MemberSelectTree) {
                        Element type = typeOf(path);
                        if (type instanceof TypeElement && inRepository(type)) {
                            add(path, argument, "escape", written(argument), type);
                        }
                    }
                }
            }

            @Override
            public Void visitAssignment(AssignmentTree tree, Void unused) {
                writeReceiver(tree.getVariable());
                return super.visitAssignment(tree, unused);
            }

            @Override
            public Void visitCompoundAssignment(CompoundAssignmentTree tree, Void unused) {
                writeReceiver(tree.getVariable());
                return super.visitCompoundAssignment(tree, unused);
            }

            @Override
            public Void visitUnary(UnaryTree tree, Void unused) {
                switch (tree.getKind()) {
                    case PREFIX_INCREMENT, PREFIX_DECREMENT, POSTFIX_INCREMENT, POSTFIX_DECREMENT ->
                        writeReceiver(tree.getExpression());
                    default -> {
                    }
                }
                return super.visitUnary(tree, unused);
            }

            void writeReceiver(ExpressionTree variable) {
                TreePath path = at(variable);
                Element field = trees.getElement(path);
                if (field == null || field.getKind() != ElementKind.FIELD) {
                    return;
                }
                Element receiver = variable instanceof MemberSelectTree select
                        ? typeOf(at(select.getExpression()))
                        : trees.getScope(path).getEnclosingClass();
                add(path, variable, "write-receiver", written(variable), receiver);
            }

            @Override
            public Void visitCatch(CatchTree tree, Void unused) {
                Tree type = tree.getParameter().getType();
                List<? extends Tree> alternatives = type instanceof UnionTypeTree union
                        ? union.getTypeAlternatives() : List.of(type);
                for (Tree alternative : alternatives) {
                    add(at(alternative), alternative, "handler-type", written(alternative),
                            typeOf(at(alternative)));
                }
                return super.visitCatch(tree, unused);
            }
        }.scan(cu, null);
    }

    static String written(Tree tree) {
        return switch (tree) {
            case IdentifierTree identifier -> identifier.getName().toString();
            case MemberSelectTree select -> written(select.getExpression()) + "." + select.getIdentifier();
            case MemberReferenceTree reference ->
                written(reference.getQualifierExpression()) + "::" + reference.getName();
            case MethodInvocationTree call -> written(call.getMethodSelect()) + "()";
            case NewClassTree creation -> written(creation.getIdentifier()) + "()";
            case ParameterizedTypeTree parameterized -> written(parameterized.getType());
            default -> "";
        };
    }

    TypeElement topLevel(Element element) {
        TypeElement top = null;
        for (Element e = element; e != null && !(e instanceof PackageElement); e = e.getEnclosingElement()) {
            if (e instanceof TypeElement type) {
                top = type;
            }
        }
        return top;
    }

    boolean inRepository(Element element) {
        TypeElement top = topLevel(element);
        return top != null && trees.getPath(top) != null;
    }

    ExecutableElement outsideOverridden(ExecutableElement method) {
        var owner = (TypeElement) method.getEnclosingElement();
        var queue = new ArrayDeque<TypeMirror>(types.directSupertypes(owner.asType()));
        Set<Element> seen = new HashSet<>();
        while (!queue.isEmpty()) {
            Element type = types.asElement(queue.poll());
            if (!(type instanceof TypeElement supertype) || !seen.add(supertype)) {
                continue;
            }
            for (Element member : supertype.getEnclosedElements()) {
                if (member instanceof ExecutableElement candidate
                        && candidate.getSimpleName().equals(method.getSimpleName())
                        && elements.overrides(method, candidate, owner) && !inRepository(candidate)) {
                    return candidate;
                }
            }
            queue.addAll(types.directSupertypes(supertype.asType()));
        }
        return null;
    }

    ExecutableElement functionalMethod(Element type) {
        if (!(type instanceof TypeElement functional)) {
            return null;
        }
        Set<String> objectMethods = Set.of("equals", "hashCode", "toString");
        return elements.getAllMembers(functional).stream()
                .filter(member -> member instanceof ExecutableElement
                        && member.getModifiers().contains(Modifier.ABSTRACT)
                        && !objectMethods.contains(member.getSimpleName().toString()))
                .map(ExecutableElement.class::cast).findFirst().orElse(null);
    }

    String outcome(Element target) {
        if (target == null || target.asType().getKind() == TypeKind.ERROR) {
            return "{\"unresolved\":{\"reason\":\"not_found\"}}";
        }
        if (target instanceof PackageElement) {
            throw new IllegalStateException("an on-demand import, which the Java site rules do not cover yet");
        }
        TypeElement top = topLevel(target);
        String module = elements.getPackageOf(top).getQualifiedName().toString();
        String name = qualified(target).substring(module.isEmpty() ? 0 : module.length() + 1);
        String names = "\"module\":" + json(module) + (name.isEmpty() ? "" : ",\"name\":" + json(name));
        if (inRepository(top)) {
            return "{\"resolved\":{" + names + "}}";
        }
        String pin = pinOf(top);
        if (pin != null) {
            return "{\"external\":{\"package\":{\"pin\":" + json(pin) + "}," + names + "}}";
        }
        var jdkModule = elements.getModuleOf(top);
        if (jdkModule != null && !jdkModule.isUnnamed()) {
            return "{\"external\":{\"package\":\"stdlib\"," + names + "}}";
        }
        return "{\"unresolved\":{\"reason\":\"not_found\"}}";
    }

    String qualified(Element element) {
        return switch (element) {
            case TypeElement type -> type.getQualifiedName().toString();
            case ExecutableElement executable when executable.getKind() == ElementKind.CONSTRUCTOR ->
                qualified(executable.getEnclosingElement());
            default -> qualified(element.getEnclosingElement()) + "." + element.getSimpleName();
        };
    }

    String pinOf(TypeElement top) {
        try {
            JavaFileObject file = files.getJavaFileForInput(StandardLocation.CLASS_PATH,
                    elements.getBinaryName(top).toString(), JavaFileObject.Kind.CLASS);
            if (file == null) {
                return null;
            }
            String uri = file.toUri().toString();
            String jar = uri.substring("jar:".length(), uri.indexOf("!/"));
            String found = coordinates.get(Path.of(URI.create(jar)).toRealPath());
            return found == null ? null : unit + "/pin:" + slug(found);
        } catch (IOException e) {
            throw new IllegalStateException("the class file of " + top + " reads", e);
        }
    }

    static String slug(String name) {
        return name.toLowerCase().replaceAll("[^a-z0-9]+", "-").replaceAll("^-|-$", "");
    }

    void write(Path out) throws IOException {
        Map<String, List<Site>> groups = sites.stream()
                .sorted(Comparator.comparing(Site::path).thenComparingLong(Site::start))
                .collect(Collectors.groupingBy(site -> site.base().replaceFirst("(-[0-9]+)+$", ""),
                        TreeMap::new, Collectors.toList()));
        String checker = "{\"name\":\"javac\",\"version\":\"" + Runtime.version().feature() + "\"}";
        var records = new TreeMap<String, String>();
        var anchors = new TreeMap<String, String>();
        for (List<Site> group : groups.values()) {
            for (int i = 0; i < group.size(); i++) {
                Site site = group.get(i);
                String id = unit + "/site:" + site.base() + (group.size() > 1 ? "-" + i : "");
                if (records.put(id, "{\"id\":" + json(id) + ",\"kind\":" + json(site.kind()) + ",\"outcome\":"
                        + site.outcome() + ",\"checker\":" + checker + ",\"source\":\"extractor\"}") != null) {
                    throw new IllegalStateException("two sites have the ID " + id);
                }
                String quote = String.join("\n", sourceLines.get(site.path()).subList(site.first() - 1, site.last()));
                anchors.put(id, "{\"id\":" + json(id) + ",\"path\":" + json(site.path()) + ",\"lines\":["
                        + site.first() + "," + site.last() + "],\"quote\":" + json(quote) + "}");
            }
        }
        Files.createDirectories(out);
        Files.writeString(out.resolve("site.jsonl"), lines(records));
        Files.writeString(out.resolve("anchors.jsonl"), lines(anchors));
    }

    static String lines(TreeMap<String, String> records) {
        return records.values().stream().map(line -> line + "\n").collect(Collectors.joining());
    }

    static String json(String text) {
        var out = new StringBuilder("\"");
        for (char c : text.toCharArray()) {
            switch (c) {
                case '"' -> out.append("\\\"");
                case '\\' -> out.append("\\\\");
                case '\n' -> out.append("\\n");
                case '\r' -> out.append("\\r");
                case '\t' -> out.append("\\t");
                case '\b' -> out.append("\\b");
                case '\f' -> out.append("\\f");
                default -> out.append(c < 0x20 ? String.format("\\u%04x", (int) c) : String.valueOf(c));
            }
        }
        return out.append('"').toString();
    }
}
