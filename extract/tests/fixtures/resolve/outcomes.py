import os  # $ import=resolve-outcomes-import-os@external:stdlib:os:
import nowhere  # $ import=resolve-outcomes-import-nowhere@unresolved:missing_module
import stray  # $ import=resolve-outcomes-import-stray@unresolved:not_found
import local_pkg  # $ import=resolve-outcomes-import-local-pkg@unresolved:not_found
from greet import absent, hello  # $ import=resolve-outcomes-import-greet-absent@unresolved:not_found import=resolve-outcomes-import-greet-hello-0@external:greet:greet:hello
from os.path import *  # $ import=resolve-outcomes-import-os-path-0@unresolved:dynamic
from typing import Callable, Never  # $ import=resolve-outcomes-import-typing-callable@external:stdlib:typing:Callable import=resolve-outcomes-import-typing-never@external:stdlib:typing:Never
import os.path  # $ import=resolve-outcomes-import-os-path-1@external:stdlib:os.path:
from os.path import absent  # $ import=resolve-outcomes-import-os-path-absent@unresolved:not_found
from greet.missing import nothing  # $ import=resolve-outcomes-import-greet-missing-nothing@unresolved:missing_module

from resolve.helpers import Shape, decorator_factory, double, run  # $ import=resolve-outcomes-import-resolve-helpers-shape@resolved:resolve.helpers:Shape import=resolve-outcomes-import-resolve-helpers-decorator-factory@resolved:resolve.helpers:decorator_factory import=resolve-outcomes-import-resolve-helpers-double-0@resolved:resolve.helpers:double import=resolve-outcomes-import-resolve-helpers-run@resolved:resolve.helpers:run

try:
    from greet import hello as pick  # $ import=resolve-outcomes-import-greet-hello-1@external:greet:greet:hello
except ImportError:  # $ handler-type=resolve-outcomes-handler-type-importerror@external:stdlib:builtins:ImportError
    from resolve.helpers import double as pick  # $ import=resolve-outcomes-import-resolve-helpers-double-1@resolved:resolve.helpers:double


def calls(untyped, handlers: list, never: Never):
    print("x")  # $ callee=resolve-outcomes-calls-callee-print@external:stdlib:builtins:print
    double(1)  # $ callee=resolve-outcomes-calls-callee-double@resolved:resolve.helpers:double
    untyped.run()  # $ callee=resolve-outcomes-calls-callee-untyped-run@unresolved:no_type
    Shape().missing()  # $ callee=resolve-outcomes-calls-callee-shape-missing@unresolved:not_found callee=resolve-outcomes-calls-callee-shape@resolved:resolve.helpers:Shape
    handlers[0]()  # $ callee=resolve-outcomes-calls-callee-handlers@unresolved:dynamic
    undefined()  # $ callee=resolve-outcomes-calls-callee-undefined@unresolved:not_found
    never.run()  # $ callee=resolve-outcomes-calls-callee-never-run@unresolved:unreachable
    untyped()  # $ callee=resolve-outcomes-calls-callee-untyped@unresolved:no_type
    never()  # $ callee=resolve-outcomes-calls-callee-never@unresolved:unreachable
    pick()  # $ callee=resolve-outcomes-calls-callee-pick@unresolved:ambiguous


def writes(shape: Shape, untyped):
    shape.sides = 3  # $ write-receiver=resolve-outcomes-writes-write-receiver-shape-sides@resolved:resolve.helpers:Shape
    untyped.sides = 3  # $ write-receiver=resolve-outcomes-writes-write-receiver-untyped-sides@unresolved:no_type


def escapes(shape: Shape):
    register(double)  # $ callee=resolve-outcomes-escapes-callee-register@unresolved:not_found escape=resolve-outcomes-escapes-escape-double@resolved:resolve.helpers:double
    os.fspath(Shape)  # $ callee=resolve-outcomes-escapes-callee-os-fspath-0@external:stdlib:os:fspath escape=resolve-outcomes-escapes-escape-shape@resolved:resolve.helpers:Shape
    os.fspath(shape)  # $ callee=resolve-outcomes-escapes-callee-os-fspath-1@external:stdlib:os:fspath
    os.fspath(nothing_here)  # $ callee=resolve-outcomes-escapes-callee-os-fspath-2@external:stdlib:os:fspath escape=resolve-outcomes-escapes-escape-nothing-here@unresolved:not_found
    os.fspath(path=os.sep)  # $ callee=resolve-outcomes-escapes-callee-os-fspath-3@external:stdlib:os:fspath
    double(double)  # $ callee=resolve-outcomes-escapes-callee-double@resolved:resolve.helpers:double


@hello  # $ decorator=resolve-outcomes-decorated-decorator-hello@external:greet:greet:hello
def decorated():
    pass


class Square(Shape):  # $ base-class=resolve-outcomes-square-base-class-shape@resolved:resolve.helpers:Shape
    pass


def runs(kind: type[Shape]):
    run()  # $ callee=resolve-outcomes-runs-callee-run@resolved:resolve.helpers:Runner.__call__
    kind()  # $ callee=resolve-outcomes-runs-callee-kind@resolved:resolve.helpers:Shape


@decorator_factory()  # $ decorator=resolve-outcomes-produced-decorator-decorator-factory@resolved:resolve.helpers:decorator_factory
def produced():
    pass


def callbacks(callback: Callable[[], None]):
    callback()  # $ callee=resolve-outcomes-callbacks-callee-callback@resolved:resolve.outcomes:callbacks.callback


class Box:
    def __init__(self):
        self.size = 1  # $ write-receiver=resolve-outcomes-box-init-write-receiver-self-size@resolved:resolve.outcomes:Box

    @classmethod  # $ decorator=resolve-outcomes-box-make-decorator-classmethod@external:stdlib:builtins:classmethod
    def make(cls):
        return cls()  # $ callee=resolve-outcomes-box-make-callee-cls@resolved:resolve.outcomes:Box

    def copy(self):
        def grow():
            self.size = 2  # $ write-receiver=resolve-outcomes-box-copy-grow-write-receiver-self-size@resolved:resolve.outcomes:Box

        grow()  # $ callee=resolve-outcomes-box-copy-callee-grow@resolved:resolve.outcomes:Box.copy.grow
        return self.__class__()  # $ callee=resolve-outcomes-box-copy-callee-self-class@resolved:resolve.outcomes:Box


def made[T: Shape](kind: type[T]):
    kind()  # $ callee=resolve-outcomes-made-callee-kind@resolved:resolve.outcomes:made.T
