
// int swap(int *x, int *y) {
//     int e = 0;
//     e = *y;
//     *y = *x;
//     *x = e;

//     return 0;
// }

// int add(int x, int y) { return x + y; }

int main(void) {
    int a = 100;
    // int b = 50;

    return *(&a);
}