# Security policy

## Supported versions

Only the latest release gets security fixes. Knoa updates itself, so most people are on
it already.

## Reporting a vulnerability

Report it privately through
[GitHub's private vulnerability reporting](https://github.com/50BytesOfJohn/knoa/security/advisories/new).
Don't open a public issue for it.

Say what the problem is, how to reproduce it, and which version you saw it in. You'll get
a reply within a week. Once a fix ships, the advisory is published and you're credited,
unless you'd rather not be.

Things worth reporting include: a way for a fetched web page or an LLM response to run
code or read files outside Knoa's data folder, leaks of your API key, and anything that
gets around the update signature check.
