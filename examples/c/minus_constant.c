int a = -1;
long b = -1l;
float c = -1.5f;
double d = -1.5;

int main() {
    return (a + b + (int)c + (long)d) == -4;
}
