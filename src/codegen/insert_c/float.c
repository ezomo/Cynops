// 小数点以下は4桁とする；
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

// 掛け算

int IS_Zero(Double a) {
    return (a.decimal_part == a.integer_part) && (a.integer_part == 0);
}

// 整数 n の桁数を返す
int digits(int n) {
    if (n == 0) return 1;  // 0 は 1 桁

    int d = 0;
    while (n > 0) {
        d++;
        n /= 10;
    }
    return d;
}
// n の i 桁目 (右から数えて i=0 が1の位) を返す
int get_digit(int n, int i) {
    while (i--) n /= 10;
    return n % 10;
}

// a と b を "桁列として" 結合して、下位(右)から2桁ずつ切る
// ただし整数合成はしない
int split_join_2(int a, int b, int (*out)[0]) {
    int da = digits(a);
    int db = digits(b);

    int total = da + db;
    int groups = (total + 1) / 2;

    int out_i = 0;  // out[0] が最小桁グループ
    int g;
    for (g = 0; g < groups; g++) {
        int idx_low = 2 * g;
        int idx_high = 2 * g + 1;

        int low = 0, high = 0;

        // --- low digit ---
        if (idx_low < db)
            low = get_digit(b, idx_low);
        else
            low = get_digit(a, idx_low - db);

        // --- high digit ---
        if (idx_high < db)
            high = get_digit(b, idx_high);
        else if (idx_high < total)
            high = get_digit(a, idx_high - db);
        else
            high = 0;

        (*out)[out_i] = high * 10 + low;
        out_i += 1;
    }

    return groups;
}

void print_int_len(int (*array)[0], int n) {
    int i = 0;
    for (i = n - 1; i >= 0; i--) {
        print_int((*array)[i]), putchar(' ');
    }

    return;
}

int(mul_help(int a[5], int b))[5] {
    int result[5] =
    { a[0] * b,
      a[1] * b,
      a[2] * b,
      a[3] * b,
      a[4] * b,
    }

    return result;
}

int digits_without_trailing_zeros(int n) {
    if (n == 0) return 0;

    // 負数なら正数化
    if (n < 0) n = -n;

    // 末尾のゼロを除去
    while (n % 10 == 0) {
        n /= 10;
    }

    // 桁数を数える
    int count = 0;
    while (n > 0) {
        count++;
        n /= 10;
    }

    return count;
}

int pow_int(int a, int b) {
    if (a <= 0) {
        return 1;
    } else
        return b * pow_int(a - 1, b);
}

int take_digits(int (*arr)[0], int size, int off) {
    int buf[10];
    int n = 0;

    int i = 0;

    // 1. すべての要素を 1 桁ずつ後ろから取得（%10）
    int i;
    for (i = 4; i >= 0; i--) {
        int v = (*arr)[i];

        buf[n] = v / 10;
        buf[n + 1] = v % 10;
        v %= 10;
        n += 2;
    }

    // 2. 末尾の 0 を削除（buf の末尾 = 最初の桁列の末尾）
    n = 10;
    while (buf[n - 1] == 0) n--;
    if (n == 0) return 0;
    n -= off;

    int result = 0;
    for (i = 0; i < size; i++) {
        int o = n - size + i;
        if ((0 <= o) && (0 <= 9)) {
            result += pow_int(size - i - 1, 10) * buf[n - size + i];
        }
    }

    return result;
}

int min(int a, int b) { return (a < b) ? a : b; }

Double DoubleMul(Double a, Double b) {
    if (IS_Zero(a) || IS_Zero(b)) {
        return InitDouble(0, 0);
    }

    int a_[5];  // 99 99
    int b_[5];

    {
        split_join_2(a.integer_part, a.decimal_part, (int (*)[0]) & a_);
        split_join_2(b.integer_part, b.decimal_part, (int (*)[0]) & b_);
    }

    // 2+5
    int result[5][5];
    {  // 多次元配列の関数を用いた初期化がうまくできない
        result[0] = mul_help(a_, b_[0]);
        result[1] = mul_help(a_, b_[1]);
        result[2] = mul_help(a_, b_[2]);
        result[3] = mul_help(a_, b_[3]);
        result[4] = mul_help(a_, b_[4]);
    }

    int result_sum[5 + 5 - 1];
    {
        int y;
        for (y = 0; y < 4; y++) {
            int x;
            for (x = 0; x < 4; x++) {
                result_sum[x + y] += result[y][x];
            }
        }
    }

    {  // 桁上げ処理
        int x;
        for (x = 0; x < 5 + 5 - 1 - 1; x++) {
            int up = result_sum[x] / 100;
            result_sum[x] = result_sum[x] % 100;
            result_sum[x + 1] += up;
        }
    }

    int d_part_size = digits_without_trailing_zeros(a.decimal_part) +
                      digits_without_trailing_zeros(b.decimal_part);

    Double re_turn = InitDouble(0, 0);

    int off = (d_part_size > 4) ? (d_part_size - 4) : 0;
    int size = min(d_part_size, 4);
    re_turn.decimal_part = take_digits((int (*)[0]) & result_sum, size, off);
    re_turn.integer_part =
        take_digits((int (*)[0]) & result_sum, 4, size + off);
    re_turn.sgn = a.sgn * b.sgn;

    return re_turn;
}