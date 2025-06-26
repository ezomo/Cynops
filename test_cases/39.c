void test() {
  a[1];     // 配列アクセス
  func();   // 関数呼び出し
  a++;      // 後置インクリメント
  b--;      // 後置デクリメント
  s.member; // メンバアクセス（.）
  p->field; // メンバアクセス（->）
}

void test() {
  obj.method()[3].x->y++; // 複合例
  a[1][2][3][4][5][6][7][8][9][10];
  b[a[1]];

  obj->list[func(1, 2)].next->get()[index++]--; // ← 複合構文
  a.b[3].c->d().e[4]++;               // ← 再帰的なメンバ・配列アクセス
  (((x)))[0]->y.z().w--;              // ← 多重括弧 + 複雑なsuffix列
  p[0]->arr[1] = call().data[2]->key; // ← 代入も含めたケース
  ((((((((((v))))))))))++;            // ← 多重括弧 + 後置演算
}

void basic_postfix_tests() {
  // 基本的な単一演算子
  a[0];
  b[99];
  func();
  method(1, 2, 3);
  obj.field;
  ptr->member;
  counter++;
  index--;

  // 二重組み合わせ
  arr[i++];
  list[--j];
  func()[0];
  get_array()[index];
  obj.array[5];
  ptr->data[key];
  obj.method();
  ptr->callback();
  obj.counter++;
  ptr->index--;

  // 三重組み合わせ
  arr[func()];
  list[obj.size];
  matrix[i][j];
  cube[x][y][z];
  obj.arr[10];
  ptr->list[key];
  get_obj().field;
  create_ptr()->data;
  obj.method()++;
  ptr->get_val()--;
}
void array_access_combinations() {
  // 多次元配列アクセス
  a[1][2];
  b[1][2][3];
  c[1][2][3][4];
  d[1][2][3][4][5];
  matrix[row][col][depth][time][layer];

  // 関数戻り値の配列アクセス
  get_array()[0];
  create_matrix()[i][j];
  func(x, y)[result];
  callback()[index++];
  method(a, b, c)[--pos];

  // 配列の配列
  arrays[0][1];
  lists[group][item];
  buffers[bank][page][offset];

  // 複雑な配列インデックス
  arr[func(1, 2, 3)];
  list[obj.get_index()];
  data[ptr->current_pos];
  buffer[counter++];
  matrix[i++][j--];
  cube[get_x()][get_y()][get_z()];
}

void function_call_combinations() {
  // 連続関数呼び出し
  func()();
  method()()();
  callback()()()();

  // 関数戻り値のメンバアクセス
  get_object().field;
  create_struct().member;
  factory().data.value;
  builder().config.settings.mode;

  // 関数戻り値へのポインタアクセス
  get_pointer()->field;
  create_ptr()->data->next;
  alloc_node()->left->right->value;

  // 引数付き関数呼び出し
  func(1, 2, 3);
  method(a, b, c, d, e);
  callback(obj.field, ptr->data);
  complex_func(arr[i], obj.method(), ptr->callback());

  // 関数呼び出し後の演算
  get_counter()++;
  create_index()--;
  func(x, y)++;
  method()--;
}

void member_access_combinations() {
  // ドット演算子チェーン
  obj.field;
  obj.nested.field;
  obj.level1.level2.level3;
  obj.a.b.c.d.e.f.g.h;

  // アロー演算子チェーン
  ptr->field;
  ptr->next->data;
  ptr->child->parent->sibling;
  ptr->a->b->c->d->e;

  // ドットとアローの混合
  obj.ptr->field;
  ptr->obj.field;
  obj.nested->deep.value;
  ptr->strsuct.inner->data;
  obj.list->head.next->value;

  // // メンバの配列アクセス
  obj.array[0];
  ptr->buffer[index];
  obj.matrix[i][j];
  ptr->data[key]->next;

  // // メンバ関数呼び出し
  obj.method();
  ptr->callback();
  obj.get_value();
  ptr->set_data(x);
}

void increment_decrement_combinations() {
  // 基本的な後置演算
  i++;
  j--;
  counter++;
  index--;

  // 配列要素の増減
  arr[0]++;
  list[index]--;
  matrix[i][j]++;

  // メンバの増減
  obj.counter++;
  ptr->index--;
  obj.nested.value++;
  ptr->data->count--;

  // 関数戻り値の増減
  get_value()++;
  create_counter()--;
  func()++;
  method()--;

  // 複合的な増減
  obj.array[i++];
  ptr->list[--j];
  (obj.get_ptr())->counter++;
  (ptr->get_object()).value--;
}

