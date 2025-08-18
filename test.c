int main(void) {
  int a[4] = {0, 1, 2, 3};
  int (*b)[4] = &a;
  int (**c)[4] = &b;
  (**c)[3] = 100;

  return a[3];
}
