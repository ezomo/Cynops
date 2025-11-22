void putchar(int a);
void exit(void);

void printf(char (*s)[0]) {
    int i = 0;
    for (i = 0; (*s)[i] != '\0'; i += 1) {
        putchar((int)(*s)[i]);
    }
}

void print_int(int x) {
    if (x < 0) {
        putchar((int)'-');
        x = -x;
    }

    if (x >= 10) {
        print_int(x / 10);
    }
    putchar((int)'0' + (x % 10));

    return;
}

// == == = kokokara/

int n(void) { return 8; }

void init_board(int (*board)[8][8]) {
    int y = 0;
    for (y = 0; y < n(); y++) {
        int x = 0;
        for (x = 0; x < n(); x++) (*board)[y][x] = (int)' ';
    }

    (*board)[3][3] = (int)'O';
    (*board)[3][4] = (int)'*';
    (*board)[4][3] = (int)'*';
    (*board)[4][4] = (int)'O';

    return;
}

void draw(int turn, int (*board)[8][8]) {
    char t1[] = {'\x1b', '[', '2', 'J', '\x1b', '[', 'H', '\0'};
    printf((char (*)[0]) & t1);  // clear + home

    char t2[] = {' ', ' ', ' ', 'a', ' ', 'b', ' ', 'c', ' ',  'd',
                 ' ', 'e', ' ', 'f', ' ', 'g', ' ', 'h', '\n', '\0'};
    printf((char (*)[0]) & t2);
    int y = 0;
    for (y = 0; y < n(); y++) {
        print_int(y + 1);
        putchar((int)' ');
        int x = 0;
        for (x = 0; x < n(); x++) {
            putchar((int)'|');
            putchar((*board)[4][4]);
        }
        putchar((int)' ');
        putchar((int)'\n');
    }
    char t3[] = {'\n', 'C', 'u', 'r', 'r', 'e', 'n', 't',
                 ' ',  't', 'u', 'r', 'n', ':', ' ', '\0'};
    printf((char (*)[0]) & t3);
    char t_black[] = {'*', '(', 'B', 'l', 'a', 'c', 'k', ')', '\0'};
    char t_white[] = {'O', '(', 'W', 'h', 'i', 't', 'e', ')', '\0'};

    if (turn == 1) {
        printf((char (*)[0]) & t_black);
    } else {
        printf((char (*)[0]) & t_white);
    }
    putchar((int)'\n');
    return;
}

// Check if move is legal
int can_put(int x, int y, char me, char op, int (*board)[8][8]) {
    int dx[8] = {1, 1, 0, -1, -1, -1, 0, 1};
    int dy[8] = {0, 1, 1, 1, 0, -1, -1, -1};
    if ((*board)[y][x] != (int)' ') {
        return 0;
    } else {
        // Check 8 directions
        int d;
        for (d = 0; d < 8; d++) {
            int cx = x + dx[d];
            int cy = y + dy[d];
            int found_op = 0;

            while (cx >= 0 && cx < n() && cy >= 0 && cy < n()) {
                if (board[cy][cx] == op) {
                    found_op = 1;
                } else if (board[cy][cx] == me) {
                    if (found_op)
                        return 1;
                    else
                        break;
                } else
                    break;

                cx += dx[d];
                cy += dy[d];
            }
        }
        return 0;
    }
}

int has_any_move(char me, char op, int (*board)[8][8]) {
    int y = 0;
    for (y = 0; y < n(); y++) {
        int x = 0;
        for (x = 0; x < n(); x++)
            if (can_put(x, y, me, op, board)) return 1;
    }
    return 0;
}

int main(void) {
    int board[8][8];

    init_board(&board);
    int turn = 1;  // 1 = Black (human), -1 = White (AI)

    {
        draw(turn, &board);
        char me = (turn == 1 ? '*' : 'O');
        char op = (turn == 1 ? 'O' : '*');
        // Check moves availability
        int my_has = has_any_move(me, op, &board);
        int op_has = has_any_move(op, me, &board);
    }

    return;
}