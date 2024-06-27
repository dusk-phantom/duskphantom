.file "test.c"
.option pic
.attribute arch, "rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0_zifencei2p0"
.attribute unaligned_access, 0
.attribute stack_align, 16

.text
.align	3
.globl	main
.type	main, @function
main:
entry:
addi x32,zero,0
store x32,[0-4]
addi a0,zero,0
ret
.size	main, .-main
.ident	"compiler: (visionfive2) 0.1.0"
.section	.note.GNU-stack,"",@progbits
