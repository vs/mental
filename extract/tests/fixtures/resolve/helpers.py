def double(x):
    return x * 2


class Shape:
    sides = 0


class Runner:
    def __call__(self, *args):
        return args


run = Runner()  # $ callee=resolve-helpers-callee-runner


def decorator_factory():
    return run
