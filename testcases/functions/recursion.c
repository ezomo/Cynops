void putchar(char);
// ↓これが再帰であることは一旦無視する
void print_int_core(int x) {
    if (x < 0) {
        putchar('-');
        x = -x;
    }

    if (x >= 10) {
        print_int_core(x / 10);
    }
    // キャストを適当に作っているので明示的に
    putchar((char)((int)'0' + (x % 10)));
    return;
}
void print_int(int a) {
    print_int_core(a);
    putchar('\n');
    return;
}

int gcd(int a, int b) {
    if (b == 0) return a;
    return gcd(b, a % b);
}

int sum(int (*array)[0], int n) {
    if (n == 0) return 0;
    return (*array)[n - 1] + sum(array, n - 1);
}

void main(void) {
    print_int(gcd(48, 18));   // 6
    print_int(gcd(33, 121));  // 11

    int array[] = {1, 2, 3, 5};  // 11
    print_int(sum((int (*)[0]) & array, 4));

    return;
}