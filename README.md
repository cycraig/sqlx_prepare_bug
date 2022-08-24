# `sqlx prepare` Bug Example

Attempt to reproduce a bug where `sqlx prepare` pulls in queries from multiple crates in the same workspace.

See https://github.com/launchbadge/sqlx/issues/2049

## Setup

Start a local, ephemeral postgres instance:

```bash
docker run -it -p 127.0.0.1:5432:5432 --rm -e POSTGRES_PASSWORD=password postgres
```

In a separate terminal, set the `DATABASE_URL` environment variable in your shell accordingly:

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
cd ./crate-b && cargo check &
cd ./cratea && cargo sqlx prepare & 
cd ./crate_ç && cargo check &
```

I think the bug is due to a possible race condition between `cargo sqlx prepare` deleting the `target/sqlx` directory and reading from it where other query files can be generated.

Note that file locks will sometimes prevent the bug from occurring:

```text
Blocking waiting for file lock on package cache
Blocking waiting for file lock on build directory
```

### Expected

The `cratea/sqlx-data.json` file should contain a single `query`.

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

The contents of `cratea/sqlx-data.json` may contain queries from other crates in the workspace intermittently, e.g.

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
  },
  "d4b7dc3d15f66233aaa184ec5f47327cca9dbfd9010fe8d084031daf3b81dbb9": {
    "describe": {
      "columns": [
        {
          "name": "id_b",
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
    "query": "\nINSERT INTO table_b ( name_b )\nVALUES ( $1 )\nRETURNING id_b\n        "
  }
}
```

Multiple runs may be required to reproduce this behaviour. 

## Evaluate Fix

First, change the `sqlx` dependency in the Cargo.toml files of `cratea`, `crate-b` and `crate_ç` to use the fix from the forked branch [fix/prepare-race-condition](https://github.com/cycraig/sqlx/tree/fix/prepare-race-condition):

```toml
sqlx = { git = "https://github.com/cycraig/sqlx", branch = "fix/prepare-race-condition", features = ["offline", "postgres", "runtime-tokio-rustls"] }
```

It is also necessary to install the forked `sqlx-cli`. 
Either:

- Install directly from git:

```bash
cargo install -f --git https://github.com/cycraig/sqlx --branch fix/prepare-race-condition sqlx-cli 
```

- OR clone the branch then install it:
```bash
git clone --single-branch --branch fix/prepare-race-condition https://github.com/cycraig/sqlx.git
cargo install -f --path ./sqlx/sqlx-cli 
```

Finally, run the following again:
```bash
cd ./crate-b && cargo check &
cd ./cratea && cargo sqlx prepare & 
cd ./crate_ç && cargo check &
```

It should correctly generate `cratea/sqlx-data.json` with only the queries for `cratea`, no matter the order of execution or timing.
