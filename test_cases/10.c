/* 3.3 ネストしたif文 */
int main(void) {
    int x = 10;
    int y = 5;
    if (x > 0) {
        if (y > 0) {
            x = x + y;
        } else {
            x = x - y;
        }
    }
    return 0;
}
