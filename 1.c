int a = 3;
long long b[2] = {44, 2};
int c[2][2];
float ff = 2.2;
const char *name = "hello";

extern work(void *m);
int main()
{
    work(&a);
    work(b);
    work(c);
    work(&ff);
    work(name);
    return 0;
}