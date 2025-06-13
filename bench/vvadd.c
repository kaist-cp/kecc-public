//**************************************************************************
// Vector-vector add benchmark
//--------------------------------------------------------------------------
//
// This benchmark uses adds to vectors and writes the results to a
// third vector.

int input1_vvadd[1000];
int input2_vvadd[1000];
int results_vvadd[1000];

void vvadd_init(int nonce) {
    int i;
    int x = nonce;
    int y = nonce;

    for (i = 0; i < 1000; i++) {
        x = (x * 97 + 17) % 1009;
        y = (x * 17 + 23) % 1007;
        input1_vvadd[i] = x;
        input2_vvadd[i] = y;
    }
}

void vvadd(int n, int a[1000], int b[1000], int c[1000]) {
    int i;

    for (i = 0; i < n; i++)
        c[i] = a[i] + b[i];
}

int verify_vvadd(int n, int* test) {
    int i;
    int result = 0;

    for (i = 0; i < n; i++) {
        int v = test[i];
        result ^= v;
    }

    return result;
}

int run_vvadd(int dummy_0, int nonce) {
    vvadd_init(nonce);
    vvadd(1000, input1_vvadd, input2_vvadd, results_vvadd);
    return verify_vvadd(1000, results_vvadd);
}
