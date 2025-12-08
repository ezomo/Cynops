void putchar(char);
void print_int(int);
typedef struct {
    int integer_part;
    int decimal_part;
} Double;

void print_double(Double dou) {
    print_int(dou.integer_part);
    putchar('.');
    print_int(dou.decimal_part);
    return;
}

Double InitDouble(int num, int dp) {
    Double tmp = {num, dp};
    return tmp;
}

int DoubleGreater(Double a, Double b) {
    // まず整数部で比較
    if (a.integer_part > b.integer_part) return 1;
    if (a.integer_part < b.integer_part) return 0;

    // 整数部が同じなら小数部で比較
    if (a.decimal_part > b.decimal_part) return 1;
    return 0;
}
int DoubleLess(Double a, Double b) { return DoubleGreater(b, a); }

int DoubleLess(Double a, Double b) { return DoubleGreater(b, a); }

int DoubleEqual(Double a, Double b) {
    return (a.integer_part == b.integer_part) &&
           (a.decimal_part == b.decimal_part);
}