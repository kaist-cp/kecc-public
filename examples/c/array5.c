int g_a[5] = {1, 2, 3};

int main() {
    int init = 1;
    int a[5] = {init, 2, 3, 4, -5};
    int sum = 0;

    for(int i = 0; i < 5; i++) {
        sum += a[i];
        sum += g_a[i];
    }
    
    return sum;
}
