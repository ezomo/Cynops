/* 7.3 ネストした関数呼び出し */
int add(int a, int b) {
    return a + b;
}

int multiply(int a, int b) {
    return a * b;
}

int main(void) {
    int result;
    result = multiply(add(2, 3), add(4, 5));
    return 0;
}
