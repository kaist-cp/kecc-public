int b[10];
int main() {
    int *a = 1 + b;
    a += 3;
    b[4] = 10;
    return *a;
}
