/* 1.5 ポインタ型の関数 */
int *get_array(void) { return &array[0]; }

void process_data(int *data, char *buffer) { *data = 42; }
