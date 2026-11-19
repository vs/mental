def callee_y():
    import z  # $ import=collisions-joined-callee-y-import-z-0


y.import_z()  # $ callee=collisions-joined-callee-y-import-z-1
