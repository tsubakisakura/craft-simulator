import ulid
import time
import subprocess
import sys

import train
import tools
from credentials import *

from google.cloud import storage as gcs

def command_new(args):
    # 新規モデルを作成
    trainer = train.Trainer()
    frozen_model = trainer.get_frozen_model()
    train.export_frozen_model(frozen_model,args.filename)

    # 新規モデルの構造の表示
    trainer.model.summary()
    print(f"{frozen_model.inputs}")
    print(f"{frozen_model.outputs}")

def command_fit(args):
    trainer = train.Trainer()
    samples = train.read_samples(args.source)
    buffer = train.ReplayBuffer(sys.maxsize);
    buffer.add_samples( samples )
    trainer.train( buffer.states, buffer.policies, buffer.values, args.epochs )
    frozen_model = trainer.get_frozen_model()
    train.export_frozen_model(frozen_model,args.destination)

# 最新サンプルファイルを全部取得します。
def get_new_samples(client, bucket, last_name):
    prefix = "sample/"

    # 辞書順なので、ここでなんか適当な文字を付ければ、次の範囲を列挙することができます
    start = last_name + "0" if last_name != None else None

    # python の google-cloud-storage で バケット内のオブジェクトを start_offset, end_offset, (prefix)を使って直近の日付ファイル名に絞り込む
    # https://qiita.com/munaita_/items/caa77fe17d99649c67d1
    return [blob for blob in client.list_blobs(bucket, prefix=prefix, start_offset=start)]

# GCSから指定ファイルをダウンロードしてnumpy配列として取得します。
def download_samples(blob):
    # ダウンロード
    print( "download: {}".format(blob.name) )
    blob.download_to_filename("sample.txt.bz2")

    # 展開
    proc = subprocess.run(["bzip2","-d","-f","sample.txt.bz2"], stdout=sys.stdout)
    if proc.returncode != 0:
        raise Exception("bzip2 has exit with error code:" + str(proc.returncode))

    # 展開したファイルからサンプルデータを読み込み
    return train.read_samples("sample.txt")

# 新規blobを使ってバッファを埋めます。
# 後ろからダウンロードしてバッファを埋めて、バッファを埋めきったらそこで処理が停止します。
# バッファの更新方法は色々あるのですが現状はソースの通りの方法にしています。他には
# 1. 先行dropは特にしなくても別に要らないのでは
# 2. add_samplesではdropせずに追記だけしていき最後に余ったのをdropすればよいのではないか
# などがあります(メモリ断片化とか最大メモリ保存量とかまで考えるとどういうのが良いかよく分からない
def add_samples_from_blobs(buffer, blobs, read_sum):
    if blobs == []:
        return

    samples = download_samples(blobs[-1])
    read_sum = read_sum + len(samples)
    buffer.drop(len(samples))

    if read_sum < buffer.max_length:
        add_samples_from_blobs(buffer, blobs[:-1], read_sum)

    buffer.add_samples(samples, blobs[-1].name)

# 何かモデルがあるか調べます
def is_exist_model(connection):
    with connection.cursor() as cursor:
        cursor.execute("SELECT COUNT(*) FROM evaluation")
        (x,) = cursor.fetchone()
        connection.commit()
        return x != 0

    raise Exception("cannot get model count")

# frozen_modelを出力します
def output_frozen_model(client, bucket, connection, frozen_model):
    # 出力します。
    train.export_frozen_model(frozen_model,"frozen_model.pb")
    model_blob = bucket.blob("model/" + ulid.new().str)
    model_blob.upload_from_filename("frozen_model.pb", content_type="application/x-protobuf")

    # 最初の初期状態のレコードを作ります。
    with connection.cursor() as cursor:
        cursor.execute("INSERT INTO evaluation (name,total_reward,total_count) VALUES (%s,0,0)", (model_blob.name))
        connection.commit()

# 1エピソードぶんの処理です。
def run_episode(client, bucket, connection, trainer, buffer, epochs):
    # GCSからファイル一覧を読み取ります。
    print( "enumerate sample files from GCS" )
    blobs = get_new_samples( client, bucket, buffer.last_name )

    # リプレイバッファを更新します。
    print( "download samples..." )
    add_samples_from_blobs( buffer, blobs, 0 )

    if not buffer.is_empty():
        # バッファに何かあるなら学習します。
        print( "Start training..." )
        trainer.train( buffer.states, buffer.policies, buffer.values, epochs )
        output_frozen_model( client, bucket, connection, trainer.get_frozen_model() )
    elif not is_exist_model(connection):
        # バッファに何もなく、モデルもないなら、今のモデルを初期状態として出力します。
        print( "Output initial graph..." )
        output_frozen_model( client, bucket, connection, trainer.get_frozen_model() )
    else:
        # バッファに何もないんだけど、モデルはある状態です。
        # evaluatorとgeneratorは最良モデルを利用して計算をしているはずなので新しいサンプルを待機します。
        print("Wait for new samples...")
        time.sleep(3)

def run_with_connection(args, connection):
    client = gcs.Client(project_id)
    bucket = client.get_bucket(bucket_name)

    trainer = train.Trainer()
    buffer = train.ReplayBuffer(args.buffer)

    while True:
        run_episode( client, bucket, connection, trainer, buffer, args.epochs )

# GCSに存在するサンプルデータを元に学習し続けるデーモンです。
def command_learner(args):
    tools.with_ssh_tunnel( lambda: tools.with_mysql_connection( lambda connection: run_with_connection( args, connection ) ) )
