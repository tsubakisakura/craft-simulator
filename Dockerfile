# ビルドキャッシュ作成
#
# 元々rustの公式イメージを利用していましたが、
# rust側がlibpython3.9でtensorflow側がlibpython3.8であり、
# それぞれお互いに互換性のあるライブラリが置いてなかったので、
# 実行環境であるtensorflowベースでRustのビルド環境を構築することにしました。
FROM tensorflow/tensorflow:latest AS build
WORKDIR /workdir

# rustup を非対話的環境でインストールする方法
# https://qiita.com/maguro_tuna/items/f69b2e41f753d2ff0cc2
# When installing Rust toolchain in Docker, Bash `source` command doesn't work
# https://stackoverflow.com/questions/49676490/when-installing-rust-toolchain-in-docker-bash-source-command-doesnt-work
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-get update \
 && apt-get install -y \
        libssl-dev \
 && apt-get clean

# rust で failed to run custom build command for `openssl-sys` が出るときにすること
# https://qiita.com/nacika_ins/items/465e89a7b3fbeb373605
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu/
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl

# 依存ライブラリのビルド
COPY ./Cargo.toml ./Cargo.lock ./
RUN mkdir ./src
RUN echo "fn main() {}" > ./src/main.rs
RUN cargo build --release

# バイナリ作成
# main.rsのタイムスタンプをいじらないとビルドされないらしいです
COPY ./src/ ./src
RUN touch ./src/main.rs
RUN cargo build --release

# コンテナ作成
#
# alpineのシングルバイナリは使えませんでした(libtensorflowとかが静的にならなかったため)
# debianの場合、Rustのほうはlibtensorflow.so等を持ってくれば動いたのですが、
# python側の各種ライブラリのインストールに苦戦してしまったので最終的にtensorflow公式のイメージを使うことにしました。
#
# ただ、Rustのプログラムが参照するlibtensorflowに関しては、LD_LIBRARY_PATHを直接指定することにしています。
# pythonでも別途参照するので競合しないようにするためです。
FROM tensorflow/tensorflow:latest
WORKDIR /workdir
RUN pip install --upgrade pip \
 && pip install --no-cache-dir \
        ulid-py \
        pymysql \
        sshtunnel \
        google-cloud-secret-manager \
        google-cloud-storage
COPY --from=build /workdir/target/release/build/tensorflow-sys-0d5122ccecfa2b3f/out/libtensorflow.so.2 ./target/release/
COPY --from=build /workdir/target/release/build/tensorflow-sys-0d5122ccecfa2b3f/out/libtensorflow_framework.so.2 ./target/release/
COPY --from=build /workdir/target/release/craft-simulator ./target/release/
COPY ./pysrc/ ./pysrc
CMD ["/bin/bash"]
