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