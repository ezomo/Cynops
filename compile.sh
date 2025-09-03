#!/bin/bash
# 出力ディレクトリを作成



# cargoを実行して出力をファイルに書き込む
llc ./out/result.ll
clang ./out/result.s -o ./out/result