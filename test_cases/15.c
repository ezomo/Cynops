/* 4.1 基本的なswitch文 */
int main(void) {
    int x = 2;
    switch (x) {
        case 1:
            x = 10;
            break;
        case 2:
            x = 20;
            break;
        default:
            x = 0;
            break;
    }
    return 0;
}
