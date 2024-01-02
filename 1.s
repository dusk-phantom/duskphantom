	.file	"1.c"
	.option pic
	.attribute arch, "rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0_zifencei2p0"
	.attribute unaligned_access, 0
	.attribute stack_align, 16
	.text
	.globl	a
	.data
	.align	2
	.type	a, @object
	.size	a, 4
a:
	.word	3
	.globl	b
	.align	3
	.type	b, @object
	.size	b, 16
b:
	.dword	44
	.dword	2
	.globl	c
	.bss
	.align	3
	.type	c, @object
	.size	c, 16
c:
	.zero	16
	.globl	ff
	.data
	.align	2
	.type	ff, @object
	.size	ff, 4
ff:
	.word	1074580685
	.text
	.align	1
	.globl	main
	.type	main, @function
main:
	addi	sp,sp,-16
	sd	ra,8(sp)
	sd	s0,0(sp)
	addi	s0,sp,16
	lla	a0,a
	call	work@plt
	lla	a0,b
	call	work@plt
	lla	a0,c
	call	work@plt
	li	a5,0
	mv	a0,a5
	ld	ra,8(sp)
	ld	s0,0(sp)
	addi	sp,sp,16
	jr	ra
	.size	main, .-main
	.ident	"GCC: (Ubuntu 12.3.0-1ubuntu1~22.04) 12.3.0"
	.section	.note.GNU-stack,"",@progbits
