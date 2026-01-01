# CLI Usage

This CLI has two modes:

1) Parse a recipe URL (existing behavior):

```bash
cli <url>
```

2) Bootstrap the admin user (single-user auth):

```bash
cargo run -p cli -- admin-bootstrap [--db <path-or-url>] [--username <u>] [--password <p>]
```

Notes:
- `--db` defaults to `./dev.db`.
- You can pass a full sqlite URL (`sqlite:./dev.db`) or a path (`./dev.db`).
- If no admin user exists, omitting `--username` and `--password` generates random
  credentials and sets `must_change_password = true`. The CLI prints them once.
- If an admin user already exists, this command will *not* reset the password.
  It will only print the existing username and `must_change_password` state.
  To reset, delete the database (e.g. `./dev.db`) and run the command again.

Example output when generated:

```
admin user: admin_ab12cd34
generated username: admin_ab12cd34
generated password: <printed once>
must_change_password: true
```
