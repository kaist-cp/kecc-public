int b[10];
int main() {
    int *a = b + 1;
    a += 3;
    b[4] = 10;
    return *a;
}
