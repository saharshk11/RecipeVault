# CLI Usage

This CLI has two modes:

1) Parse a recipe URL (existing behavior):

```bash
cli <url>
```

2) Bootstrap or reset the admin user (single-user auth):

```bash
cli admin-bootstrap [--db <path-or-url>] [--username <u>] [--password <p>]
```

Notes:
- `--db` defaults to `./dev.db`.
- You can pass a full sqlite URL (`sqlite:./dev.db`) or a path (`./dev.db`).
- If `--username` and `--password` are not provided, the CLI generates random credentials
  and marks the user as `must_change_password = true`.
