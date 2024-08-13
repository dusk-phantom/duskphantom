int arr1[10][2][3][4][5][6][2]; // size=10*2*3*4*5*6*2*4=172800
int arr2[10][2][3][2][4][8][7]; // size=10*2*3*2*4*8*7*4=107520
int loop3(int h, int i, int j, int k, int l, int m, int n) {
  int a, b, c, d, e, f, g;
  int ans = 0;
  a = 0;
  while (a < 10) {
    b = 0;
    while (b < 100) {
      c = 0;
      while (c < 1000) {
        d = 0;
        while (d < 10000) {
          e = 0;
          while (e < 100000) {
            f = 0;
            while (f < 1000000) {
              g = 0;
              while (g < 10000000) {
                ans = ans % 817 + arr1[a][b][c][d][e][f][g] +
                      arr2[a][b][c][d][e][f][g];
                g = g + 1;
                if (g >= n)
                  break;
              }
              f = f + 1;
              if (f >= m)
                break;
            }
            e = e + 1;
            if (e >= l)
              break;
          }
          d = d + 1;
          if (d >= k)
            break;
        }
        c = c + 1;
        if (c >= j)
          break;
      }
      b = b + 1;
      if (b >= i)
        break;
    }
    a = a + 1;
    if (a >= h)
      break;
  }
  return ans;
}
