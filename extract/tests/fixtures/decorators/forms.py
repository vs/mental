import functools  # $ import=decorators-forms-import-functools

registry = make_registry()  # $ callee=decorators-forms-callee-make-registry


@functools.cache  # $ decorator=decorators-forms-plain-decorator-functools-cache
def plain():
    pass


@registry.route("/a", methods=["GET"])  # $ decorator=decorators-forms-called-decorator-registry-route
def called():
    pass


@registry.on(signal())  # $ decorator=decorators-forms-with-call-in-arguments-decorator-registry-on callee=decorators-forms-with-call-in-arguments-callee-signal
def with_call_in_arguments():
    pass


@registry.group().command()  # $ decorator=decorators-forms-chained-decorator-registry-group-command callee=decorators-forms-chained-callee-registry-group
def chained():
    pass


@(lambda function: function)  # $ decorator=decorators-forms-anything-decorator
def anything():
    pass


@functools.total_ordering  # $ decorator=decorators-forms-decorated-decorator-functools-total-ordering
class Decorated:
    pass
