
void putchar(char);
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

int add(int a, int b) { return (a + b); }
int sub(int a, int b) { return (a - b); }
int mul(int a, int b) { return (a * b); }
int div(int a, int b) { return (a / b); }
int mod(int a, int b) { return (a % b); }

int apply(int a, int b, int (*fn)(int, int)) { return (*fn)(a, b); }

void main(void) {
    int a = 5, b = 5;
    print_int(apply(a, b, &add));
    print_int(apply(a, b, &sub));
    print_int(apply(a, b, &mul));
    print_int(apply(a, b, &div));
    print_int(apply(a, b, &mod));

    return;
}