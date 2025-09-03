void printf(char*, ...);

int main(void) {
    int a[][2 + 2] = {{1, 2}, {3, 4}};
    char c[] = "%d\n\0";
    a[1][0]++;
    printf(&c[0], a[1][0]);
}
