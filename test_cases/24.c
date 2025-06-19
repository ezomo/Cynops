/* 6.6 ポインタ演算子 */
int main(void) {
    int x = 42;
    int* ptr;
    ptr = &x;
    x = *ptr;
    return 0;
}
