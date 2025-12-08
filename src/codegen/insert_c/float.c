void putchar(char);
void print_int(int);
typedef struct {
    int sgn;  // 1 or -1
    int integer_part;
    int decimal_part;
} Double;

int make3digits(int a) {
    if (a == 0) return 0;

    int i = 0;
    for (i = 0; a < 999; i++) {
        a *= 10;
    }

    return a;
}

void print_double(Double a) {
    if (a.sgn == -1) {
        putchar('-');
    }
    print_int(a.integer_part);
    putchar('.');
    print_int(a.decimal_part);
    return;
}

// 外部からの呼び出し専用;
Double InitDouble(int num, int dp) {
    Double tmp = {1, num, make3digits(dp)};
    return tmp;
}

// Unary
// a * (-1)
Double DoubleMinus(Double a) {
    a.sgn *= -1;

    return a;
}

// Binary
int DoubleGreater(Double a, Double b) {
    // 符号が異なる場合
    if (a.sgn > b.sgn) return 1;  // a > 0, b < 0
    if (a.sgn < b.sgn) return 0;  // a < 0, b > 0

    // 同符号の場合
    int factor = a.sgn;  // 符号が -1 なら逆転
    if (a.integer_part != b.integer_part)
        return (a.integer_part - b.integer_part) * factor > 0;

    // 整数部が同じなら小数部を比較
    return (a.decimal_part - b.decimal_part) * factor > 0;
}

int DoubleLess(Double a, Double b) { return DoubleGreater(b, a); }

int DoubleEqual(Double a, Double b) {
    return (a.sgn == b.sgn) && (a.integer_part == b.integer_part) &&
           (a.decimal_part == b.decimal_part);
}

// 加減算

// a >= b>0
Double DoubleSubAGeBPos(Double a, Double b);
// a>0,b>0
Double DoubleAddBothPositive(Double a, Double b);
Double DoubleSub(Double a, Double b);
Double DoubleAdd(Double a, Double b);

// a>0,b>0
Double DoubleAddBothPositive(Double a, Double b) {
    int i = a.integer_part + b.integer_part;
    int d = a.decimal_part + b.decimal_part;
    // 桁あげ
    if (d >= 10000) {
        d -= 10000;
        i += 1;
    }
    Double tmp = {1, i, d};
    return tmp;
}

Double DoubleAdd(Double a, Double b) {
    if (a.sgn == 1 && b.sgn == 1) {
        // a+b;
        return DoubleAddBothPositive(a, b);
    }

    if (a.sgn == 1 && b.sgn == -1) {
        // a +(-b) => a-b
        b.sgn = 1;
        return DoubleSub(a, b);
    }

    if (a.sgn == -1 && b.sgn == 1) {
        // -a + b => b-a
        a.sgn = 1;
        return DoubleSub(b, a);
    }

    if (a.sgn == -1 && b.sgn == -1) {
        // -a +(-b) => -(a + b)
        a.sgn = b.sgn = 1;
        Double tmp = DoubleAddBothPositive(a, b);
        tmp.sgn = -1;
        return tmp;
    }
}

// a >= b>0
Double DoubleSubAGeBPos(Double a, Double b) {
    int i = a.integer_part - b.integer_part;
    int d = 0;
    if (a.decimal_part < b.decimal_part) {
        d = 10000 + a.decimal_part - b.decimal_part;
        i -= 1;
    } else {
        d = a.decimal_part - b.decimal_part;
    }
    Double tmp = {1, i, d};
    return tmp;
}

Double DoubleSub(Double a, Double b) {
    if (a.sgn == 1 && b.sgn == 1) {
        // a - b
        if (DoubleGreater(a, b)) {
            // a > b
            return DoubleSubAGeBPos(a, b)
        } else {
            // a<=b => -(b-a)
            Double tmp = DoubleSubAGeBPos(b, a);
            tmp.sgn = -1;
            return tmp;
        }
    }

    if (a.sgn == 1 && b.sgn == -1) {
        // a -(-b) => a+b
        b.sgn = 1;
        return DoubleAddBothPositive(a, b);
    }

    if (a.sgn == -1 && b.sgn == 1) {
        // -a - b => -(a+b)
        a.sgn = 1;
        Double tmp = DoubleAddBothPositive(b, a);
        tmp.sgn = -1;
        return tmp;
    }

    if (a.sgn == -1 && b.sgn == -1) {
        // -a -(-b) => -a + b => b-a;
        a.sgn = b.sgn = 1;
        return DoubleSub(b, a);
    }
}