use super::reg_alloc::backend_from_self;

#[test]
fn test_simplify_term() {
    let code = "int lim;
int fun(int n,int dep){
	if(n==1) return dep;
	else{
		if(n%2==0) return fun(n/2,dep+1);
		else{
			if(n*3+1<=lim)
				return fun(n*3+1,dep+1);
			else
				return 0;
		}
	}
}

const int mod = 1000000007;

int main(){
	lim = getint();
	int ans = 0;
	int i = 1;
        starttime();
	while(i<=lim){
		ans = (ans+fun(i,0))%mod;
		i = i+1;
	}
        stoptime();
	putint(ans);
	putch(10);
}";
    let prog = backend_from_self(code);
    let mut f = prog
        .modules
        .first()
        .unwrap()
        .funcs
        .iter()
        .find(|f| f.name() == "main")
        .unwrap()
        .clone();
    let f_old = f.clone();
    f.simplify_term().unwrap();
    f.desimplify_term().unwrap();
    assert_eq!(f.gen_asm(), f_old.gen_asm());
}
