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

void main(void) {
    // 出力チェック
    {
        print_int(1);
        print_int(-1);

        print_int(500);
        print_int(-500);
    }

    // 足し算
    {
        print_int(1 + 1);
        print_int(500 + 500);

        print_int(1 + 1 + 1);
        print_int(500 + 500 + 500);
    }

    // 引き算
    {
        print_int(1 - 1);
        print_int(500 - 500);

        print_int(1 - 1 - 1);
        print_int(500 - 500 - 500);
    }

    // 掛け算
    {
        print_int(1 - 1);
        print_int(500 - 500);

        print_int(1 - 1 - 1);
        print_int(500 - 500 - 500);
    }

    // 割り算
    {
        {
            print_int(1 / 1);
            print_int(4 / 2);
            print_int(500 / 200);
            print_int(1 / 500);
        }

        {
            print_int(-1 / 1);
            print_int(-4 / 2);
            print_int(-500 / 200);
            print_int(-1 / 500);
        }

        {
            print_int(1 / -1);
            print_int(4 / -2);
            print_int(500 / -200);
            print_int(1 / -500);
        }

        {
            print_int(-1 / -1);
            print_int(-4 / -2);
            print_int(-500 / -200);
            print_int(-1 / -500);
        }
    }

    // 掛け算
    {
        {
            print_int(1 * 1);
            print_int(4 * 0);
            print_int(200 * 100);
            print_int(1 * 500);
        }

        {
            print_int(-1 * 1);
            print_int(-4 * 0);
            print_int(-200 * 100);
            print_int(-1 * 100);
        }

        {
            print_int(1 * -1);
            print_int(4 * 0);
            print_int(200 * -100);
            print_int(1 * -100);
        }

        {
            print_int(-1 * -1);
            print_int(-4 * 0);
            print_int(-200 * -100);
            print_int(-1 * -100);
        }
    }

    // mod
    {
        {
            print_int(1 % 1);
            print_int(4 % 2);
            print_int(500 % 200);
            print_int(1 % 500);
        }

        {
            print_int(-1 % 1);
            print_int(-4 % 2);
            print_int(-500 % 200);
            print_int(-1 % 500);
        }

        {
            print_int(1 % -1);
            print_int(4 % -2);
            print_int(500 % -200);
            print_int(1 % -500);
        }

        {
            print_int(-1 % -1);
            print_int(-4 % -2);
            print_int(-500 % -200);
            print_int(-1 % -500);
        }
    }

    return;
}
