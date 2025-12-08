void print_double(double);
void putchar(char);
void put_double(double a) {
    print_double(a);
    putchar('\n');
    return;
}

int main(void) {
    put_double(1.5);  // test

    // Add
    {
        put_double(1.5 + 1.0);
        put_double(1.5 + 1.2);
        put_double(1.5 + 1.9);
        put_double(0.0 + 1.9);
        put_double(0.0 + 0.0);

        put_double(1.5 + (-1.0));
        put_double(1.5 + (-1.2));
        put_double(1.5 + (-1.9));
        put_double(0.0 + (-1.9));
        put_double(0.0 + (-0.0));

        put_double((-1.5) + (-1.0));
        put_double((-1.5) + (-1.2));
        put_double((-1.5) + (-1.9));
        put_double((-0.0) + (-1.9));
        put_double((-0.0) + (-0.0));

        put_double((-1.5) + 1.0);
        put_double((-1.5) + 1.2);
        put_double((-1.5) + 1.9);
        put_double((-0.0) + 1.9);
        put_double((-0.0) + 0.0);
    }

    // Sub
    {
        put_double(1.5 - 1.0);
        put_double(1.5 - 1.2);
        put_double(1.5 - 1.9);
        put_double(0.0 - 1.9);
        put_double(0.0 - 0.0);

        put_double(1.5 - (-1.0));
        put_double(1.5 - (-1.2));
        put_double(1.5 - (-1.9));
        put_double(0.0 - (-1.9));
        put_double(0.0 - (-0.0));

        put_double((-1.5) - (-1.0));
        put_double((-1.5) - (-1.2));
        put_double((-1.5) - (-1.9));
        put_double((-0.0) - (-1.9));
        put_double((-0.0) - (-0.0));

        put_double((-1.5) - 1.0);
        put_double((-1.5) - 1.2);
        put_double((-1.5) - 1.9);
        put_double((-0.0) - 1.9);
        put_double((-0.0) - 0.0);
    }

    return;
}