void complex_combinations() {
  // 超複雑な組み合わせ
  obj.method()[0].next->get_data()[key++]--;
  ptr->callback(x, y)[result].field->array[--index];
  ((factory().create()))->list[0]->data.buffer[pos++];

  // 多重括弧と複合演算子
  (((((obj))))).field;
  ((((ptr))))->data;
  (((func())))[0];
  ((((arr[0]))))[1];
  (((((counter++)))))--;

  // 長いチェーン
  a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t;
  p->a->b->c->d->e->f->g->h->i->j->k->l->m->n;
  obj[0][1][2][3][4][5][6][7][8][9][10][11][12];

  // 関数呼び出しチェーン
  func()()()()()()()()();
  a.b().c().d().e().f().g().h();
  p->get()->next()->prev()->data();

  // 混合チェーン
  obj.get_array()[func(x)].next->callback()[i++]--;
  ptr->data[obj.index].method()->result.value++;
  ((factory()))[0]->create().instance.data[key]++;

  // 代入を含む複合例
  result = obj.method()[index].field->get_data();
  ptr->array[i] = func().data[j]->value;
  obj.list[key++] = create_object()->initialize();

  // 条件式での使用
  if (obj.array[i++] > ptr->threshold--) {
    // ...
  }

  // 関数引数での使用
  callback(obj.data[i++], ptr->get_next()->value--);
  method(arr[func()], obj.counter++, ptr->index--);

  // 三項演算子での使用
  result = condition ? obj.a[i++] : ptr->b[j--];
  value = (counter++ > 10) ? get_max() : get_min()[index];
}

void stress_test_cases() {
  // ストレステスト用の極端なケース

  // 超多次元配列
  hypercube[a][b][c][d][e][f][g][h][i][j][k][l][m][n][o][p];

  // 超長メンバアクセスチェーン
  obj.a.b.c.d.e.f.g.h.i.j.k.l.m.n.o.p.q.r.s.t.u.v.w.x.y.z;
  ptr->a->b->c->d->e->f->g->h->i->j->k->l->m->n->o->p->q->r->s->t;

  // 超長関数呼び出しチェーン
  a()()()()()()()()()()()()()()()()()()()()();

  // 全演算子の複合
  ((((obj.method()[arr[func(x++)]].next->get_data()))[key--])++);

  // 極端な括弧ネスト
  ((((((((((((((((x))))))))))))))))++;

  // 複雑な代入式
  obj->data[i++] = ptr.array[func()].next->get()[--j];
  result.field = ((factory()))[index].create()->initialize()[key++];

  // 返り値の連鎖的使用
  get_object().get_array()[get_index()].get_pointer()->get_value()++;
  create_factory()->build_object().get_data()[compute_key()]--;

  // 条件演算子の入れ子
  result = (a++ > b--) ? obj.x[i++] : ptr->y[j--];
  value = condition ? get_obj().data[k++] : create_ptr()->buffer[l--];

  // 関数引数での複雑な式
  complex_function(obj.array[i++].method(), ptr->data[func(x, y)]->get_next(),
                   ((factory()))[index].create()->value++,
                   get_callback()[key--]);
}

void realistic_usage_examples() {
  // 一般的なデータ構造操作

  // 連結リスト走査
  node->next->data;
  list->head->next->value++;

  // 二次元配列操作
  matrix[row][col]++;
  image[y][x] = pixel_data[i++];

  // 構造体配列
  students[i].grades[j];
  employees[index].salary++;

  // 関数テーブル
  function_table[opcode]();
  callbacks[event_type](data);

  // バッファ操作
  buffer[offset++] = value;
  queue[rear--] = item;

  // ファイル/ストリーム操作風
  stream.buffer[stream.pos++];
  file->data[file->offset++];

  // GUI関連風
  window.widgets[i].properties.visible;
  dialog->buttons[OK_BUTTON].callback();

  // ゲーム関連風
  player.inventory[slot].item->use();
  world.chunks[x][z].blocks[y]++;

  // データベース関連風
  table->rows[i].columns[j].data;
  query.results[index].fields[name];
}