/* 5.1 break/continue */
int main(void) {
    int i;
    for (i = 0; i < 20; i = i + 1) {
        if (i == 5) continue;
        if (i == 15) break;
    }
    return 0;
}
