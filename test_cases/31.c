/* 8.3 複雑な式 */
int main(void) {
  int a = 10;
  int b = 5;
  int c = 3;
  int result;

  result = (a > b) ? (a + b * c) : (a - b / c);
  result = a++ * --b + c;
  result = (a & 0) | ((b << 8) & 1);

  return 0;
}
