int printf(char*, ...);
char* malloc(int);
void free(char*);
double sin(double);
double cos(double);
double sqrt(double);

typedef struct {
    int red;
    int green;
    int blue;
} Colour;

Colour colour_new(int red, int green, int blue) {
    Colour tmp = {red, green, blue};
    return tmp;
}

Colour colour_init(void) { return colour_new(0, 0, 0); }

typedef struct {
    Colour fg;
    Colour bg;
} Char;

Char char_init(void) {
    Char tmp = {colour_new(0, 0, 0), colour_new(0, 0, 0)};
    return tmp;
}

typedef struct {
    int wide;
    int height;
    Char (*vec)[0];
} Canvas;

typedef struct {
    int x;
    int y;
} Coordinates;

Coordinates coordinates_new(int x, int y) {
    Coordinates tmp = {x, y};
    return tmp;
}

Coordinates coordinates_init(void) { return coordinates_new(0, 0); }

typedef struct {
    double vx;
    double vy;
    double vz;
} Vec3;

Vec3 vec3_new(double vx, double vy, double vz) {
    Vec3 tmp = {vx, vy, vz};
    return tmp;
}

Vec3 vec3_init(void) { return vec3_new(0.0, 0.0, 0.0); }

typedef struct {
    Vec3 position;
    Vec3 rotation;
    Vec3 scale;
} Transform;

Transform transform_new(Vec3 position, Vec3 rotation, Vec3 scale) {
    Transform tmp = {position, rotation, scale};
    return tmp;
}

Transform transform_init(void) {
    return transform_new(vec3_init(), vec3_init(), vec3_new(1.0, 1.0, 1.0));
}

int id(int x, int y, int w) { return y * w + x; }

void move(int x, int y) { printf(&"\x1b[%d;%dH\0"[0], y + 1, x + 1); }
void print_fg(Colour colour) {
    printf(&"\x1b[38;2;%d;%d;%dm\0"[0], colour.red, colour.green, colour.blue);
}
void print_bg(Colour colour) {
    printf(&"\x1b[48;2;%d;%d;%dm\0"[0], colour.red, colour.green, colour.blue);
}
void print_block(void) { printf(&"\xE2\x96\x80\x1b[0m\0"[0]); }

Char* canvas_index(int n, Canvas* canvas) { return &(*(canvas->vec))[n]; }

void setpixel(Coordinates xy, Colour colour, Canvas* canvas) {
    int x = xy.x;
    int y = xy.y / 2;
    int index_pos = id(x, y, canvas->wide);

    if (xy.y % 2 == 0) {
        canvas_index(index_pos, canvas)->fg = colour;
    } else {
        canvas_index(index_pos, canvas)->bg = colour;
    }

    move(xy.x, xy.y / 2);
    print_fg(canvas_index(index_pos, canvas)->fg);
    print_bg(canvas_index(index_pos, canvas)->bg);
    print_block();

    move(0, canvas->height + 1);
}

void clear_screen(void) { printf(&"\x1b[2J\x1b[1;1H\0"[0]); }

Canvas* canvas_init(int wide, int height, Colour base) {
    Canvas* canvas = (Canvas*)malloc(sizeof(Canvas));
    canvas->wide = wide;
    canvas->height = height / 2;
    canvas->vec =
        (Char(*)[0])malloc(sizeof(Char) * canvas->height * canvas->wide);

    clear_screen();
    int x = 0;
    for (x = 0; x < canvas->wide; x++) {
        int y = 0;
        for (; y < canvas->height; y++) {
            int index_pos = id(x, y / 2, canvas->wide);
            canvas_index(index_pos, canvas)->fg = base;
            canvas_index(index_pos, canvas)->bg = base;
        }
    }

    for (x = 0; x < wide; x++) {
        int y = 0;
        for (; y < height; y++) {
            Coordinates xy = {x, y};
            setpixel(xy, base, canvas);
        }
    }

    return canvas;
}

typedef struct {
    Coordinates xy;
    Colour colour;
} Block;

Block block_new(Coordinates xy, Colour colour) {
    Block tmp = {xy, colour};
    return tmp;
}

