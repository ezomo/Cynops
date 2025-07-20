int main(void) {
  int b = 0;
  int *a = b;
  *a = 2;
  return *a; // 素数なら 1、そうでなければ 0 を返す
}