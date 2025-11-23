void exit(void);
void putchar(int a);

int sgn(int);
int abs(int);

int sgn(int a) {
    int r = 1;
    if (32767 <= a) {
        r = 0;  // 負数
    }

    return r;
}

int abs(int x) {
    int n = x;
    if (sgn(x) == 0) {
        n = 0 - x;
    }
    return n;
}

int not(int a) {
    if (a >= 32768) {
        return 0;  // 負
    } else
        return 1;  // 正
}

// a > b
int Greater(int a, int b) {
    int a_abs = abs(a);
    int a_sgn = sgn(a);

    int b_abs = abs(b);
    int b_sgn = sgn(b);

    if (a_sgn < b_sgn) {
        return 0;
    } else if (a_sgn > b_sgn) {
        return 1;
    } else {
        if (a_sgn == 0) {
            return not(a > b);
        }

        return (a > b);
    }
    return 0;
}

int Less(int a, int b) { return Greater(b, a); }

int GreaterEqual(int a, int b) { return Greater(a, b) || a == b; }

int LessEqual(int a, int b) { return Less(a, b) || a == b; }

int Ternary(int a, int b, int c) {
    if (a != 0) {
        return b;
    } else {
        return c;
    }
}

int Slash(int a, int b) {
    int a_abs = abs(a);
    int b_abs = abs(b);

    if (b_abs == 0) {  //
        char error[23] = {'\n', 'e', 'r', 'r', 'o', 'r,', ':', ' ',
                          'd',  'i', 'v', 'i', 'd', 'e',  ' ', 'b',
                          'y',  ' ', 'z', 'e', 'r', 'o',  '\n'};
        int i = 0;
        for (i = 0; i < 23; i++) {
            putchar((int)error[i]);
        }

        exit();
    }

    int r = 1;

    // 標準で使える/記号はなぜかb=1の時機能しなかった．
    if (b_abs == 1) {
        r = a_abs;
    } else {
        r = a_abs / b_abs;
    }

    if (sgn(a) != sgn(b)) {
        return -r;
    } else {
        return r;
    }
}

int Mod(int a, int b) {
    if (b == 0) {
        char error[23] = {'\n', 'e', 'r', 'r', 'o', 'r', ':', ' ',
                          'm',  'o', 'd', 'u', 'l', 'o', ' ', 'b',
                          'y',  ' ', 'z', 'e', 'r', 'o', '\n'};
        int i = 0;
        for (i = 0; i < 23; i++) {
            putchar((int)error[i]);
        }
        exit();
    }

    int q = Slash(a, b);
    int r = a - q * b;

    if (Less(r, 0)) r += Ternary(Greater(b, 0), b, -b);
    return r;
}