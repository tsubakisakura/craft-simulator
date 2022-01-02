import sshtunnel

from credentials import *

from google.cloud import secretmanager as sm

def get_secret(name):
    client = sm.SecretManagerServiceClient()
    path = client.secret_version_path(project_id, name, "latest")
    response = client.access_secret_version(name=path)
    return response.payload.data.decode('UTF-8')

def get_mysql_password():
    return get_secret(mysql_password_name)

def with_ssh_tunnel(func):
    print("get ssh secrets...")
    ssh_private_key = get_secret(ssh_private_key_name)

    with open("./ssh_private_key", mode='w') as f:
        f.write(ssh_private_key)

    print("connecting ssh...")
    with sshtunnel.open_tunnel(
        (mysql_hostname,22),
        ssh_username=ssh_username,
        ssh_pkey="./ssh_private_key",
        remote_bind_address=('127.0.0.1', 3306),
        local_bind_address=('0.0.0.0', 3306) ) as tunnel:

        return func()
