int x;
int x = 5;
int x, y, z;
int x = 1, y = 2, z = 3;
int x[10];
int x[10] = {1, 2, 3, 4};
int *x;
int *x = 0;
int *x, *y, *z;
int *x = &y, y = 5;

char c;
char c = 'a';
char s[] = {'h', 'e', 'l', 'l', 'o'};
// char *s = "hello";

void (*f)();          // 関数ポインタ
int (*cmp)(int, int); // 関数ポインタ with param
int *(*f)(int);       // 関数ポインタ returning pointer
int (*a[5])(int);     // 関数ポインタの配列

int matrix[3][3];     // 多次元配列
int (*matrix_ptr)[3]; // ポインタ to 配列

int (*complex(
    int x))[5]; // 関数 complex(int) returning pointer to array of 5 int

int (*judge)(int) = 0; // 初期化ありの関数ポインタ