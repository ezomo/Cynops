int mod(int x, int y) {
  if (x * y == 0) {
    return 120;
  }

  while (x >= y) {
    x = x - y;
  }

  return x;
}

int is_prime(int n) {
  int i = 2;
  if (n <= 1)
    return 0;
  while (i * i <= n) {
    if (mod(n, i) == 0) {
      return 0;
    }
    i = i + 1;
  }
  return 1;
}

int main(void) {
  return is_prime(7); // 素数なら 1、そうでなければ 0 を返す
}