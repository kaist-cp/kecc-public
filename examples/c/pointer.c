int* foo(int* a) {
    return a;
}

int main() {
    int a = 1;
    int* p = &a;
    int** p2 = &*&p;
    int* p3 = *&p;

    *&*foo(*p2) += 1;
    *foo(p3) += 1;

    return a == 3;
}
