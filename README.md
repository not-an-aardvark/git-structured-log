# git-structured-log

A tool for extracting git data in bulk

## Motivation

When using git, it's difficult to extract data from a large number of commits. For example, when autogenerating a changelog, one might want to obtain the commit message, commit hash, and commit author of each of the last 100 commits, and run a script that analyzes this data. The most commonly-used solution would be to use something like `git log --pretty=format` and then parse the output:

```bash
$ git log --pretty=format:"(%h) (%s) (%an)" aaaaaaa..bbbbbbb
(bbbbbbb) (Foo Bar) (John Smith)
(1234567) (Baz Qux) (Jane Doe)
...
```

However, this solution has some problems: if the user puts special characters such as parentheses in their name or the commit message, then it won't be possible to parse the output correctly. Format strings appear to be git's only mechanism for bulk commit data extraction, but it's not possible to make format strings reliable.

This tool aims to provide an alternative by allowing bulk git data to be exported in a well-structured format.

## Installation

```bash
cargo install git_structured_log
git_structured_log <exclusive start of range>..<inclusive end of range> <comma-separated list of format flags>
```

The tool will output a one-line JSON object for each commit in the given range. The keys of the object will be the same as the provided format flags.

Most of the format flags from [git pretty-formats](https://git-scm.com/docs/pretty-formats) are supported (see the section with the `%` placeholders). For example, in order to output the commit message, commit hash, and commit author name of the last 5 commits, one might use:

```bash
$ git_structured_log HEAD~5..HEAD s,h,an
{"an":"Deepti Gandluri","h":"ad3d0ba","s":"[wasm] Add I64 Atomic binary operations for x64"}
{"an":"Sathya Gunasekaran","h":"658af9d","s":"[test262] Roll test262"}
{"an":"Jakob Kummerow","h":"7c79a9f","s":"[bigint] Stage BigInts"}
{"an":"Deepti Gandluri","h":"782f640","s":"Revert \"[parser] Implements proposal-numeric-separator.\""}
{"an":"Taketoshi Aono","h":"517df52","s":"[parser] Implements proposal-numeric-separator."}
```

## Goals

- Allow data to be safely extracted from repositories where commits come from untrusted users.
- Be comparable in performance to git itself.
- Avoid producing any ambiguous output.
- Prioritize usage by integrations (e.g. avoid adding multiple formatting modes for the same data if an integration would easily be able to format the data on its own).

## License

MIT License (see the `LICENSE.md` file for more details)
