double custom_abs(double a) {
    return a < 0 ? -a : a;
}

double custom_max(double a, double b) {
    return a > b ? a : b;
}

int is_close(double a, double b, double rel_tol, double abs_tol) {
    return custom_abs(a - b) <= custom_max(rel_tol * custom_max(custom_abs(a), custom_abs(b)), abs_tol);
}

double average(int len, int a[10]) {
    int sum = 0;
    int i;

    for(i = 0; i < len; i++) {
        sum += a[i];
    }

    return (double) sum / len;
}

int main() {
    int a[10];
    int len = 10;
    
    for (int i = 0; i < len; i++) {
        a[i] = i;
    }

    float avg = average(len, a);
    
    return is_close(avg, 4.5, 1e-09, 0.1);
}
