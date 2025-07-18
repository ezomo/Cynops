int mod(int x, int y) {
  if (x * y == 0) {
    return 120;
  }

  while (x >= y) {
    x = x - y;
  }

  return x;
}

int gcd(int a, int b) {
  if (b == 0) {
    return a;
  } else {
    return gcd(b, mod(a, b));
  }

  return 0;
}

int main(void) { return gcd(630, 300); }