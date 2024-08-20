# example-python

Clone this repo:
```bash
git clone git@github.com:carderne/upid.git
cd upid/examples/python
```

Get a Docker image running:
```bash
docker run --rm -e POSTGRES_PASSWORD=mypassword \
  -p 5432:5432 --detach carderne/postgres-upid:16
```

Install requirements:
```bash
pip install -r requirements.txt
```


Run the script:
```bash
python example.py

# Extension ready
# Table created
# Inserted:
# id_upid=user_2acqxw7dpigf2y345ug7ia
# id_uuid=01916ef0-a9ab-98b0-7822-1e985ed61576
# id_text=user_2acqxw7dpigf2y345ug7ia
```
