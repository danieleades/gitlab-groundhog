# GroundHog

[![codecov](https://codecov.io/gh/danieleades/gitlab-groundhog/graph/badge.svg?token=nkwdf2qdhs)](https://codecov.io/gh/danieleades/gitlab-groundhog)

A small utility for creating recurring tasks in Gitlab.

## Installation

Easiest way to install is using [Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html)

        cargo install --git https://github.com/danieleades/gitlab-groundhog

Check the docs:

        groundhog --help

## Getting Started

`GroundHog` expects to be run in a directory containing a few required files

- `issues.yml` defines the recurring issues
- `ledger.json` provides a record of issues that have already been created. *This should not be manually edited*.
- `templates/` is a directory which contains [Tera templates](https://keats.github.io/tera/docs/#introduction) used to generate issue bodies

You can initialise a new directory containing all the required files using the `groundhog init "path/to/directory"` command.

Once the directory is set up, you can generate new gitlab issues using `groundhog run`. GroundHog will use the current date and the 'ledger' to determine when new issues need to be created.

Use a cronjob or similar to run `groundhog` at regular intervals.
