// 余り計算
int remainder(int a, int b) {
  while (a >= b) {
    a = a - b;
  }
  return a;
}

// 素数カウント
int count_primes(int limit) {
  int count = 0;
  int n = 2;

  while (n <= limit) {
    int is_prime = 1;
    int i = 2;

    while (i * i <= n) {
      if (remainder(n, i) == 0) {
        is_prime = 0;
      }
      i = i + 1;
    }

    if (is_prime == 1) {
      count = count + 1;
    }

    n = n + 1;
  }

  return count;
}

int main(void) {
  return count_primes(100); // 100までの素数の個数
}