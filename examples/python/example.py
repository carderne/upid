import psycopg
from psycopg.conninfo import make_conninfo

from upid import upid

conninfo = make_conninfo(
    host="localhost",
    port=5432,
    user="postgres",
    password="mypassword",
    dbname="postgres",
)

id_obj = upid("user")

with psycopg.connect(conninfo) as conn:
    create_ext = "CREATE EXTENSION IF NOT EXISTS upid_pg;"
    conn.execute(create_ext)
    print("Extension ready")

    drop_table = "DROP TABLE IF EXISTS test_upid;"
    conn.execute(drop_table)

    create_table = """
    CREATE TABLE test_upid (
        id_upid upid NOT NULL,   -- pass a string
        id_uuid uuid NOT NULL,   -- pass binary
        id_text text NOT NULL    -- pass a string
    );
    """
    conn.execute(create_table)
    print("Table created")

    query = """
    INSERT INTO test_upid (id_upid, id_uuid, id_text)
    VALUES (%s, %s, %s)
    RETURNING *;
    """
    values = (
        id_obj.to_str(),           # string for upid type
        id_obj.to_uuid(),          # uuid for uuid type
        id_obj.to_str(),           # string for text type
    )

    row = conn.execute(query, values).fetchone()
    if row:
        print(f"Inserted:\nid_upid={row[0]}\nid_uuid={row[1]}\nid_text={row[2]}")
