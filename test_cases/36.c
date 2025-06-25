int *filter(int (*judge)(int), int *list, int len, int *out_len) {
  int count = 0;
  int i;
  for (i = 0; i < len; i++) {
    if (judge(list[i]))
      rlist[count++] = list[i];
  }
  *out_len = count;
  return rlist;
}

int j(int x) { return x > 500; }

int main() {
  int list[1000];
  int i;
  for (i = 0; i < 1000; i++) {
  }
  int len = 1000;
  int filtered_len = 0;
  int *l = filter(j, list, len, &filtered_len);

  free(l); // ← メモリリーク防止
}