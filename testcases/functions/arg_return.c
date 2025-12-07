
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
void print_int(int a) {
    print_int_core(a);
    putchar('\n');
    return;
}
void printf(char (*s)[0]) {
    int i = 0;
    for (i = 0; (*s)[i] != '\0'; i += 1) {
        putchar((*s)[i]);
    }
    putchar('\n');
    return;
}

void none(void) { return; }
int same(int a) { return a; }
int copy(int* a) { return *a }
char(ezomo(void))[6] { return "ezomo\0"; }

void swap(char* a, char* b) {
    char tmp = *a;
    *a = *b;
    *b = tmp;
    return;
}

// fn(char [6]) -> char [6]
char(rev_ezomo(char ez[6]))[6] {
    swap(&ez[0], &ez[4]);
    swap(&ez[1], &ez[3]);
    return ez;
}

void reset_ezomo(char (*ez)[6]) {
    *ez = "ezomo\0";
    return;
}

struct st {
    int age;
    char family_name[6];
    char given_name[7];
    char separator;
};

struct st init_st1(void) {
    struct st tmp = {0, "00000\0", "111111\0", '2'};
    return tmp;
}

void main(void) {
    {
        none();
        print_int(same(50));
    }

    {
        int a = 500;
        print_int(copy(&a));
    }

    {
        char ez[6] = ezomo();
        printf((char (*)[0]) & ez);

        ez = rev_ezomo(ez);
        printf((char (*)[0]) & ez);

        reset_ezomo(&ez);
        printf((char (*)[0]) & ez);
    }

    {
        struct st st1 = init_st1();
        printf((char (*)[0]) & st1.family_name);
        printf((char (*)[0]) & st1.given_name);
        putchar(st1.separator);
    }

    return;
}