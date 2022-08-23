# `sqlx prepare` Bug Example

Attempt to reproduce a bug where `sqlx prepare` pulls in queries from multiple crates in the same workspace.

See https://github.com/launchbadge/sqlx/issues/2049

## Setup

Start a postgres instance:

```bash
docker run -it -p 127.0.0.1:5432:5432 --rm -e POSTGRES_PASSWORD=password postgres
```

Set the `DATABASE_URL` environment variable in your shell accordingly:

```bash
export DATABASE_URL=postgres://postgres:password@localhost/mre
```

or, for PowerShell:

```psh
$env:DATABASE_URL="postgres://postgres:password@localhost/mre"
```

Install `sqlx-cli`:

```bash
cargo install -f sqlx-cli
```

Create the database and run migrations:

```bash
sqlx database create
sqlx migrate run
```

## Reproduction

Attempt to reproduce having `cargo sqlx prepare` seeing queries from other crates in the same workspace by building them at the same time, which will generate query files for them under `target/sqlx/query*.json`. 

```bash
cd crate-b && cargo check &
cd cratea && cargo sqlx prepare & 
cd crate_c && cargo check &
```

I think the bug is due to a possible race condition between `cargo sqlx prepare` deleting the `target/sqlx` directory and reading from it where other query files can be generated.

Note that file locks will sometimes prevent the bug from occurring:

```text
Blocking waiting for file lock on package cache
Blocking waiting for file lock on build directory
```

### Expected

The `sqlx-data.json` file should contain a single table definition and `query`.

E.g.
```json
{
  "db": "PostgreSQL",
  "11341dbfb48a0d625d76ff27c1f9da48f2046904ea8702f0f406f8f123afa7ac": {
    "describe": {
      "columns": [
        {
          "name": "id_a",
          "ordinal": 0,
          "type_info": "Int8"
        }
      ],
      "nullable": [
        false
      ],
      "parameters": {
        "Left": [
          "Text"
        ]
      }
    },
    "query": "\nINSERT INTO table_a ( name_a )\nVALUES ( $1 )\nRETURNING id_a\n        "
  }
}
```

### Actual

The contents of `sqlx-data.json` may sometimes be missing, e.g.

```json
{
  "db": "PostgreSQL"
}
```

TODO: reproduce multiple queries in the same file.
