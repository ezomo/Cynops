# Cynops — A C-to-Brainfuck Compiler

Cynops は、**C 風の言語から Brainfuck へのコンパイラ**です。

> ⚠️ Cynops は「C ライク」な言語ですが、**そのままの C ではありません**。独自の制約と拡張を持ちます。

---

## 概要

このプロジェクトは、C 言語に近い構文を持つ言語を Brainfuck にコンパイルすることを目的としています。

以下の既存プロジェクト・実装に強く影響を受けています：

- **c2bf**\
  [https://github.com/iacgm/c2bf](https://github.com/iacgm/c2bf)\
  一部コードを引用しています。これがなければ本プロジェクトは成立しませんでした。\
  特に、非構造化な制御フローをプログラムカウンタを模した単一の while ループで表現できる、いわゆる Single-while-loop 版 の構造化定理の存在を知れたことが大きな収穫でした。

- **hydrogen.c (Brainfuck interpreter)**\
  [https://github.com/rdebath/Brainfuck/blob/master/extras/hydrogen.c](https://github.com/rdebath/Brainfuck/blob/master/extras/hydrogen.c)\
  高速かつ快適で、Cynops の開発を実用レベルで可能にしてくれました。

Cynops は c2bf をベースに、**ポインタ・配列・関数・高階関数などを拡張**したものと考えてください。

---

## 対応機能

### 基本機能

- 型

  - `int`（16bit 整数）
  - 固定小数点演算（加算・減算）

- 変数

  - 変数の宣言と初期化
  - `typedef` による型エイリアス

- 配列

  - n 次元配列

- ポインタ

  - 任意レベルのポインタ（※ポインタ演算は禁止）

- 構造体

- 関数

  - 通常の関数定義と呼び出し
  - 再帰呼び出し
  - 関数ポインタ
  - 高階関数（関数ポインタを引数に取る関数）

- 制御構文

  - 条件分岐：`if` / `else`
  - 反復：`while` / `for`
  - ループ制御：`break` / `continue`

---

## 追加・独自機能

### 配列は「値」として扱われる

Cynops では、配列をポインタではなく**値として扱います**。

```c
int a[3] = {1, 2, 3};
int b[3] = a;
```

その結果、**配列の代入や返却がそのまま可能**です。

```c
// fn(char [6]) -> char [6]
char (fn(char arg[6]))[6] {
  return arg;
}
```

### ポインタ演算は禁止

ポインタは「参照」としてのみ使用可能で、演算はできません。

```c
int a[3] = {1, 2, 3};
int* p = &a[2];  // OK
p = p + 1;       // NG
```

---

## 実行方法

### コンパイル・実行

```sh
./run.sh <ファイル名>
```

### テストケース実行

```sh
./test.sh
```

---

## デモ

- **オセロ**\
  [https://www.youtube.com/watch?v=i5pTHRw2-z8](https://www.youtube.com/watch?v=i5pTHRw2-z8)

- **画面描画デモ**\
  [https://www.youtube.com/watch?v=kMH9iaTFzEQ](https://www.youtube.com/watch?v=kMH9iaTFzEQ)


