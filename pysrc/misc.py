import sys
import json
import dataclasses
from credentials import *
from google.cloud import storage as gcs

# Rustのenum型と共通になるように読みだします
# https://igaguri.hatenablog.com/entry/2018/12/28/120500
def make_ok_result():
    return '{"Ok":"()"}'

def make_error_result(e):
    return '{"Err":"' + "{}".format(e) + '"}'

def convert_to_command(line):
    try:
        dict = json.loads(line)
    except:
        raise Exception("cannot parse json")

    try:
        if "Upload" in dict:
            return Upload(**dict["Upload"])
    except:
        raise Exception("cannot convert command")

    raise Exception("cannot detect command type")

# アップロード処理
@dataclasses.dataclass
class Upload:
    source: str
    destination: str
    content_type: str = "application/octet-stream"

def run_upload(cmd):
    client = gcs.Client(project_id)
    bucket = client.get_bucket(bucket_name)
    blob = bucket.blob(cmd.destination)
    blob.upload_from_filename(cmd.source, content_type=cmd.content_type)

# コマンド実行
def run_command(cmd):
    if isinstance(cmd, Upload):
        return run_upload(cmd)

    raise Exception("Unexpected command {}".format(cmd))

# REPLループ
def run_repl(line):
    # 入力の読み込み
    try:
        command = convert_to_command(line)
    except Exception as e:
        return make_error_result(e)

    # コマンド実行
    try:
        run_command(command)
    except Exception as e:
        return make_error_result(e)

    # 何事もなければ終わり
    return make_ok_result()

# コマンドのREPLループです
# Rustから処理しづらいものを標準入出力経由でむりやり実行します
def command_gcs(args):
    client = gcs.Client(project_id)
    bucket = client.get_bucket(bucket_name)

    line = sys.stdin.readline()
    while line:
        result = run_repl( line )
        print(result)
        line = sys.stdin.readline()



# 単発アップロードです
# パフォーマンスそんなに良くないけど、とりあえず簡単に使えるものになります
def command_upload(args):
    client = gcs.Client(project_id)
    bucket = client.get_bucket(bucket_name)
    blob = bucket.blob(args.destination)
    blob.upload_from_filename(args.source, content_type=args.content_type)

def command_download(args):
    client = gcs.Client(project_id)
    bucket = client.get_bucket(bucket_name)
    blob = bucket.blob(args.source)
    blob.download_to_filename(args.destination)
