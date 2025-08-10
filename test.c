int main(void) {
  int a = 5;
  int *p = &a;
  int **pp = &p;
  **pp = 10;
  return a;
}