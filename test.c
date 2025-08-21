
// int swap(int *x, int *y) {
//     int e = 0;
//     e = *y;
//     *y = *x;
//     *x = e;

//     return 0;
// }

// test関数：二重ポインタ配列の中の特定要素を返す
int test(int (*(*x)[3])[2][2]) {
    // 2番目の配列、1番目の2x2配列の[1][0] を返す
    return (*(*x)[1])[1][0];
}

int main(void) {
    // 2x2 の配列を3つ用意
    int a[2][2] = {{1, 2}, {3, 4}};
    int b[2][2] = {{5, 6}, {7, 8}};
    int c[2][2] = {{9, 10}, {11, 12}};

    // 2x2 配列へのポインタを格納した配列
    int (*arr[3])[2][2] = {&a, &b, &c};

    // arr 自体のポインタを test に渡す
    int result = test(&arr);

    return result;
}