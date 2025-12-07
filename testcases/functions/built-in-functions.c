// 型まで正しく表記しないと呼べない
// 例　void exit(int) <- NG

void putchar(char);
char getchar(void);
void exit(void);

int main(void) {
    // io
    {
        putchar(getchar());
    }

    // exit sample
    {
        char c;
        while (1) {
            c = getchar();
            if (c == '.') {
                exit();
            }
            putchar(c);
        }

        putchar('u');
        putchar('n');
        putchar('r');
        putchar('e');
        putchar('a');
        putchar('c');
        putchar('h');
        putchar('a');
        putchar('b');
        putchar('l');
        putchar('e');
        putchar('!');
        putchar('\n');
    }

    return 0;
}
