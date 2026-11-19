import abc  # $ import=classes-bases-import-abc
from typing import Generic, TypeVar  # $ import=classes-bases-import-typing-generic import=classes-bases-import-typing-typevar

T = TypeVar("T")  # $ callee=classes-bases-callee-typevar


class Plain:
    pass


class One(Plain):  # $ base-class=classes-bases-one-base-class-plain
    pass


class Many(abc.ABC, Generic[T], metaclass=abc.ABCMeta):  # $ base-class=classes-bases-many-base-class-abc-abc base-class=classes-bases-many-base-class-generic
    pass


class Made(make_base()):  # $ base-class=classes-bases-made-base-class-make-base callee=classes-bases-made-callee-make-base
    pass
