import numpy as np
import tensorflow as tf
import tensorflow.keras.layers as kl

from tensorflow.python.framework.convert_to_constants import convert_variables_to_constants_v2
from numpy.testing import assert_array_equal, assert_allclose

# 定数定義
STATE_NUM=36
ACTION_NUM=32

# リプレイバッファ
# 専用のデータ構造として扱いやすい構造にしておきます
class ReplayBuffer:
    def __init__(self, max_length):
        self.states = np.empty((0,STATE_NUM), dtype='float32')
        self.policies = np.empty((0,ACTION_NUM), dtype='float32')
        self.values = np.empty((0,1), dtype='float32')
        self.max_length = max_length
        self.last_name = None # 凝集度が低いんだけど、ここにファイル名を記録しておきます(参照しやすいので)

    def is_empty(self):
        return len(self.states) == 0

    def drop(self, size):
        # pythonのスライスの範囲。引数1の場合は(:stop)と等価になりますので先頭から削除されていきます
        # https://note.nkmk.me/python-slice-usage/
        #
        # なお配列よりも大きなスライスで削除しても空配列になるだけで処理は有効です
        remove_slice = slice(size)
        self.states = np.delete(self.states,remove_slice, axis=0)
        self.policies = np.delete(self.policies,remove_slice, axis=0)
        self.values = np.delete(self.values,remove_slice, axis=0)

    def add_samples(self, samples, last_name):
        # データを入力と出力に分解します。結果はstate, policy, valueの順に出力されます
        new_states, new_policies, new_values = np.hsplit(samples,[STATE_NUM,STATE_NUM+ACTION_NUM])
        self.states = np.append(self.states, new_states, axis=0)
        self.policies = np.append(self.policies, new_policies, axis=0)
        self.values = np.append(self.values, new_values, axis=0)
        self.last_name = last_name

        if len(self.states) > self.max_length:
            self.drop(len(self.states) - self.max_length)

# TSV出力されたサンプルファイルから読み込みます。
def read_samples(filename):
    # デフォルトは"float"データ型らしいのだけれど、floatのビット幅が環境依存らしいので、直接float32を指定して読み込みます
    return np.loadtxt(filename, dtype='float32');

# NNで学習するためのモデルを作ります。
def create_model():
    # とりあえず雑なモデルで作ってみる
    inputs = kl.Input(shape=(STATE_NUM,), name='inputs')
    x = kl.Dense(128, activation='relu')(inputs)
    x = kl.Dense(128, activation='relu')(x)
    x = kl.Dense(128, activation='relu')(x)
    x = kl.Dense(128, activation='relu')(x)
    policy = kl.Dense(ACTION_NUM, activation='softmax', name='policy')(x)
    value = kl.Dense(1, activation='sigmoid', name='value')(x)

    # モデル構築
    return tf.keras.models.Model(inputs=inputs, outputs=[policy,value])

# 損失関数
# AlphaZeroの場合、valueとpolicyで別々の損失関数を与えて総和を取るので、policyのlossは定義します
# https://tadaoyamaoka.hatenablog.com/entry/2017/10/24/000521
# 軸0がミニバッチの大きさを示しているはずなので、軸1で総和して、軸0でreduce_meanで平均します
def loss_policy(y_true, y_pred):
    return tf.reduce_mean( tf.reduce_sum( -y_true * tf.math.log(y_pred + 0.0001), axis=1 ) )

# 訓練システム
# 何度も tf.function を使うと重くなるよとtensorflowに怒られてしまうため、
# concrete_modelを一度作り、それを再利用し続けるためのコンテキストになります。
class Trainer:
    def __init__(self):
        # 初期モデルを作成
        self.model = create_model()

        # keras modelを計算グラフを取得するために TF2 function モデルに変換します
        x = tf.TensorSpec((None,STATE_NUM), tf.float32, name="frozen_graph_input")
        self.graph_model = tf.function( lambda x: self.model(x) )

        # 具象関数を取得します。
        self.concrete_model = self.graph_model.get_concrete_function(x=x)

        # 学習用モデルのコンパイル
        # このニューラルネットは単純なクラス分類問題ではなく損失関数の最小化だけできてればよいので
        # metricsを改めて表示することはせずシンプルに損失関数の情報だけ出力します
        self.model.compile(optimizer="adam", loss=[loss_policy,'mean_squared_error'])

    def train( self, source_states, source_policy, source_value, epochs ):
        self.model.fit(x=source_states,y=[source_policy,source_value], batch_size=512, epochs=epochs, shuffle=True )

    def get_frozen_model(self):
        # 具象関数の変数を定数に固定します。
        return convert_variables_to_constants_v2(self.concrete_model)

# frozen modelをファイル出力します。
def export_frozen_model(frozen_model, filename):
    tf.io.write_graph(frozen_model.graph, ".", filename, as_text=False)
