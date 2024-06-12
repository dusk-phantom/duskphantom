	.text
	.attribute	4, 16
	.attribute	5, "rv64i2p0_m2p0_a2p0_f2p0_d2p0_c2p0"
	.file	"1.sy"
	.globl	main
	.p2align	1
	.type	main,@function
main:
	addi	sp, sp, -32
	sd	ra, 24(sp)
	sd	s0, 16(sp)
	addi	s0, sp, 32
	li	a0, 0
	sd	a0, -32(s0)
	sw	a0, -20(s0)
	call	f1
	mv	a1, a0
	ld	a0, -32(s0)
	sw	a1, -24(s0)
	lw	a1, -24(s0)
	blt	a0, a1, .LBB0_2
	j	.LBB0_1
.LBB0_1:
	li	a0, 1
	sw	a0, -20(s0)
	j	.LBB0_3
.LBB0_2:
	li	a0, 0
	sw	a0, -20(s0)
	j	.LBB0_3
.LBB0_3:
	lw	a0, -20(s0)
	ld	ra, 24(sp)
	ld	s0, 16(sp)
	addi	sp, sp, 32
	ret
.Lfunc_end0:
	.size	main, .Lfunc_end0-main

	.ident	"Ubuntu clang version 16.0.6 (++20231112100510+7cbf1a259152-1~exp1~20231112100554.106)"
	.section	".note.GNU-stack","",@progbits
	.addrsig
	.addrsig_sym f1
