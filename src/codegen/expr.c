int sgn(int);
int abs(int);
void putchar(int a);

int sgn(int a) {
    int r = 1;
    if (32767 <= a && a <= 65535) {
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
