int nonce = 1; // For random input

struct Foo
{
    int x;
};

struct Foo f()
{
    struct Foo x;
    x.x = nonce;
    return x;
}

int main()
{
    int x = f().x;
    return x;
}
