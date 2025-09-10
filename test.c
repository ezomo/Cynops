void printf(char*, ...);
int (*malloc(int))[0];

enum week { Mon, Tue = 5, Wed, Thu, Fri, Sat, Sun };

void main(void) {
    enum week wk0;
    wk0 = Sun;

    // a.a = 0;
    printf(&"%d\n\0"[0], Mon);
}