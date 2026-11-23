try:
    import fast_json  # $ import=resolve-optional-import-fast-json@unresolved:missing_module
    try:
        import inner_absent  # $ import=resolve-optional-import-inner-absent@unresolved:missing_module
    finally:
        pass

    class Holder:
        import class_absent  # $ import=resolve-optional-holder-import-class-absent@unresolved:missing_module

    def later():
        import call_absent  # $ import=resolve-optional-later-import-call-absent@unresolved:missing_module
except ImportError:  # $ handler-type=resolve-optional-handler-type-importerror@external:stdlib:builtins:ImportError
    import handler_absent  # $ import=resolve-optional-import-handler-absent@unresolved:missing_module
else:
    import else_absent  # $ import=resolve-optional-import-else-absent@unresolved:missing_module
finally:
    import finally_absent  # $ import=resolve-optional-import-finally-absent@unresolved:missing_module

try:
    from tuple_absent import thing  # $ import=resolve-optional-import-tuple-absent-thing@unresolved:missing_module
except (ValueError, ModuleNotFoundError):  # $ handler-type=resolve-optional-handler-type-valueerror@external:stdlib:builtins:ValueError handler-type=resolve-optional-handler-type-modulenotfounderror@external:stdlib:builtins:ModuleNotFoundError
    thing = None

try:
    import broad_absent  # $ import=resolve-optional-import-broad-absent@unresolved:missing_module
except Exception:  # $ handler-type=resolve-optional-handler-type-exception@external:stdlib:builtins:Exception
    pass

try:
    import bare_absent  # $ import=resolve-optional-import-bare-absent@unresolved:missing_module
except:  # noqa: E722
    pass
