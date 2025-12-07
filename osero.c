void putchar(char);
char getchar(void);
void exit(void);
void printf(char (*s)[0]) {
    int i = 0;
    for (i = 0; (*s)[i] != '\0'; i += 1) {
        putchar((*s)[i]);
    }
}
void print_int(int x) {
    if (x < 0) {
        putchar('-');
        x = -x;
    }

    if (x >= 10) {
        print_int(x / 10);
    }
    putchar('0' + (char)(x % 10));

    return;
}

// ==start
typedef struct {
    char grid[8][8];
    int n;
} Board;

typedef struct {
    char me;
    char op;
} MeOp;

MeOp meop_swap(MeOp* meop) {
    MeOp tmp;
    tmp.op = meop->me;
    tmp.me = meop->op;
    return tmp;
}

Board init_Board(void) {
    Board board;
    board.n = 8;
    int y = 0;
    for (y = 0; y < board.n; y++) {
        int x = 0;
        for (x = 0; x < board.n; x++) {
            board.grid[y][x] = ' ';
        }
    }
    board.grid[3][3] = 'O';
    board.grid[3][4] = '*';
    board.grid[4][3] = '*';
    board.grid[4][4] = 'O';

    return board;
}

// Draw board
void draw(int turn, Board* board) {
    char t1[] = "\x1b[2J\x1b[H\0";
    printf((char (*)[0]) & t1);

    char t2[] = "  a b c d e f g h\n\0";
    printf((char (*)[0]) & t2);

    int y = 0;
    for (y = 0; y < board->n; y++) {
        print_int(y + 1);
        int x = 0;
        for (x = 0; x < board->n; x++) {
            putchar('|'), putchar(board->grid[y][x]);
        }
        putchar('|'), putchar('\n');
    }

    char t3[] = "\nCurrent turn: \0";
    printf((char (*)[0]) & t3);
    if (turn == 1) {
        char t4[] = "*(Black)\n\0";
        printf((char (*)[0]) & t4);

    } else {
        char t4[] = "O(White)\n\0";
        printf((char (*)[0]) & t4);
    }
}

