int nonce = 1;  // For random input

int main() {
    int i;
    int p = 2;
    int q = 5;
    int r = (0 ? ((p > q) ? (p -= 2) : (p += 2)) : (p + q));
    int loop_num = nonce % 100;

    for (i = 0; i < loop_num; ((i % 2) ? (i += 2) : ++i)) {
        if (i % 2) {
            p += q;
        } else {
            p += r;
        }
    }

    return p;
}
