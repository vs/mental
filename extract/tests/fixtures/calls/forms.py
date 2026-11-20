print("module")  # $ callee=calls-forms-callee-print


def forms(s, rows, handlers):
    s.query(rows).filter(rows).all()  # $ callee=calls-forms-forms-callee-s-query callee=calls-forms-forms-callee-s-query-filter callee=calls-forms-forms-callee-s-query-filter-all
    handlers[0](rows)  # $ callee=calls-forms-forms-callee-handlers
    make()()  # $ callee=calls-forms-forms-callee-make-0 callee=calls-forms-forms-callee-make-1
    ", ".join(rows)  # $ callee=calls-forms-forms-callee-join
    (lambda: rows)()  # $ callee=calls-forms-forms-callee
    print(f"{len(rows)}")  # $ callee=calls-forms-forms-callee-print callee=calls-forms-forms-callee-len
    sorted(rows, key=lambda row: row.get("k"))  # $ callee=calls-forms-forms-callee-sorted callee=calls-forms-forms-callee-row-get
    return (
        s.flush()  # $ callee=calls-forms-forms-callee-s-flush
    )
