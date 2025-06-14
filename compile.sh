#!/bin/bash

# 出力ディレクトリを作成

rm -rf ./out/
mkdir -p ./out/

# 引数を取得
input=$1

# 引数がない場合はエラー
if [ -z "$input" ]; then
  exit 1
fi

# cargoを実行して出力をファイルに書き込む
cargo run  "$(cat $input)";