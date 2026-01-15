void putchar(char);
char getchar(void);
void exit(void);

void print_int(int x) {
    if (x < 0) {
        putchar('-');
        x = -x;
    }

    if (x >= 10) {
        print_int(x / 10);
    }
    // キャストを適当に作っているので明示的に
    putchar((char)((int)'0' + (x % 10)));
    return;
}

void print_hex_digit(int digit) {
    if (digit < 10) {
        putchar('0' + (char)digit);  // 0-9
    } else {
        putchar('a' + (char)(digit - 10));  // A-F
    }

    return;
}

// intを16進数で表示する関数（再帰版）
void print_hex_recursive(int n, int digits) {
    if (digits > 0) {
        // 上位の桁を先に再帰的に処理
        print_hex_recursive(n >> 4, digits - 1);
        // 現在の最下位4ビットを表示
        print_hex_digit(n & 15);
    }
    return;
}

void print_hex(int n) {
    // "0x"を表示
    putchar('0');
    putchar('x');

    // 4桁の16進数として表示
    print_hex_recursive(n, 4);
    putchar('\n');

    return;
}

int main(void) {
    int sum1 = 255;
    int sum2 = 255;
    char c;

    while ((c = getchar()) != '\0') {
        sum1 = (sum1 + (int)c) % 255;
        sum2 = (sum2 + sum1) % 255;
    }

    int hash = (sum2 << 8) | sum1;

    print_hex(hash & 65535);

    return 0;
}

// cat ./sample/osero.c | {./run.sh ./sample/fletcher16.c}