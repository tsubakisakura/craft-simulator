import argparse
import importlib
import sys

def command_upload(args):
    print("loading modules...", file=sys.stderr)
    misc = importlib.import_module('misc')
    print("done.", file=sys.stderr)
    misc.command_upload(args)

def command_download(args):
    print("loading modules...", file=sys.stderr)
    misc = importlib.import_module('misc')
    print("done.", file=sys.stderr)
    misc.command_download(args)

# Pythonのargparseでサブコマンドを実現する
# https://qiita.com/oohira/items/308bbd33a77200a35a3d
def main():
    parser = argparse.ArgumentParser(description='Learn neural network.')
    # 本当ならsubparserを作るときにrequiredフラグを入れたいのですがバグで入れられないようです。
    # このため、もしhandlerが手に入らなかった場合にヘルプを表示します。詳細はこちらのURLを参照。
    # https://bugs.python.org/issue33109
    subparser = parser.add_subparsers()

    parser_upload = subparser.add_parser('upload', help='upload to gcs.')
    parser_upload.add_argument('source', type=str, help='source path.')
    parser_upload.add_argument('destination', type=str, help='destination path.')
    parser_upload.add_argument('--content-type', type=str, default='application/octet-stream', help='content type.')
    parser_upload.set_defaults(handler=command_upload)

    parser_download = subparser.add_parser('download', help='download to gcs.')
    parser_download.add_argument('source', type=str, help='source path.')
    parser_download.add_argument('destination', type=str, help='destination path.')
    parser_download.set_defaults(handler=command_download)

    args = parser.parse_args()
    if hasattr( args, "handler" ):
        args.handler(args)
    else:
        parser.print_help()

if __name__ == '__main__':
    main()