Block block_init(void) { return block_new(coordinates_init(), colour_init()); }

typedef struct {
    Block block;
} Data;

Data data_new(Block block) {
    Data tmp = {block};
    return tmp;
}

Data data_init(void) { return data_new(block_init()); }

typedef struct Vector {
    int is_end;
    Data data;
    struct Vector* next;
} Vector;

Vector* vec_init(void) {
    Vector* vec = (Vector*)malloc(sizeof(Vector));
    vec->is_end = 1;
    vec->data = data_init();
    return vec;
}

Vector* end(Vector* vec) {
    Vector* current = vec;
    while (current->is_end == 0) {
        current = current->next;
    }
    return current;
}

int len(Vector* vec) {
    Vector* current = vec;
    int i = 0;
    while (current->is_end == 0) {
        current = current->next;
        i++;
    }
    return i + 1;
}

Vector* append(Vector* vec) {
    Vector* last = end(vec);
    last->is_end = 0;
    last->next = (Vector*)malloc(sizeof(Vector));
    last->next->is_end = 1;
    last->next->data = data_init();
    return last->next;
}

Vector* vec_index(Vector* vec, int index) {
    int i = 0;
    Vector* current = vec;
    for (i = 0; i < index; i++) {
        if (current->is_end == 1) {
            printf(&"out of index\n\0"[0]);
            return vec;
        }
        current = current->next;
    }
    return current;
}

void vec_free(Vector* vec) {
    if (vec->is_end == 1) {
        return;
    }
    vec->is_end = 1;
    Vector* current = vec->next;
    while (current->is_end == 0) {
        Vector* tmp = current->next;
        free((char*)current);
        current = tmp;
    }
    free((char*)current);
}

void capacity(Vector* vec, int size) {
    Vector* current = vec;
    int current_size = len(vec);

    int i = current_size;
    for (; i < size; i++) {
        Vector* last = end(vec);
        append(last);
    }

    Vector* enough = vec_index(vec, size - 1);
    vec_free(enough);
}

typedef struct {
    Canvas* canvas;
    int wide;
    int height;
    Vector* wrote;
    Colour base;
} Map;

Map map_new(int wide, int height, Colour base) {
    Map tmp;
    tmp.canvas = canvas_init(wide, height, base);
    tmp.height = height;
    tmp.wide = wide;
    tmp.wrote = vec_init();
    tmp.base = base;
    return tmp;
}

Map map_init(int wide, int height, Colour base) {
    return map_new(wide, height, base);
}

void setblock(Map* map, Block block) {
    setpixel(block.xy, block.colour, map->canvas);
    append(map->wrote)->data.block = block;
}

void clear_map(Map* map) {
    int i = 0;
    for (i = 0; i < len(map->wrote); i++) {
        Block now = vec_index(map->wrote, i)->data.block;
        setpixel(now.xy, map->base, map->canvas);
    }
    vec_free(map->wrote);
    end(map->wrote)->data = data_init();
}

void setblocks(Map* map, Vector* blocks) {
    int i = 0;
    for (i = 0; i < len(blocks); i++) {
        Block now = vec_index(blocks, i)->data.block;
        setblock(map, now);
    }
}

// // // 正12面体の頂点を生成
Vector* create_dodecahedron_vertices(void) {
    Vector* vertices = vec_init();
    capacity(vertices, 20);

    double phi = 1.618033988749895;     // 黄金比
    double invphi = 0.618033988749895;  // 1/phi

    // 正12面体の20個の頂点座標
    double coords[2][3] = {
        // 立方体の面の中央
        {1.0, 1.0, 1.0},
        {1.0, 1.0, -1.0},
    };

    int i = 0;
    for (i = 0; i < 20; i++) {
        Vec3 vertex = vec3_new(coords[i][0], coords[i][1], coords[i][2]);
        vec_index(vertices, i)->data.block =
            block_new(coordinates_init(), colour_new(0, 0, 255));
        // 頂点座標を一時的にblock構造体に格納（後で変換）
    }

    return vertices;
}

