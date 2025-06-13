//**************************************************************************
// Median filter bencmark
//--------------------------------------------------------------------------
//
// This benchmark performs a 1D three element median filter.

int input_median[400];
int results_median[400];

void median_data_init(int nonce) {
    int i;
    int x = nonce;

    for (i = 0; i < 400; i++) {
        x = (x * 97 + 17) % 1000;
        input_median[i] = x;
    }
}

void median(int n, int input[400], int results[400]) {
    int A, B, C, i;

    // Zero the ends
    results[0] = 0;
    results[n - 1] = 0;

    // Do the filter
    for (i = 1; i < (n - 1); i++) {
        A = input[i - 1];
        B = input[i];
        C = input[i + 1];

        if (A < B) {
            if (B < C)
                results[i] = B;
            else if (C < A)
                results[i] = A;
            else
                results[i] = C;
        }

        else {
            if (A < C)
                results[i] = A;
            else if (C < B)
                results[i] = B;
            else
                results[i] = C;
        }
    }
}

int verify_median(int n, int* test) {
    int i;
    int result = 0;

    for (i = 0; i < n; i++) {
        int v = test[i];
        result ^= v;
    }

    return result;
}

int run_median(int dummy_0, int nonce) {
    median_data_init(nonce);
    median(400, input_median, results_median);

    return verify_median(400, results_median);
}
