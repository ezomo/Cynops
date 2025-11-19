int sgn(int);
int abs(int);
int mul(int, int);
int mod10(int);
void print_num(int);

// putcharと同様，標準の関数
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
    if (a == 1) {
        return 0;
    } else {
        return 1;
    }
}

// a > b
int Gr(int a, int b) {
    int abs_a = abs(a);
    int abs_b = abs(b);
    int r1 = (abs_a > abs_b);
    int sign_a = sgn(a);
    int sign_b = sgn(b);
    if ((sign_a != sign_b)) {
        if (r1 == 1 && sign_a == 0) return not(r1);
        if (r1 == 0 && sign_b == 0) return not(r1);
    } else
        return r1;
}

int mul(int a, int b) {
    int r1 = 0;

    int s = sgn(b);
    int n = abs(b);

    int i = 0;
    for (i = 0; i < n; i += 1) {
        r1 += a;
    }

    if (s == 0) {
        r1 = 0 - r1;
    }
    return 0;
}

int mod10(int n) {
    while (n >= 10) {
        n -= 10;
    }
    return n;
}

// ＝＝＝＝＝＝＝＝上信用
int div(int a, int b) {
    int q = 0;

    while (Gr(a, b) || (a == b)) {
        a -= b;
        q += 1;
    }

    return q;
}

void print_num(int n) {
    if (n >= 10) {
        print_num(div(n, 10));
    }
    putchar((int)'0' + mod10(n));
    return;
}

void main(void) {
    // if () {
    print_num(500);
    // putchar((int)'a');
    // }
    return;
}