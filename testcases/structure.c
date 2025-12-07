void putchar(char);
void print_int_core(int x) {
    if (x < 0) {
        putchar('-');
        x = -x;
    }

    if (x >= 10) {
        print_int_core(x / 10);
    }
    // キャストを適当に作っているので明示的に
    putchar((char)((int)'0' + (x % 10)));
    return;
}
void printf(char (*s)[0]) {
    int i = 0;
    for (i = 0; (*s)[i] != '\0'; i += 1) {
        putchar((*s)[i]);
    }
    return;
}

struct st1 {
    int age;
    char family_name[6];
    char given_name[7];
    char separator;
};

void main(void) {
    struct st1 st1_me = {18, "Ezomo\0", "Daniel\0", '_'};

    // Basic
    {
        print_int_core(st1_me.age);
        putchar(st1_me.separator);
        printf((char (*)[0]) & st1_me.family_name);
        putchar(st1_me.separator);
        printf((char (*)[0]) & st1_me.given_name);
        putchar('\n');
    }

    // Copy
    {
        struct st1 st1_me2 = st1_me;

        print_int_core(st1_me2.age);
        putchar(st1_me2.separator);
        printf((char (*)[0]) & st1_me2.family_name);
        putchar(st1_me2.separator);
        printf((char (*)[0]) & st1_me2.given_name);
        putchar('\n');
    }

    // Pointer
    {
        struct st1* st1_me3 = &st1_me;

        print_int_core(st1_me3->age);
        putchar(st1_me3->separator);
        printf((char (*)[0]) & st1_me3->family_name);
        putchar(st1_me3->separator);
        printf((char (*)[0]) & st1_me3->given_name);
        putchar('\n');
    }

    // Write
    {
        st1_me.age = 81;
        st1_me.family_name = "omozE\0";
        st1_me.given_name = "leinaD\0";
        st1_me.separator = '|';

        print_int_core(st1_me.age);
        putchar(st1_me.separator);
        printf((char (*)[0]) & st1_me.family_name);
        putchar(st1_me.separator);
        printf((char (*)[0]) & st1_me.given_name);
        putchar('\n');
    }

    // Write using pointers.
    {
        struct st1* st1_me4 = &st1_me;
        st1_me4->age = 180;
        st1_me4->family_name = "EZOMO\0";
        st1_me4->given_name = "DANIEL\0";
        st1_me4->separator = '-';

        print_int_core(st1_me.age);
        putchar(st1_me.separator);
        printf((char (*)[0]) & st1_me.family_name);
        putchar(st1_me.separator);
        printf((char (*)[0]) & st1_me.given_name);
        putchar('\n');
    }

    // Partial reference
    {
        int* age = &st1_me.age;
        print_int_core(*age), putchar('\n');
        *age = 100;
        print_int_core(st1_me.age);
    }

    return;
}
