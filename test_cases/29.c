/* 8.1 再帰関数 */
int factorial(int n) {
    if (n <= 1) {
        return 1;
    } else {
        return n * factorial(n - 1);
    }
}

int main(void) {
    int result;
    result = factorial(5);
    return 0;
}
