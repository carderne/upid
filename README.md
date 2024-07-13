# UPID

pronounced YOO-pid

**aka Universally Unique Prefixed Lexicographically Sortable Identifier**

This is the spec and Python implementation for UPID.

UPID is based on [ULID](https://github.com/ulid/spec) but with some modifications, inspired by [this article](https://brandur.org/nanoglyphs/026-ids) and [Stripe IDs](https://dev.to/stripe/designing-apis-for-humans-object-ids-3o5a).

The core idea is that a **meaningful prefix** is specified that is stored in a 128-bit UUID-shaped slot.
Thus a UPID is **human-readable** (like a Stripe ID), but still efficient to store, sort and index.

UPID allows a prefix of up to **4 characters** (will be right-padded if shorter than 4), includes a non-wrapping timestamp with about 250 millisecond precision, and 64 bits of entropy.

This is a UPID in Python:
```python
upid("user")            # user_2accvpp5guht4dts56je5a
```

And in Rust:
```rust
UPID::new("user")      // user_2accvpp5guht4dts56je5a
```

And in Postgres too:
```sql
CREATE TABLE users (id upid NOT NULL DEFAULT gen_upid('user') PRIMARY KEY);
INSERT INTO users DEFAULT VALUES;
SELECT id FROM users;  -- user_2accvpp5guht4dts56je5a

-- this also works
SELECT id FROM users WHERE id = 'user_2accvpp5guht4dts56je5a';
```

Plays nice with your server code, no extra work needed:
```python
with psycopg.connect("postgresql://...") as conn:
    res = conn.execute("SELECT id FROM users").fetchone()
    print(res)          # user_2accvpp5guht4dts56je5a
```

## Specification
Key changes relative to ULID:
1. Uses a modified form of [Crockford's base32](https://www.crockford.com/base32.html) that uses lower-case and includes the full alphabet (for prefix flexibility).
2. Does not permit upper-case/lower-case to be decoded interchangeably.
3. The text encoding is still 5 bits per base32 character.
4. 20 bits assigned to the prefix
5. 40 bits (down from 48) assigned to the timestamp, placed first in binary for sorting
6. 64 bits (down from 80) for randomness
7. 4 bits as a version specifier

```elm
    user       2accvpp5      guht4dts56je5       a
   |----|     |--------|    |-------------|   |-----|
   prefix       time            random        version     total
   4 chars      8 chars         13 chars      1 char      26 chars
       \________/________________|___________    |
               /                 |           \   |
              /                  |            \  |
           40 bits            64 bits         24 bits    128 bits
           5 bytes            8 bytes         3 bytes     16 bytes
           time               random      prefix+version
```

### Binary layout
```
 0                   1                   2                   3
 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                            time_high                          |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|    time_low   |                     random                    |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|                             random                            |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
|     random    |                  prefix_and_version           |
+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
```

### Collision
Relative to ULID, the time precision is reduced from 48 to 40 bits (keeping the most significant bits, so overflow still won't occur until 10889 AD), and the randomness reduced from 80 to 64 bits.

The timestamp precision at 40 bits is around 250 milliseconds. In order to have a 50% probability of collision with 64 bits of randomness, you would need to generate around **4 billion items per 250 millisecond window**.

## Python implementation
This aims to be maximally simple to convey the core working of the spec.
The current Python implementation is entirely based on [mdomke/python-ulid](https://github.com/mdomke/python-ulid).

#### Installation
```bash
pip install upid
```

#### Usage
Run from the CLI:
```bash
python -m upid user
```

Use in a program:
```python
from upid import upid
upid("user")
```

Or more explicitly:
```python
from upid import UPID
UPID.from_prefix("user")
```

Or specifying your own timestamp or datetime
```python
import time, datetime
UPID.from_prefix_and_milliseconds("user", milliseconds)
UPID.from_prefix_and_datetime("user", datetime.datetime.now())
```

From and to a string:
```python
u = UPID.from_str("user_2accvpp5guht4dts56je5a")
u.to_str()        # user_2a...
```

Get stuff out:
```python
u.prefix     # user
u.datetime   # 2024-07-07 ...
```

Convert to other formats:
```python
int(u)       # 2079795568564925668398930358940603766
u.hex        # 01908dd6a3669b912738191ea3d61576
u.to_uuid()  # UUID('01908dd6-a366-9b91-2738-191ea3d61576')
```

#### Development
Code and tests are in the [py/](./py/) directory.
Using [Rye](https://rye.astral.sh/) for development (installation instructions at the link).

```bash
# can be run from the repo root
rye sync
rye run all  # or fmt/lint/check/test
```

If you just want to have a look around, pip should also work:
```bash
pip install -e .
```

Please open a PR if you spot a bug or improvement!

## Rust implementation
The current Rust implementation is based on [dylanhart/ulid-rs](https://github.com/dylanhart/ulid-rs), but using the same lookup base32 lookup method as the Python implementation.

#### Installation
```bash
cargo add upid
```

#### Usage
```rust
use upid::Upid;
Upid::new("user");
```

Or specifying your own timestamp or datetime:
```rust
use std::time::SystemTime;
Upid::from_prefix_and_milliseconds("user", 1720366572288);
Upid::from_prefix_and_datetime("user", SystemTime::now());
```

From and to a string:
```rust
let u = Upid::from_string("user_2accvpp5guht4dts56je5a");
u.to_string();
```

Get stuff out:
```rust
u.prefix();       // user
u.datetime();     // 2024-07-07 ...
u.milliseconds(); // 17203...
```

Convert to other formats:
```rust
u.to_bytes();
```

#### Development
Code and tests are in the [upid_rs/](./upid_rs/) directory.

```bash
cd upid_rs
cargo check  # or fmt/clippy/build/test/run
```

Please open a PR if you spot a bug or improvement!

## Postgres extension
There is also a Postgres extension built on the Rust implementation, using [pgrx](https://github.com/pgcentralfoundation/pgrx) and based on the very similar extension [pksunkara/pgx_ulid](https://github.com/pksunkara/pgx_ulid).

#### Installation
The easiest would be to try out the Docker image [carderne/postgres-upid:16](https://hub.docker.com/r/carderne/postgres-upid), currently built for arm64 and amd64 but only for Postgres 16:
```bash
docker run -e POSTGRES_HOST_AUTH_METHOD=trust -p 5432:5432 carderne/postgres-upid:16
```

You can also grab a Linux `.deb` from the [Releases](https://github.com/carderne/upid/releases) page. This is built for Postgres 16 and amd64 only.

More architectures and versions will follow once it is out of alpha.

#### Usage
```sql
CREATE EXTENSION upid_pg;

CREATE TABLE users (
    id   upid NOT NULL DEFAULT gen_upid('user') PRIMARY KEY,
    name text NOT NULL
);

INSERT INTO users (name) VALUES('Bob');

SELECT * FROM users;
--              id              | name
-- -----------------------------+------
--  user_2accvpp5guht4dts56je5a | Bob
```

You can get the raw `bytea` data, or the prefix or timestamp:
```sql
SELECT upid_to_bytea(id) FROM users;
-- \x019...

SELECT upid_to_prefix(id) FROM users;
-- 'user'

SELECT upid_to_timestamp(id) FROM users;
-- 2024-07-07 ...
```

You can convert a `UPID` to a regular Postgres `UUID`:
```sql
SELECT upid_to_uuid(gen_upid('user'));
```

Or the reverse (although the prefix and timestamp will no longer make sense):
```sql
select upid_from_uuid(gen_random_uuid());
```

#### Development
If you want to install it into another Postgres, you'll install pgrx and follow its [installation instructions](https://github.com/pgcentralfoundation/pgrx/blob/develop/cargo-pgrx/README.md).
Something like this:
```bash
cd upid_pg
cargo install --locked cargo-pgrx
cargo pgrx init
cargo pgrx install
```

Some `cargo` commands work as normal:
```bash
cargo check  # or fmt/clippy
```

But building, testing and running must be done via pgrx.
This will compile it into a Postgres installation, and allow an interactive session and tests there.

```bash
cargo pgrx test pg16
# or       run
# or       install
```
