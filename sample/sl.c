void putchar(char);
char getchar(void);

void print_int(int x)
{
    if (x < 0)
    {
        putchar('-');
        x = -x;
    }

    if (x >= 10)
    {
        print_int(x / 10);
    }
    // キャストを適当に作っているので明示的に
    putchar((char)((int)'0' + (x % 10)));
    return;
}

void printf(char (*s)[0])
{
    int i = 0;
    for (i = 0; (*s)[i] != '\0'; i += 1)
    {
        putchar((*s)[i]);
    }
    return;
}

void move(int x, int y)
{
    putchar('\x1b'), putchar('[');
    print_int(y + 1), putchar(';');
    print_int(x + 1), putchar('H');
    return;
}

void hide_cursor(void)
{
    putchar('\x1b');
    putchar('[');
    putchar('?');
    putchar('2');
    putchar('5');
    putchar('l');
    return;
}

void show_cursor(void)
{
    putchar('\x1b');
    putchar('[');
    putchar('?');
    putchar('2');
    putchar('5');
    putchar('h');
    return;
}

void clear_screen(void)
{
    char tmp[] = "\x1b[2J\x1b[1;1H\0";
    printf((char (*)[0]) & tmp);
    return;
}

int strlen(char (*s)[0])
{
    int i = 0;
    while ((*s)[i] != '\0')
        i++;
    return i;
}

/* x,y座標を指定して文字列を出力。x<0の場合は先頭をskipして0列から描画 */
void draw_str(int x, int y, char (*s)[0])
{
    if (y < 0)
        return;
    int len = strlen(s);
    if (x >= 0)
    {
        move(x, y);
        printf((char (*)[0])s);
    }
    else
    {
        int skip = -x;
        if (skip < len)
        {
            move(0, y);
            printf((char (*)[0]) & (*s)[skip]);
        }
    }
}

void printSL(void)
{
    int COLS = 100;
    int lineNum = 100 / 4;

    char smoke1[5][83];
    smoke1[0] = "                        (  ) (@@) ( )  (@)  ()    @@    O     @     O     @      O\0",
    smoke1[1] = "                   (@@@)\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    smoke1[2] = "               (    )\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    smoke1[3] = "            (@@@@)\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    smoke1[4] = "          (   )\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

    char smoke2[5][83];
    smoke2[0] = "                        (@@) (  ) ( )  ()  (@)    @@    @     O     @     O      @\0",
    smoke2[1] = "                   (   )\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    smoke2[2] = "               (@@@@)\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    smoke2[3] = "            (    )\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    smoke2[4] = "          (@@@)\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";

    char body[7][83];
    body[0] = "      ====        ________                ___________\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    body[1] = "  _D _|  |_______/        \\__I_I_____===__|_________|\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0";
    body[2] = "   |(_)---  |   H\\________/ |   |        =|___ ___|      _________________\0\0\0\0\0\0\0\0\0";
    body[3] = "   /     |  |   H  |  |     |   |         ||_| |_||     _|                \\_____A\0\0";
    body[4] = "  |      |  |   H  |__--------------------| [___] |   =|                        |\0\0";
    body[5] = "  | ________|___H__/__|_____/[][]~\\_______|       |   -|                        |\0\0";
    body[6] = "  |/ |   |-----------i_____I [][] []  D   |=======|____|________________________|_\0";

    char wh[7][83];
    /* 0 */
    wh[0] = "__/ =| o |=-O=====O=====O=====O\\  ____Y___________|__|__________________________|_\0";
    wh[1] = " |/-=|___|=    ||    ||    ||    |_____/\\___/          |_D__D__D_|  |_D__D__D_|\0  \0";
    wh[2] = "  \\_/      \\__/  \\__/  \\__/  \\__/      \\_/               \\_/   \\_/    \\_/   \\_/\0  \0";
    wh[3] = "__/ =| o |=-~~\\  /~~\\  /~~\\  /~~\\ ____Y___________|__|__________________________|_\0";
    wh[4] = " |/-=|___|=O=====O=====O=====O   |_____/\\___/          |_D__D__D_|  |_D__D__D_|\0  \0";
    wh[5] = "  \\_/      \\_O=====O=====O=====O/      \\_/               \\_/   \\_/    \\_/   \\_/\0  \0";
    wh[6] = " |/-=|___|=   O=====O=====O=====O|_____/\\___/          |_D__D__D_|  |_D__D__D_|\0  \0";

    int maxLen = strlen((char (*)[0]) & wh[2]);
    int total = COLS + maxLen + 6;
    int count = 1;

    hide_cursor();

    int pos = 10;
    for (pos = 0; pos < total; pos += 4)
    {
        clear_screen();

        int x = COLS - pos;

        // 煙の切り替え
        char (*smoke)[5][83] = (pos % 3 == 0) ? (&smoke1) : (&smoke2);
        int lineNum = 0;

        // 足
        int frame = count % 4;
        if (frame == 1)
        {
            draw_str(x, lineNum + 13, (char (*)[0]) & wh[0]);
            draw_str(x, lineNum + 14, (char (*)[0]) & wh[1]);
            draw_str(x, lineNum + 15, (char (*)[0]) & wh[2]);
        }
        else if (frame == 2)
        {
            draw_str(x, lineNum + 13, (char (*)[0]) & wh[3]);
            draw_str(x, lineNum + 14, (char (*)[0]) & wh[4]);
            draw_str(x, lineNum + 15, (char (*)[0]) & wh[2]);
        }
        else if (frame == 3)
        {
            draw_str(x, lineNum + 13, (char (*)[0]) & wh[3]);
            draw_str(x, lineNum + 14, (char (*)[0]) & wh[1]);
            draw_str(x, lineNum + 15, (char (*)[0]) & wh[5]);
        }
        else
        {
            draw_str(x, lineNum + 13, (char (*)[0]) & wh[3]);
            draw_str(x, lineNum + 14, (char (*)[0]) & wh[6]);
            draw_str(x, lineNum + 15, (char (*)[0]) & wh[2]);
        }
        // /* 車体 */
        draw_str(x, lineNum + 6, (char (*)[0]) & body[0]);
        draw_str(x, lineNum + 7, (char (*)[0]) & body[1]);

        draw_str(x, lineNum + 11, (char (*)[0]) & body[6]);
        draw_str(x, lineNum + 12, (char (*)[0]) & body[0]);

        draw_str(x, lineNum + 9, (char (*)[0]) & body[3]);

        draw_str(x, lineNum + 8, (char (*)[0]) & body[2]);
        draw_str(x, lineNum + 10, (char (*)[0]) & body[4]);

        // 煙
        draw_str(x, lineNum, (char (*)[0]) & (*smoke)[0]);
        draw_str(x, lineNum + 1, (char (*)[0]) & (*smoke)[1]);

        draw_str(x, lineNum + 2, (char (*)[0]) & (*smoke)[2]);
        draw_str(x, lineNum + 3, (char (*)[0]) & (*smoke)[3]);
        /* lineNum+4 は空行 */
        draw_str(x, lineNum + 5, (char (*)[0]) & (*smoke)[4]);

        if (count / 8 == 1)
            count = 1;
        else
            count++;

        int i = 0;
        for (i = 0; i < 100 * 2; i++)
        {
            int j = 0;
            for (j = 0; i < 100; i++)
            {
                /* code */
            }
        }
    }

    show_cursor();
    return;
}

void main(void)
{
    printSL();
    return;
}

// ./run.sh ./sample/sl.c