// Check if move is legal
int can_put(int x, int y, MeOp meop, Board* board) {
    // Directions
    int dx[8] = {1, 1, 0, -1, -1, -1, 0, 1};
    int dy[8] = {0, 1, 1, 1, 0, -1, -1, -1};

    if (board->grid[y][x] != ' ') return 0;

    // Check 8 directions
    int d = 0;
    for (d = 0; d < 8; d++) {
        int cx = x + dx[d];
        int cy = y + dy[d];
        int found_op = 0;

        while (cx >= 0 && cx < board->n && cy >= 0 && cy < board->n) {
            if (board->grid[cy][cx] == meop.op) {
                found_op = 1;
            } else if (board->grid[cy][cx] == meop.me) {
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

// Count how many pieces would be flipped by placing at (x,y)
// (does not modify the board)
int count_flips(int x, int y, MeOp meop, Board* board) {
    // Directions
    int dx[8] = {1, 1, 0, -1, -1, -1, 0, 1};
    int dy[8] = {0, 1, 1, 1, 0, -1, -1, -1};

    if (board->grid[y][x] != ' ') return 0;

    int total = 0;
    int d = 0;
    for (d = 0; d < 8; d++) {
        int cx = x + dx[d];
        int cy = y + dy[d];
        int count = 0;

        while (cx >= 0 && cx < board->n && cy >= 0 && cy < board->n) {
            if (board->grid[cy][cx] == meop.op) {
                count++;
            } else if (board->grid[cy][cx] == meop.me) {
                if (count > 0) {
                    total += count;
                }
                break;
            } else
                break;

            cx += dx[d];
            cy += dy[d];
        }
    }

    return total;
}

// Place disk
void put_disk(int x, int y, MeOp meop, Board* board) {
    int dx[8] = {1, 1, 0, -1, -1, -1, 0, 1};
    int dy[8] = {0, 1, 1, 1, 0, -1, -1, -1};
    board->grid[y][x] = meop.me;
    int d = 0;
    for (d = 0; d < 8; d++) {
        int cx = x + dx[d];
        int cy = y + dy[d];
        int count = 0;

        while (cx >= 0 && cx < board->n && cy >= 0 && cy < board->n) {
            if (board->grid[cy][cx] == meop.op) {
                count++;
            } else if (board->grid[cy][cx] == meop.me) {
                // Flip
                int i = 0;
                for (i = 0; i < count; i++) {
                    board->grid[y + dy[d] * (i + 1)][x + dx[d] * (i + 1)] =
                        meop.me;
                }
                break;
            } else
                break;

            cx += dx[d];
            cy += dy[d];
        }
    }
}

// Check if player has any legal move
int has_any_move(MeOp meop, Board* board) {
    int y = 0;
    for (y = 0; y < board->n; y++) {
        int x = 0;
        for (x = 0; x < board->n; x++) {
            if (can_put(x, y, meop, board)) {
                return 1;
            }
        }
    }
    return 0;
}

// Simple AI: choose the legal move that flips the maximum number of disks.
// Returns 1 if a move was chosen/applied, 0 if no move available.
int ai_move(MeOp meop, Board* board) {
    int best_x = -1, best_y = -1;
    int best_score = -1;

    int y = 0;
    for (y = 0; y < board->n; y++) {
        int x = 0;
        for (x = 0; x < board->n; x++) {
            if (!can_put(x, y, meop, board)) continue;
            int flips = count_flips(x, y, meop, board);
            if (flips > best_score) {
                best_score = flips;
                best_x = x;
                best_y = y;
            }
        }
    }

    if (best_score <= 0) return 0;

    // Apply move
    put_disk(best_x, best_y, meop, board);
    // Print chosen move (convert to coordinates)
    char col = 'a' + (char)best_x;
    char row = '1' + (char)best_y;

    char t1[] = "AI plays: \0";
    printf((char (*)[0]) & t1);
    putchar(col), putchar(row);

    char t2[] = "(flips \0";
    printf((char (*)[0]) & t2);
    print_int(best_score);

    char t3[] = ")\n \0";
    printf((char (*)[0]) & t3);

    return 1;
}

// Count final score
void count_score(int* black, int* white, Board board) {
    *black = *white = 0;
    int y = 0;
    for (y = 0; y < board.n; y++) {
        int x = 0;
        for (x = 0; x < board.n; x++) {
            if (board.grid[y][x] == '*')
                (*black)++;
            else if (board.grid[y][x] == 'O')
                (*white)++;
        }
    }
}

void main(void) {
    Board board = init_Board();
    MeOp meop;
    int turn = 1;

    while (1) {
        draw(turn, &board);

        if (turn == 1) {
            meop.me = '*';
            meop.op = 'O';
        } else {
            meop.me = 'O';
            meop.op = '*';
        }

        int my_has = has_any_move(meop, &board);
        int op_has = has_any_move(meop_swap(&meop), &board);

        if (!my_has && !op_has) {
            // Game over
            int black, white;
            count_score(&black, &white, board);

            char t1[] = "Game over.\n\0";
            printf((char (*)[0]) & t1);

            char t2[] = "Black (*) : \0";
            printf((char (*)[0]) & t2);
            print_int(black), putchar('\n');

            char t3[] = "White (O) : \0";
            printf((char (*)[0]) & t3);
            print_int(white), putchar('\n');

            if (black > white) {
                char t4[] = "Black wins!\n\0";
                printf((char (*)[0]) & t4);
            } else if (white > black) {
                char t5[] = "White wins!\n\0";
                printf((char (*)[0]) & t5);
            } else {
                char t6[] = "It's a tie.\n\0";
                printf((char (*)[0]) & t6);
            }
            break;
        }

        if (!my_has) {
            // Current player must pass

            if (turn == 1) {
                char t1[] = "Black (*)\0";
                printf((char (*)[0]) & t1);

            } else {
                char t1[] = "White (O)\0";
                printf((char (*)[0]) & t1);
            }
            char t2[] = "has no legal moves and must pass.\n\0";
            printf((char (*)[0]) & t2);

            turn *= -1;

            char t3[] = "Press Enter to continue...\0";
            printf((char (*)[0]) & t3);
            getchar();
            continue;
        }

        if (turn == 1) {
            // Human move (Black)

            char t1[] = "Enter move (e.g., e3): \0";
            printf((char (*)[0]) & t1);

            char c1 = getchar();  // a–h
            char c2 = getchar();  // 1–8
            getchar();            // absorb newline

            int x = (int)c1 - (int)'a';
            int y = (int)c2 - (int)'1';

            if (x < 0 || x >= 8 || y < 0 || y >= 8) {
                char t2[] = "Invalid coordinate.\n\0";
                printf((char (*)[0]) & t2);
                getchar();
                continue;
            }

            if (!can_put(x, y, meop, &board)) {
                char t3[] = "You cannot place there.\n\0";
                printf((char (*)[0]) & t3);
                getchar();
                continue;
            }

            put_disk(x, y, meop, &board);
        } else {
            // AI's turn (White)
            // small pause/notification

            char t1[] = "AI thinking...\n\0";
            printf((char (*)[0]) & t1);

            // choose and apply move
            if (!ai_move(meop, &board)) {
                // Shouldn't happen because we checked my_has above, but handle
                // defensively
                char t2[] = "AI has no legal move.\n\0";
                printf((char (*)[0]) & t2);
            }
            // wait a moment so user can see AI move message
            char t3[] = "Press Enter to continue...\0";
            printf((char (*)[0]) & t3);
            getchar();
        }
        turn *= -1;  // Switch turn
    }

    return;
}
