def commits(s):
    s.commit()  # $ callee=collisions-counters-commits-callee-s-commit-0
    s.commit_0()  # $ callee=collisions-counters-commits-callee-s-commit-0-1
    s.commit()  # $ callee=collisions-counters-commits-callee-s-commit-2


def three(s):
    s.save()  # $ callee=collisions-counters-three-callee-s-save-0
    s.save()  # $ callee=collisions-counters-three-callee-s-save-1
    s.save()  # $ callee=collisions-counters-three-callee-s-save-2


def alone(s):
    s.commit_0()  # $ callee=collisions-counters-alone-callee-s-commit-0
