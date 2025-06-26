// 　構造体

struct Point {
  int x;
  int y;
};

struct RGB {
  char r, g, b;
};

struct Buffer {
  char *data;
  int size[10];
};

struct Handler {
  int (*callback)(int);
};

struct Complex {
  int *p1, arr[5], (*func)(char);
};