int matrix_a[30][30];
int matrix_b[30][30];
int matrix_c[30][30];

void matrix_init(int n, int nonce, int* x, int (*matrix)[30]) {
    for (int i = 0; i < n; ++i) {
        for (int j = 0; j < n; ++j) {
            matrix[i][j] = ++*x;

            if (*x % (nonce + 1)) {
                ++*x;
            }
        }
    }
}

int matrix_mul(int n, int nonce) {
    if (!(n <= 30)) {
        return nonce;
    }

    int x = 0;
    matrix_init(n, nonce, &x, matrix_a);
    matrix_init(n, nonce, &x, matrix_b);

    int result = 0;
    for (int i = 0; i < n; ++i) {
        for (int j = 0; j < n; ++j) {
            matrix_c[i][j] = 0;
            for (int k = 0; k < n; ++k) {
                matrix_c[i][j] += matrix_a[i][k] * matrix_b[k][j];
            }
            result ^= matrix_c[i][j];
        }
    }

    return result;
}

int matrix_add(int n, int nonce) {
    if (!(n <= 30)) {
        return nonce;
    }

    int x = 0;
    matrix_init(n, nonce, &x, matrix_a);
    matrix_init(n, nonce, &x, matrix_b);

    int result = 0;
    for (int i = 0; i < n; ++i) {
        for (int j = 0; j < n; ++j) {
            matrix_c[i][j] = matrix_a[i][j] + nonce * matrix_b[i][j];
            result ^= matrix_c[i][j];
        }
    }

    return result;
}
