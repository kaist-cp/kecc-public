int nonce = 1; // For random input
int g = 10;

int foo(int, int k);

int main() {
    int i = g;
    
    return foo(i, i);
}

int foo(int i, int j) {
    return i + j + nonce;
}
