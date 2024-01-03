	.file	"1.c"
	.option pic
	.attribute arch, "rv64i2p1_m2p0_a2p1_f2p2_d2p2_c2p0_zicsr2p0_zifencei2p0"
	.attribute unaligned_access, 0
	.attribute stack_align, 16
	.text
	.section	.text.startup,"ax",@progbits
	.align	1
	.globl	main
	.type	main, @function
main:
	addi	sp,sp,-16
	lla	a0,.LANCHOR0
	sd	ra,8(sp)
	call	work@plt
	lla	a0,.LANCHOR0+8
	call	work@plt
	lla	a0,.LANCHOR1
	call	work@plt
	lla	a0,.LANCHOR0+24
	call	work@plt
	ld	a0,.LANCHOR2
	call	work@plt
	ld	ra,8(sp)
	li	a0,0
	addi	sp,sp,16
	jr	ra
	.size	main, .-main
	.globl	name
	.section	.rodata.str1.8,"aMS",@progbits,1
	.align	3
.LC0:
	.string	"hello"
	.globl	ff
	.globl	c
	.globl	b
	.globl	a
	.data
	.align	3
	.set	.LANCHOR0,. + 0
	.type	a, @object
	.size	a, 4
a:
	.word	3
	.zero	4
	.type	b, @object
	.size	b, 16
b:
	.dword	44
	.dword	2
	.type	ff, @object
	.size	ff, 4
ff:
	.word	1074580685
	.bss
	.align	3
	.set	.LANCHOR1,. + 0
	.type	c, @object
	.size	c, 16
c:
	.zero	16
	.section	.data.rel.local,"aw"
	.align	3
	.set	.LANCHOR2,. + 0
	.type	name, @object
	.size	name, 8
name:
	.dword	.LC0
	.ident	"GCC: (Ubuntu 12.3.0-1ubuntu1~22.04) 12.3.0"
	.section	.note.GNU-stack,"",@progbits
