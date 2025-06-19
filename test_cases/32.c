/* 9.1 構文的に正しいがセマンティックに問題がある例 */
// 型チェックが必要
int main(void) {
    int x;
    char* p;
    x = p;  // 型の不整合
    return 0;
}
