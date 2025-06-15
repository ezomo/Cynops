int id(int a) { return a; }

int square(int a) { return a * a; }

int max(int a, int b) {
  if (a > b)
    return a;
  else
    return b;
}

int fact(int n) {
  int i;
  int result;
  i = 1;
  result = 1;
  while (i <= n) {
    result = result * i;
    i = i + 1;
  }
  return result;
}
