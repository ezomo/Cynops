/* 5.2 goto文とラベル */
int main(void) {
    int x = 0;
    if (x == 0) goto end;
    x = 10;
end:
    return x;
}
