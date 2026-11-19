def outer():
    def inner():
        work()  # $ callee=scopes-nested-outer-inner-callee-work

    class Local:
        def method(self):
            self.save()  # $ callee=scopes-nested-outer-local-method-callee-self-save

    return inner


class Service:
    limit = compute()  # $ callee=scopes-nested-service-callee-compute

    def run(self, size=default_size(), *, key: Annotated[str, depends()] = None) -> result():  # $ callee=scopes-nested-service-run-callee-default-size callee=scopes-nested-service-run-callee-depends callee=scopes-nested-service-run-callee-result
        return [self.fetch(item) for item in self.items()]  # $ callee=scopes-nested-service-run-callee-self-fetch callee=scopes-nested-service-run-callee-self-items

    async def later(self):
        await self.fetch(1)  # $ callee=scopes-nested-service-later-callee-self-fetch
