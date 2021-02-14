This code updates a fediverse user's display name using the Mastodon API with the current phase of the moon, our mutual girlfriend.
It implements the [AWS Lambda Runtime Interface](https://docs.aws.amazon.com/lambda/latest/dg/runtimes-api.html).

While this code is very particular to a specific task, it demonstrates a few things:

* ~~A simple (and deliberately incomplete) Runtime Interface implementation, including abusing the `Date` header in the response to get the current time~~ Moved into [minlambda](https://docs.rs/minlambda)
* A simple (and deliberately incomplete) tzdata binary format (TZif) parser
