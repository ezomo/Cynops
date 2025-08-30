int printf(const char*, ...);

int main(void) {
    char a[4] = {'%', 'd', '\n', '\0'};
    printf(&a[0], 1);
    printf(&("%d\n\0")[0], 1);
}