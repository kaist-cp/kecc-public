//**************************************************************************
// Quicksort benchmark
//--------------------------------------------------------------------------
//
// This benchmark uses quicksort to sort an array of integers. The
// implementation is largely adapted from Numerical Recipes for C.

int input_qsort_data[16384];

void qsort_data_init(int nonce) {
    int i;
    int x = nonce;

    for (i = 0; i < 16384; i++) {
        x = (x * 97 + 17) % 100009;
        input_qsort_data[i] = x;
    }
}

void swap(int* a, int* b) {
    int temp = *a;
    *a = *b;
    *b = temp;
}

void swap_if_greater(int* a, int* b) {
    if (*a > *b)
        swap(a, b);
}

void insertion_sort(int n, int* arr) {
    int i, j;
    int value;
    for (i = 1; i < n; i++) {
        value = arr[i];
        j = i;
        while (value < arr[j - 1]) {
            arr[j] = arr[j - 1];
            if (--j == 0)
                break;
        }
        arr[j] = value;
    }
}

void qsort(int n, int arr[16384]) {
    int ir = n;
    int l = 1;
    int stack[50];
    int stackp = 0;

    for (;;) {
        // Insertion sort when subarray small enough.
        if (ir - l < 10) {
            insertion_sort(ir - l + 1, &arr[l - 1]);

            if (stackp == 0)
                break;

            // Pop stack and begin a new round of partitioning.
            ir = stack[stackp--];
            l = stack[stackp--];
        } else {
            // Choose median of left, center, and right elements as
            // partitioning element a. Also rearrange so that a[l-1] <= a[l] <= a[ir-].
            swap(&arr[(l + ir) / 2 - 1], &arr[l]);
            swap_if_greater(&arr[l - 1], &arr[ir - 1]);
            swap_if_greater(&arr[l], &arr[ir - 1]);
            swap_if_greater(&arr[l - 1], &arr[l]);

            // Initialize pointers for partitioning.
            int i = l + 1;
            int j = ir;

            // Partitioning element.
            int a = arr[l];

            for (;;) {  // Beginning of innermost loop.
                while (arr[i++] < a)
                    ;  // Scan up to find element > a.
                while (arr[(j-- - 2)] > a)
                    ;  // Scan down to find element < a.
                if (j < i)
                    break;                       // Pointers crossed. Partitioning complete.
                swap(&arr[i - 1], &arr[j - 1]);  // Exchange elements.
            }  // End of innermost loop.

            // Insert partitioning element.
            arr[l] = arr[j - 1];
            arr[j - 1] = a;
            stackp += 2;

            // Push pointers to larger subarray on stack,
            // process smaller subarray immediately.

            if (ir - i + 1 >= j - l) {
                stack[stackp] = ir;
                stack[stackp - 1] = i;
                ir = j - 1;
            } else {
                stack[stackp] = j - 1;
                stack[stackp - 1] = l;
                l = i;
            }
        }
    }
}

int verify_qsort(int n, int* test) {
    int i;
    int result = 0;

    for (i = 0; i < n - 1; i++) {
        int t0 = test[i], t1 = test[i + 1];
        if (t0 > t1)
            return 1;

        result += t0;
    }

    return result;
}

int run_qsort(int dummy_0, int nonce) {
    qsort_data_init(nonce);
    qsort(16384, input_qsort_data);

    return verify_qsort(16384, input_qsort_data);
}
