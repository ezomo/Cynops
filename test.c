

int main(void) {
  int a = 1000;
  int *b = &a;

  *b = 20;

  int c;
  c = *b;

  return c;
}
