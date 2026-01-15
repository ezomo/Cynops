void putchar(char);
char getchar(void);

void print_int(int x) {
    if (x < 0) {
        putchar('-');
        x = -x;
    }

    if (x >= 10) {
        print_int(x / 10);
    }
    // キャストを適当に作っているので明示的に
    putchar((char)((int)'0' + (x % 10)));
    return;
}

int get_int(void) {
    int value = 0;
    char c;
    while ((c = getchar()) != '\0') {
        if ('0' <= c && c <= '9') {
            value = value * 10 + (int)(c - '0');
        } else {
            break;  // 数字以外が来たら終了
        }
    }
    return value;
}

void printf(char (*s)[0]) {
    int i = 0;
    for (i = 0; (*s)[i] != '\0'; i += 1) {
        putchar((*s)[i]);
    }
}

typedef struct {
    int red;
    int green;
    int blue;
} Colour;
Colour colour_new(int red, int green, int blue) {
    Colour tmp = {red, green, blue};
    return tmp;
}
typedef struct {
    Colour fg;
    Colour bg;
} Char;

typedef struct {
    int wide;
    int height;
    Char map[500][500];
} Canvas;

typedef struct {
    int x;
    int y;
} Coordinates;
Coordinates coordinates_new(int x, int y) {
    Coordinates tmp = {x, y};
    return tmp;
}

void move(int x, int y) {
    putchar('\x1b'), putchar('[');
    print_int(y + 1), putchar(';');
    print_int(x + 1), putchar('H');
    return;
}

void print_colour(Colour* colour) {
    print_int(colour->red), putchar(';');
    print_int(colour->green), putchar(';');
    print_int(colour->blue), putchar('m');
    return;
}

void print_fg(Colour colour) {
    char tmp[] = "\x1b[38;2;\0";
    printf((char (*)[0]) & tmp);
    print_colour(&colour);
    return;
}
void print_bg(Colour colour) {
    char tmp[] = "\x1b[48;2;\0";
    printf((char (*)[0]) & tmp);
    print_colour(&colour);
    return;
}

void print_block(void) {
    char tmp[] = "\xE2\x96\x80\x1b[0m\0";
    printf((char (*)[0]) & tmp);
    return;
}

void clear_screen(void) {
    char tmp[] = "\x1b[2J\x1b[1;1H\0";
    printf((char (*)[0]) & tmp);
    return;
}

void setpixel(Coordinates xy, Colour colour, Canvas* canvas) {
    int x = xy.x;
    int y = xy.y / 2;
    Char* index = &canvas->map[x][y];

    if (xy.y % 2 == 0) {
        index->fg = colour;
    } else {
        index->bg = colour;
    }

    move(xy.x, xy.y / 2);
    print_fg(index->fg);
    print_bg(index->bg);
    print_block();

    return;
}

void canvas_init(Canvas* canvas, int wide, int height, Colour base) {
    canvas->wide = wide;
    canvas->height = height / 2;

    clear_screen();
    // int x = 0;
    // int y = 0;
    // for (x = 0; x < canvas->wide; x++) {
    //     for (y = 0; y < height; y++) {
    //         if (height % 2 == 0) {
    //             Char* index = &canvas->map[wide][height];
    //             index->fg = base;
    //             index->bg = base;
    //         }
    //         Coordinates xy = {x, y};
    //         setpixel(xy, base, canvas);
    //     }
    // }

    return;
}

void main(void) {
    Canvas canvas;

    {
        // 初期化
        canvas_init(&canvas, get_int(), get_int(),
                    colour_new(get_int(), get_int(), get_int()));

        move(0, canvas.height);
    }

    Coordinates coordinates = coordinates_new(0, 0);
    Colour colour = colour_new(0, 0, 0);
    while (getchar() != '\0') {
        {  // 受け取り
            coordinates.x = get_int();
            coordinates.y = get_int();
            colour.red = get_int();
            colour.green = get_int();
            colour.blue = get_int();
        }

        {  // 書き込み
            setpixel(coordinates, colour, &canvas);
        }
    }
    move(canvas.wide, canvas.height);

    return;
}

// cat ./sample/british.text | {./run.sh ./sample/ansi_canvas.c }