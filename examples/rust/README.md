# example-python

Clone this repo:
```bash
git clone git@github.com:carderne/upid.git
cd upid/examples/rust
```

Get a Docker image running:
```bash
docker run --rm -e POSTGRES_PASSWORD=mypassword \
  -p 5432:5432 --detach carderne/postgres-upid:16
```

Run the script:
```bash
cargo run

# Extension ready
# Table created
# Inserted:
# id_upid=user_2acqyewitnjij6oflrqzda
# id_uuid=01916f2b-8ecc-dee7-928b-8dedf9d61576
# id_text=user_2acqyewitnjij6oflrqzda
```
