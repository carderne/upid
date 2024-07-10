# UPID

pronounced YOO-pid

**aka Universally Unique Prefixed Lexicographically Sortable Identifier**

This is the spec and Python implementation for UPID.

UPID is based on [ULID](https://github.com/ulid/spec) but with some modifications, inspired by [this article](https://brandur.org/nanoglyphs/026-ids) and [Stripe IDs](https://dev.to/stripe/designing-apis-for-humans-object-ids-3o5a).

The core idea is that a **meaningful prefix** is specified that is stored in a 128-bit UUID-shaped slot.
Thus a UPID is **human-readable** (like a Stripe ID), but still efficient to store, sort and index.

UPID allows a prefix of up to **4 characters** (will be right-padded if shorter than 4), includes a non-wrapping timestamp with about 300 millisecond precision, and 64 bits of entropy.

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

Plays nice with your server code too, no extra work needed:
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
Relative to ULID, the time precision is reduced from 48 to 40 bits (keeping the most significant bits, so oveflow still won't occur until 10889 AD), and the randomness reduced from 80 to 64 bits.

The timestamp precision at 40 bits is around 300 milliseconds. In order to have a 50% probability of collision with 64 bits of randomness, you would need to generate around **4 billion items per 100 millisecond window**.

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

#### Development
```bash
rye sync
rye run all  # or fmt/lint/check/test
```

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

#### Development
```bash
cargo check  # or fmt/clippy/test/run
```

## Postgres extension
There is also a Postgres extension built on the Rust implementation, using [pgrx](https://github.com/pgcentralfoundation/pgrx) and based on the very similar extension [pksunkara/pgx_ulid](https://github.com/pksunkara/pgx_ulid).

#### Installation
You will need to install pgrx and follow its installation instructions.
Something like this:
```bash
cargo install --locked cargo-pgrx
pgrx init
cd upid_pg
pgrx install
pgrx run
```

Alternatively, you can try out the Docker image `[carderne/postgres-upid:16](https://hub.docker.com/r/carderne/postgres-upid):
```bash
docker run -e POSTGRES_HOST_AUTH_METHOD=trust -p 5432:5432 carderne/postgres-upid:16
```

#### Usage
```sql
CREATE EXTENSION ulid;


CREATE TABLE users (
    id   upid NOT NULL DEFAULT gen_upid('user') PRIMARY KEY,
    name text NOT NULL
);
INSERT INTO users (name) VALUES('Bob');
SELECT * FROM users;
```

#### Development
```bash
cargo pgrx test
```
