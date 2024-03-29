###############################################################################
# 共通の基本レイヤー
###############################################################################
FROM nvidia/cuda:11.3.0-devel-ubuntu20.04 as base
WORKDIR /workdir

RUN apt-get update \
 && apt-get install -y \
        curl \
 && apt-get clean

###############################################################################
# 依存ライブラリのビルド
###############################################################################
FROM base as dependencies

# rustup を非対話的環境でインストールする方法
# https://qiita.com/maguro_tuna/items/f69b2e41f753d2ff0cc2
# When installing Rust toolchain in Docker, Bash `source` command doesn't work
# https://stackoverflow.com/questions/49676490/when-installing-rust-toolchain-in-docker-bash-source-command-doesnt-work
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

RUN apt-get update \
 && apt-get install -y \
        build-essential \
        libssl-dev \
        wget \
        unzip \
 && apt-get clean

# rust で failed to run custom build command for `openssl-sys` が出るときにすること
# https://qiita.com/nacika_ins/items/465e89a7b3fbeb373605
ENV OPENSSL_LIB_DIR=/usr/lib/x86_64-linux-gnu/
ENV OPENSSL_INCLUDE_DIR=/usr/include/openssl

# libtorchのインストール
# cxx11-abiである必要があります。通常バージョンのほうだとリンク時にエラーした
# CPU版ではこちら
#RUN wget -q https://download.pytorch.org/libtorch/cpu/libtorch-cxx11-abi-shared-with-deps-1.10.1%2Bcpu.zip \
# && unzip libtorch-cxx11-abi-shared-with-deps-1.10.1+cpu.zip -d /usr/local \
# && rm libtorch-cxx11-abi-shared-with-deps-1.10.1+cpu.zip
# GPU版ではこちら
RUN wget -q https://download.pytorch.org/libtorch/cu113/libtorch-cxx11-abi-shared-with-deps-1.10.1%2Bcu113.zip \
 && unzip libtorch-cxx11-abi-shared-with-deps-1.10.1+cu113.zip -d /usr/local \
 && rm libtorch-cxx11-abi-shared-with-deps-1.10.1+cu113.zip

ENV LIBTORCH /usr/local/libtorch
ENV TORCH_CUDA_VERSION=cu113

###############################################################################
# メインのビルド作業
###############################################################################

FROM dependencies as build

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

###############################################################################
# 実行コンテナ作成
###############################################################################
FROM base

RUN apt-get update \
 && apt-get install -y \
        python3 \
        python3-pip \
 && apt-get clean

RUN pip install --upgrade pip \
 && pip install --no-cache-dir \
        sshtunnel \
        google-cloud-secret-manager \
        google-cloud-storage

ENV LIBTORCH=/usr/local/libtorch
ENV TORCH_CUDA_VERSION=cu113
ENV LD_LIBRARY_PATH=/usr/local/libtorch/lib:${LD_LIBRARY_PATH}

COPY --from=dependencies /usr/local/libtorch/lib/*.so* /usr/local/libtorch/lib/
COPY --from=build /workdir/target/release/craft-simulator ./target/release/
COPY ./pysrc/ ./pysrc
CMD ["/bin/bash"]
