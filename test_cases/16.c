/* 4.2 fall-throughを含むswitch文 */
int main(void) {
    int x = 1;
    int result = 0;
    switch (x) {
        case 1:
            result = result + 1;
        case 2:
            result = result + 2;
            break;
        case 3:
            result = result + 3;
            break;
        default:
            result = -1;
    }
    return 0;
}