// // 3D座標を2D画面座標に投影（改良版）
Coordinates project_3d_to_2d(Vec3 point, int screen_width, int screen_height,
                             double scale) {
    double distance = 8.0;  // カメラ距離を増加

    // 透視投影
    double projected_x = (point.vx * scale * 100.0) / (distance - point.vz) +
                         (double)(screen_width / 2);
    double projected_y = (point.vy * scale * 100.0) / (distance - point.vz) +
                         (double)(screen_height / 2);

    return coordinates_new((int)projected_x, (int)projected_y);
}

// 3D回転変換
Vec3 rotate_point(Vec3 point, double angle_x, double angle_y, double angle_z) {
    Vec3 result = point;

    // Y軸回転
    double cos_y = cos(angle_y);
    double sin_y = sin(angle_y);
    double temp_x = result.vx * cos_y - result.vz * sin_y;
    double temp_z = result.vx * sin_y + result.vz * cos_y;
    result.vx = temp_x;
    result.vz = temp_z;

    // X軸回転
    double cos_x = cos(angle_x);
    double sin_x = sin(angle_x);
    double temp_y = result.vy * cos_x - result.vz * sin_x;
    temp_z = result.vy * sin_x + result.vz * cos_x;
    result.vy = temp_y;
    result.vz = temp_z;

    // Z軸回転
    double cos_z = cos(angle_z);
    double sin_z = sin(angle_z);
    temp_x = result.vx * cos_z - result.vy * sin_z;
    temp_y = result.vx * sin_z + result.vy * cos_z;
    result.vx = temp_x;
    result.vy = temp_y;

    return result;
}

// // 線を描画
void draw_line(Map* map, Coordinates start, Coordinates end, Colour colour) {
    int dx = end.x - start.x;
    int dy = end.y - start.y;
    int steps;
    int abs_dx = dx > 0 ? dx : -dx;
    int abs_dy = dy > 0 ? dy : -dy;

    if (abs_dx > abs_dy) {
        steps = abs_dx;
    } else {
        steps = abs_dy;
    }

    if (steps == 0) {
        setblock(map, block_new(start, colour));
        return;
    }

    double x_inc = (double)dx / (double)steps;
    double y_inc = (double)dy / (double)steps;

    double x = (double)start.x;
    double y = (double)start.y;

    int i = 0;
    for (i = 0; i <= steps; i++) {
        if ((int)x >= 0 && (int)x < map->wide && (int)y >= 0 &&
            (int)y < map->height) {
            setblock(map, block_new(coordinates_new((int)x, (int)y), colour));
        }
        x += x_inc;
        y += y_inc;
    }
}

