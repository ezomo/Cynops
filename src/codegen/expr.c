int sgn(int);
int abs(int);

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
    if (a == 1) {
        return 0;
    } else {
        return 1;
    }
}

// a > b
int Greater(int a, int b) { return 3; }

int Less(int a, int b) { return 3; }

int GreaterEqual(int a, int b) { return 3; }

int LessEqual(int a, int b) { return 3; }
