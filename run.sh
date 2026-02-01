#!/bin/bash

# # 引数がない場合はエラー
if [ -z "$1" ]; then
  echo "Usage: $0 <input>"
  exit 1
fi

# 出力ディレクトリを作成
rm -rf ./out/
mkdir -p ./out/

cargo run "$1" codegen > ./out/result.txt
gcc -DMASK=2 hydrogen.c -o brainfuck
./brainfuck ./out/result.txt
