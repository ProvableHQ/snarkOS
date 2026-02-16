# CI overview

snarkOS makes use of [CircleCI](.circleci) and [Github Actions](.github/workflows).

When a PR is opened from a feature branch, several CircleCI workflows are
triggered. Tests are spread across workflows for readability in Github's UI, and
are targeting a 15 minute max runtime.

When a PR is merged, `chaotic-devnet-workflow`, `upgrade-workflow` and
`windows-workflow` run additional expensive or slow tests. Moreover, benchmarks
are run from github actions.

Many tests make use of scripts defined in the `.ci` folder. To run them on
macOS you may want to `brew install bash coreutils && echo 'export
PATH="/opt/homebrew/bin:$PATH"' >> ~/.bash_profile`
