import errors  # $ import=exceptions-handlers-import-errors


def handlers(work):
    try:
        work()  # $ callee=exceptions-handlers-handlers-callee-work-0
    except ValueError:  # $ handler-type=exceptions-handlers-handlers-handler-type-valueerror
        pass
    except (KeyError, errors.Timeout) as error:  # $ handler-type=exceptions-handlers-handlers-handler-type-keyerror handler-type=exceptions-handlers-handlers-handler-type-errors-timeout
        pass
    except errors.select()[0]:  # $ handler-type=exceptions-handlers-handlers-handler-type-errors-select callee=exceptions-handlers-handlers-callee-errors-select
        pass
    except:
        pass
    try:
        work()  # $ callee=exceptions-handlers-handlers-callee-work-1
    except* OSError:  # $ handler-type=exceptions-handlers-handlers-handler-type-oserror
        pass
