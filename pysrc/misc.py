from credentials import *
from google.cloud import storage as gcs

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
