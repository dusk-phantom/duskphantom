declare i32 @getint()
declare i32 @getch()
declare float @getfloat()
declare i32 @getarray(ptr noundef %0)
declare i32 @getfarray(ptr noundef %0)
declare void @putint(i32 noundef %0)
declare void @putch(i32 noundef %0)
declare void @putarray(i32 noundef %0, ptr noundef %1)
declare void @putfloat(float noundef %0)
declare void @putfarray(i32 noundef %0, ptr noundef %1)
declare void @putf(ptr noundef %0, ...)
declare void @starttime()
declare void @stoptime()
declare void @llvm.memset.p0.i32(ptr nocapture writeonly, i8, i32, i1 immarg)