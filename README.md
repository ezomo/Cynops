# Rust C Compiler

C言語で書かれたソースコードを **LLVM IR** に変換する Rust 製の簡易コンパイラです。  
[低レイヤを知りたい人のためのコンパイラ作成入門](https://www.sigbus.info/compilerbook) の第13章までをベースに構築しています。

---

## 特徴

- **Rust製** の構文解析とコード生成
- **llc / clang** により LLVM IR → 実行ファイル への変換が可能
- 計算結果は **終了コード（exit code）** に出力
- C言語風の構文を **コマンドライン引数** から受け取り、標準出力に LLVM IR を出力

---

## 対応している構文

```bnf
program    = stmt*
stmt       = expr ";"
             | "return" expr ";"
             | "if" "(" expr ")" stmt ("else" stmt)?
             | "while" "(" expr ")" stmt
             | "for" "(" expr? ";" expr? ";" expr? ")" stmt
             | "{" stmt* "}"

expr       = assign
assign     = equality ("=" assign)?
equality   = relational ("==" relational | "!=" relational)*
relational = add ("<" add | "<=" add | ">" add | ">=" add)*
add        = mul ("+" mul | "-" mul)*
mul        = unary ("*" unary | "/" unary)*
unary      = ("+" | "-")? primary
primary    = num | ident | "(" expr ")"

```


---

## 制限事項

- `return` の結果は **プロセスの終了コード** として返されるため、**有効値は 0〜255** に限られます。
- 以下の機能は未対応です：
  - 関数定義・関数呼び出し
  - ポインタ、配列、構造体
  - グローバル変数、ヘッダ、標準ライブラリ

---

## 依存ツール

| ツール   | 説明                                     |
|----------|------------------------------------------|
| Rust     | 構文解析・LLVM IR生成（自作コンパイラ）  |
| llc      | LLVM IR (.ll) → アセンブリ (.s) 変換     |
| clang    | アセンブリ (.s) → 実行ファイル    |

---

## 実行方法

```sh
./compile.sh input.txt      # C風ソースをLLVM IRに変換し、アセンブリ・実行ファイルを生成
./run.sh ./out/result       # 実行し、終了コードに結果を出力
```
## example (gcd)
```
a = 100;
b = 35;

temp = 0;

if (a < b) {
    temp = a;
    a = b;
    b = temp;
}

while (b > 0) {
    r = a;
    
    while (r >= b) {
        r = r - b;
    }
    
    a = b;
    b = r;
}

return a;
```
