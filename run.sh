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
cargo run "$1" codegen > ./out/result.ll

opt -O2 ./out/result.ll -S -o ./out/optimized.ll
# アセンブリ生成 & 実行ファイル作成
llc ./out/optimized.ll
clang ./out/optimized.s -o ./out/result

# 実行 & 終了コード表示
./out/result
# echo $?