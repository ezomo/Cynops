int *f(int x, char c) {
  int *p = x;
  if (p > 10) {
    x = x + p * 2 - (x / 3);
  } else {
    while (x < 100) {
      x = x + 1;
    }
  }
  return p;
}