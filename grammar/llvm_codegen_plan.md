
# LLVM Code Generation Plan

# control flow
## if　else
```c
if (A) B; else C;
```
```llvm
  ; 条件の評価
  %cond_val = <A> ; i1 型の条件式

  ; 条件による分岐
  br i1 %cond_val, label %then_label, label %else_label

then_label:
  ; <B>
  br label %end_label

else_label:
  ; <C> 
    br label %end_label

end_label:
  ; 次の処理へ
```

## for
```c
for (A; B; C) D;
```
```llvm
  ; 初期化
  <A>

  ; 条件の評価
cond_label:
    %cond_val = <B> ; i1 型の条件式

  ; 条件による分岐
  br i1 %cond_val, label %body_label, label %end_label
body_label:
    ; 本体の処理
    <D>

    ; 増分の評価
    <C>

    ; 条件の再評価
    br label %cond_label
end_label:
    ; 次の処理へ
  ```

## while
```c
while (A) B;
```
```llvm
  ; 条件の評価
:cond_label
    %cond_val = <A> ; i1 型の条件式
  ; 条件による分岐
  br i1 %cond_val, label %body_label, label %end_label
:body_label:
    ;本体の処理
    <B>

    ; 条件の再評価
    br label %cond_label
end_label:
    ; 次の処理へ
```

# declaration
## function declaration
```c
int foo(int a, int b) {
  C;
  return D;
}
```
```llvm
define i32 @foo(i32 %a, i32 %b) {
    %return = alloca i32
  ; 引数の受け取り
  ; %a と %b は関数の引数として自動的に定義される

  ; 本体の処理
  <C>

  ; 戻り値の設定
  store i32 <D>, ptr %return


return_label:
  %val = load i32, ptr %return
　ret i32 %val
}
```

## variable declaration
```c
int a = 3;
int b;
```
```llvm
  ; 変数の宣言と初期化
  %a = alloca i32
  store i32 3, ptr %a

  ; 変数の宣言（初期化なし）
  %b = alloca i32
```


# call
## function call
```c
foo(b, c);
```
```llvm
  ; 関数呼び出し
  %result = call i32 @foo(i32 %b, i32 %c)
```

## variable call
```c
a+3;
```
```llvm
  ; 変数の読み取り
  %a_val = load i32,
  %result = add i32 %a_val, 3
```

# Exper
## Binary Operation
### Arithmetic
```c
3+2;
3-2;
3*2;
3/2;
3%2;
```
```llvm
  %a = add i32 3, 2
  %b = sub i32 3, 2
  %c = mul i32 3, 2
  %d = sdiv i32 3, 2
  %e = srem i32 3, 2
```

## variable assignment
```c
a = 5;
```
```llvm
  ; 変数の値の設定
  store i32 5, ptr %a
```

## variable assignment with binary operation