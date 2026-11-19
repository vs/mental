config.debug = True  # $ write-receiver=writes-forms-write-receiver-config-debug


def writes(report, rows, items):
    report.status = "done"  # $ write-receiver=writes-forms-writes-write-receiver-report-status
    report.count += 1  # $ write-receiver=writes-forms-writes-write-receiver-report-count
    report.note: str = "n"  # $ write-receiver=writes-forms-writes-write-receiver-report-note
    report.a, (report.b, *report.rest) = rows  # $ write-receiver=writes-forms-writes-write-receiver-report-a write-receiver=writes-forms-writes-write-receiver-report-b write-receiver=writes-forms-writes-write-receiver-report-rest
    for report.row in rows:  # $ write-receiver=writes-forms-writes-write-receiver-report-row
        pass
    with open_file() as report.handle:  # $ callee=writes-forms-writes-callee-open-file write-receiver=writes-forms-writes-write-receiver-report-handle
        pass
    report.parent.child = 1  # $ write-receiver=writes-forms-writes-write-receiver-report-parent-child
    items[0].name = "x"  # $ write-receiver=writes-forms-writes-write-receiver-items-name
    items["key"] = 1
    del report.status
    total = report.total
