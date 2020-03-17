int main()
{
    int y = 2;
    int x = (y += 2, 2, y + 3);
    return x;
}
