int b[10];
int main() {
    int *a = b;
    a++;
    a += 3;
    b[4] = 10;
    return *a;
}
