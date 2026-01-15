void putchar(char);
char getchar(void);
void exit(void);

void main(void) {
    int key[256];
    int keylen = 0;
    char c;
    int decrypt = 0;

    /* 鍵の読み込み */
    while (((c = getchar()) != '\0') && (c != '@')) {
        if (keylen == 0 && c == '-') {
            decrypt = 1; /* 復号モード */
            continue;
        }
        if (keylen < 256) {
            if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) {
                /* 大文字に統一して 0-25 に変換 */
                if (c >= 'a') c -= (char)32;
                key[keylen] = (int)(c - 'A');
                keylen++;
            }
        }
    }

    if (keylen == 0) {
        return; /* 鍵なし */
    }

    int s = 0;
    int i = 0;
    while ((c = getchar()) != '\0') {
        s = 0;
        int out;
        if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) {
            /* 大文字に統一して 0-25 に変換 */
            if (c >= 'a') {
                c -= (char)32;
                s = 1;
            }
            int val = (int)(c - 'A');
            if (decrypt) {
                out = (val - key[i % keylen] + 26) % 26 + (int)'A';
            } else {
                out = (val + key[i % keylen]) % 26 + (int)'A';
            }
            out += 32 * s;
            i++;
        } else {
            out = (int)c;
        }
        putchar((char)out);
    }

    return;
}

// { printf "secret@"; cat README.md} | {./run.sh ./sample/vig.c} > enc.bin

// { printf "-secret@"; cat enc.bin } | {./run.sh ./sample/vig.c}
