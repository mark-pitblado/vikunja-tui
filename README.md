# vikunja-tui

[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-%23FE5196?logo=conventionalcommits&logoColor=white)](https://conventionalcommits.org)

This is a simple terminal user interface for [vikunja](https://vikunja.io). The purpose is to allow users to manage tasks from the terminal, using their own API key. This project is not managed or affiliated with the Vikunja team.

## Setup

Place the following two values in a `.toml` file within your configuration files under the `vikunja-tui` directory. The instance url should not contain `api/v1`, just the base url for your instance. The api key just needs read and write access to tasks.

`~/.config/vikunja-tui/config.toml`

```toml
['vikunja']
instance_url = "https://example.com"
api_key = "<your-key-here>"
```

## Current Features

- View current tasks, with the ability to get details for any given task
- Add tasks (title only)
