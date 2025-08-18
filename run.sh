#!/bin/bash

# 引数がない場合はエラー
if [ -z "$1" ]; then
  echo "Usage: $0 <input>"
  exit 1
fi

# 出力ディレクトリを作成
rm -rf ./out/
mkdir -p ./out/

# cargoを実行してLLVM IRを生成
cargo run "$1" > ./out/result.ll

# アセンブリ生成 & 実行ファイル作成
llc ./out/result.ll
clang ./out/result.s -o ./out/result

# 実行 & 終了コード表示
./out/result
echo $?