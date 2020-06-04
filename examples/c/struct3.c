struct Sub {
    long m1;
    long m2;
    long m3;
    long m4;
};

struct Big {
    struct Sub m1;
    struct Sub m2;
    struct Sub m3;
};

struct Big foo(struct Big p1) {
    struct Big r = p1;
    r.m1.m1 = 10;
    return r;
}

int main() {
    struct Big a = {{1, 2, 3, 4}, {2, 3, 4, 5}, {3, 4, 5, 6}};
    struct Big r = foo(a);
    return r.m1.m1 == 10;
}