void draw_dodecahedron(Map* map, Transform transform, Colour colour) {
    double phi = 1.6180339887;  // 黄金比
    double a = 1.0;             // 基本サイズ

    // 正確な正12面体の20個の頂点（正規化済み）
    Vec3 vertices[20] = {// 第1グループ：(±1, ±1, ±1)
                         {a, a, a},
                         {a, a, -a},
                         {a, -a, a},
                         {a, -a, -a},
                         {-a, a, a},
                         {-a, a, -a},
                         {-a, -a, a},
                         {-a, -a, -a},

                         // 第2グループ：(0, ±φ, ±1/φ)
                         {0.0, phi * a, a / phi},
                         {0.0, phi * a, -a / phi},
                         {0.0, -phi * a, a / phi},
                         {0.0, -phi * a, -a / phi},

                         // 第3グループ：(±1/φ, 0, ±φ)
                         {a / phi, 0.0, phi * a},
                         {a / phi, 0.0, -phi * a},
                         {-a / phi, 0.0, phi * a},
                         {-a / phi, 0.0, -phi * a},

                         // 第4グループ：(±φ, ±1/φ, 0)
                         {phi * a, a / phi, 0.0},
                         {phi * a, -a / phi, 0.0},
                         {-phi * a, a / phi, 0.0},
                         {-phi * a, -a / phi, 0.0}};

    // 正12面体の正確なエッジ（30個）
    int edges[30][2] = {
        // 第1グループの接続
        {0, 8},
        {0, 12},
        {0, 16},  // 頂点0から
        {1, 9},
        {1, 13},
        {1, 16},  // 頂点1から
        {2, 10},
        {2, 12},
        {2, 17},  // 頂点2から
        {3, 11},
        {3, 13},
        {3, 17},  // 頂点3から
        {4, 8},
        {4, 14},
        {4, 18},  // 頂点4から
        {5, 9},
        {5, 15},
        {5, 18},  // 頂点5から
        {6, 10},
        {6, 14},
        {6, 19},  // 頂点6から
        {7, 11},
        {7, 15},
        {7, 19},  // 頂点7から

        // グループ間の接続
        {8, 9},
        {10, 11},  // 第2グループ内
        {12, 14},
        {13, 15},  // 第3グループ内
        {16, 17},
        {18, 19}  // 第4グループ内
    };

    // 変換された頂点を格納
    Coordinates projected_vertices[20];

    int i = 0;
    for (i = 0; i < 20; i++) {
        // スケール適用
        Vec3 scaled = vec3_new(vertices[i].vx * transform.scale.vx,
                               vertices[i].vy * transform.scale.vy,
                               vertices[i].vz * transform.scale.vz);

        // 回転適用
        Vec3 rotated =
            rotate_point(scaled, transform.rotation.vx, transform.rotation.vy,
                         transform.rotation.vz);

        // 平行移動適用
        rotated.vx += transform.position.vx;
        rotated.vy += transform.position.vy;
        rotated.vz += transform.position.vz;

        // 2D投影
        projected_vertices[i] =
            project_3d_to_2d(rotated, map->wide, map->height, 1.0);
    }

    // エッジを描画
    for (i = 0; i < 30; i++) {
        int start_idx = edges[i][0];
        int end_idx = edges[i][1];

        // 画面外チェック
        Coordinates start = projected_vertices[start_idx];
        Coordinates end = projected_vertices[end_idx];

        if (start.x >= 0 && start.x < map->wide && start.y >= 0 &&
            start.y < map->height && end.x >= 0 && end.x < map->wide &&
            end.y >= 0 && end.y < map->height) {
            draw_line(map, start, end, colour);
        }
    }
}

int main(void) {
    Colour base = colour_new(0, 0, 0);      // 黒背景
    Colour blue = colour_new(0, 100, 255);  // 青色
    Map map = map_new(100, 100, base);

    double base_scale = 1.5;
    // スケール調整可能なTransformを作成
    Transform dodecahedron_transform = transform_new(
        vec3_new(0.0, 0.0, 0.0),                      // position
        vec3_new(0.0, 0.0, 0.0),                      // rotation
        vec3_new(base_scale, base_scale, base_scale)  // scale (0.8倍に縮小)
    );

    double rotation_speed = 0.03;  // 回転速度を調整
    int frame = 0;

    // // アニメーションループ
    while (frame < 300000) {  // 300フレーム実行
        clear_map(&map);

        // 回転角度を更新（異なる速度で回転させて立体感を出す）
        dodecahedron_transform.rotation.vx =
            (double)frame * rotation_speed * 0.7;
        dodecahedron_transform.rotation.vy = (double)frame * rotation_speed;
        dodecahedron_transform.rotation.vz =
            (double)frame * rotation_speed * 0.3;

        // スケールを時間で変化させる（オプション）
        double scale_factor =
            base_scale +
            0.6 * sin((double)frame * 0.02);  // 0.3-0.9の範囲で変動
        dodecahedron_transform.scale.vx = scale_factor;
        dodecahedron_transform.scale.vy = scale_factor;
        dodecahedron_transform.scale.vz = scale_factor;

        draw_dodecahedron(&map, dodecahedron_transform, blue);

        frame++;

        // フレームレート制御（簡易）
        int delay = 0;
        for (delay = 0; delay < 13000; delay++) {
            int delay1 = 0;
            for (delay1 = 0; delay1 < 1; delay1++) {
                move(0, map.canvas->height + 1);
            }
        }
    }

    return 0;
}
