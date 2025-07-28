# vikunja-tui

> [!NOTE]
> While you are welcome to use this tool, I would also recommend checking out [cria](https://github.com/frigidplatypus/cria) which has an even better feature set and more active development.

[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-%23FE5196?logo=conventionalcommits&logoColor=white)](https://conventionalcommits.org)

This is a simple terminal user interface for [vikunja](https://vikunja.io). The purpose is to allow users to manage tasks from the terminal, using their own API key. This project is not managed or affiliated with the Vikunja team.

## Installation

```
cargo install vikunja-tui
```

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
- Toggle between showing complete tasks and incomplete tasks (shows incomplete by default)
- Pagination of tasks.
- Add tasks
	- Title
	- Priority, via ![1-5] (vikunja quick add magic syntax)
	- Due date,  via due:YYYY-MM-DD. No time is needed/supported at present, it will assume UTC 23:59:59.
	- Description, via a seperate input box during task creation

## Roadmap

- [ ] Parse labels via `*label`
- [ ] Edit existing tasks

