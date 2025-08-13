
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
## do while
```c
do {
  A;
} while (B);
```
```llvm
  ; 本体の処理
  <A> 
  ; 条件の評価
  %cond_val = <B> ; i1 型の条件式
  ; 条件による分岐
  br i1 %cond_val, label %body_label, label %end_label
:body_label:
  ; 本体の処理
  <A>
  ; 条件の再評価
  br label %cond_label
:end_label:
  ; 次の処理へ
```

## break
```c
break;
```
```llvm
  ; break の処理
  br label %end_label
```
## continue
```c
continue;
```
```llvm
  ; continue の処理
  br label %cond_label
``` 

## Switch
```c
switch (A) {
  case B:
    C;
    break;
  case D:
    E;
    break;
  default:
    F;
}
```
```llvm
  ; 条件の評価
  %cond_val = <A> ; i32 型の条件式
  ; switch の開始
  switch i32 %cond_val, label %default_label [
    i32 <B>, label %case_b_label
    i32 <D>, label %case_d_label
  ]
case_b_label:
    ; case B の処理
    <C>
    br label %end_label
    br label %case_d_label
case_d_label:
    ; case D の処理
    <E>
    br label %end_label
default_label:
    ; default の処理
    <F>
    br label %end_label
end_label:
  ; 次の処理へ
```

## goto
```c
goto label;
label:
```
```llvm
  ; goto の処理
  br label %label_name
label_name:
  ; ラベルの処理
```

## label
```c
label:
```
```llvm
  ; ラベルの定義
  br label %label_name
  label_name:
  ; ラベルの処理
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

int *p = &a;
int *q;

int a[2];
int b[2] = {1, 2};
a[0] = 1000;

int a[2][3];
a[1][2] = 100;


int b;
int *a[2];
a[1] = &b;
```
```llvm
  ; 変数の宣言と初期化
  %a = alloca i32
  store i32 3, ptr %a

  ; 変数の宣言（初期化なし）
  %b = alloca i32

  ; ポインタ変数の宣言と初期化
  %p = alloca ptr
  store ptr %a, ptr %p

  ; ポインタ変数の宣言（初期化なし）
  %q = alloca ptr

  ; 配列の宣言
  %arr_a = alloca [2 x i32]
  %arr_b = alloca [2 x i32]
  ; 配列の初期化
  store [2 x i32] [i32 1, i32 2], ptr %arr_b
  ; 配列の要素へのアクセス
  %arr_a_elem0 = getelementptr inbounds [2 x i32], ptr %arr_a, i64 0, i64 0
  store i32 1000, ptr %arr_a_elem0

  ; 二次元配列の宣言
  %arr_2d = alloca [2 x [3 x i32]]
  ; 二次元配列の要素へのアクセス
  %arr_2d_elem = getelementptr inbounds [2 x [3 x i32]], ptr %arr_2d, i64 0, i64 1, i64 2
  store i32 100, ptr %arr_2d_elem

  ; ポインタ配列の宣言
  %arr_ptr = alloca [2 x ptr]
  ; ポインタ配列の要素へのアクセス
  %arr_ptr_elem = getelementptr inbounds [2 x ptr], ptr %arr_ptr, i64 0, i64 1
  store ptr %b, ptr %arr_ptr_elem
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
## poinyter assignment
```c
int a = 0;
int *p = &a;
*p = *p + 1;
```
```llvm
  ; ポインタ変数の宣言と初期化
  %a = alloca i32
  store i32 0, ptr %a
  %p = alloca ptr
  store ptr %a, ptr %p

  ; ポインタの値の読み取り
  %p_val = load ptr, ptr %p
  %val = load i32, ptr %p_val

  ; ポインタの値の更新
  %new_val = add i32 %val, 1
  store i32 %new_val, ptr %p_val
```

## variable assignment with binary operation