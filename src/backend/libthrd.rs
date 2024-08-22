// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

pub const LIB_THRD: &str = r#"

# ########## ########## thrd.c ########## ########## #

	.text
	.globl              STACK_SIZE
	.globl              tmp_mem
	.globl              tids
	.section            .rodata
	.align              3
	.type               STACK_SIZE, @object
	.size               STACK_SIZE, 8
STACK_SIZE:
	.dword              536870912
	.bss
	.align              3
	.type               tmp_mem, @object
	.size               tmp_mem, 2304
tmp_mem:
	.zero               2304
	.type               tids, @object
	.size               tids, 36
tids:
	.zero               36

# ########## ########## thrd_join.c ########## ########## #

	.text
	.align              1
	.globl              thrd_join
	.type               thrd_join, @function
thrd_join:
.thrd_join_LFB65:
	.cfi_startproc
	addi                sp,sp,-64
	.cfi_def_cfa_offset 64
	li                  a0,178
	sd                  s1,40(sp)
	sd                  ra,56(sp)
	.cfi_offset         9, -24
	.cfi_offset         1, -8
	la                  s1,tids
	call                syscall@plt
	lw                  a5,4(s1)
	beq                 a0,a5,.thrd_join_L16
	li                  a5,-1
	amoadd.w.aqrl       zero,a5,0(s1)
.thrd_join_L8:
	li                  a1,0
	li                  a0,93
	call                syscall@plt
	j                   .thrd_join_L8
.thrd_join_L16:
	sd                  s2,32(sp)
	li                  a5,-1
	amoadd.w.aqrl       zero,a5,0(s1)
	.cfi_offset         18, -32
	j                   .thrd_join_L3
.thrd_join_L4:
#APP
# 15 "src/thrd_join.c" 1
	nop
# 0 "" 2
#NO_APP
.thrd_join_L3:
	fence               rw,rw
	lw                  a5,0(s1)
	fence               r,rw
	sext.w              a5,a5
	bgt                 a5,zero,.thrd_join_L4
	li                  a5,2
	sw                  a5,4(sp)
	lw                  a5,4(sp)
	la                  s2,tmp_mem
	slliw               a5,a5,8
	add                 a5,s2,a5
	lbu                 a5,0(a5)
	beq                 a5,zero,.thrd_join_L5
	sd                  s3,24(sp)
	.cfi_offset         19, -40
	li                  s3,4096
	sd                  s0,48(sp)
	sd                  s4,16(sp)
	.cfi_offset         8, -16
	.cfi_offset         20, -48
	addi                s3,s3,-1792
.thrd_join_L7:
	lw                  s0,4(sp)
	slliw               s0,s0,8
	add                 s4,s2,s0
	ld                  a5,112(s4)
	sd                  a5,8(sp)
	ld                  a0,8(sp)
	call                free@plt
	li                  a2,128
	li                  a1,0
	mv                  a0,s4
	mv                  a3,s0
	bgeu                s0,s3,.thrd_join_L6
	mv                  a3,s3
.thrd_join_L6:
	sub                 a3,a3,s0
	call                __memset_chk@plt
	sd                  zero,8(sp)
	lw                  a5,4(sp)
	addiw               a5,a5,1
	sw                  a5,4(sp)
	lw                  a5,4(sp)
	slliw               a5,a5,8
	add                 a5,s2,a5
	lbu                 a5,0(a5)
	bne                 a5,zero,.thrd_join_L7
	ld                  s0,48(sp)
	.cfi_restore        8
	ld                  s3,24(sp)
	.cfi_restore        19
	ld                  s4,16(sp)
	.cfi_restore        20
.thrd_join_L5:
	sw                  zero,0(s1)
	sw                  zero,4(s1)
	sw                  zero,8(s1)
	sw                  zero,12(s1)
	sw                  zero,16(s1)
	sw                  zero,20(s1)
	sw                  zero,24(s1)
	sw                  zero,28(s1)
	sw                  zero,32(s1)
	ld                  ra,56(sp)
	.cfi_restore        1
	ld                  s2,32(sp)
	.cfi_restore        18
	ld                  s1,40(sp)
	.cfi_restore        9
	li                  a0,0
	addi                sp,sp,64
	.cfi_def_cfa_offset 0
	jr                  ra
	.cfi_endproc
.thrd_join_LFE65:
	.size               thrd_join, .-thrd_join

# ########## ########## thrd_create.c ########## ########## #

	.text
	.align              1
	.globl              __get_tid
	.type               __get_tid, @function
__get_tid:
.LFB6__get_tid:
	.cfi_startproc

# 判断是不是还没有创建过线程
	lla                 t1, tids
	lw                  t1, 0(t1)                                                      # cnt
	bnez                t1, .LFB6__get_tid_start
	li                  a0, 0
	jr                  ra

.LFB6__get_tid_start:
	addi                sp, sp, -8
	sd                  ra, 0(sp)
	call                gettid@plt
	mv                  a7, a0
.while_init__get_tid:
	lla                 a1, tids                                                       # a1 = tids 基址
	lw                  a0, 0(a1)                                                      # a0 = cnt
	j                   .while_cond__get_tid
.while_block__get_tid:
	addiw               a0, a0, -1
.while_cond__get_tid:
	blez                a0, .return__get_tid
	mv                  a2, a0
	slliw               a2, a2, 2
	add                 a2, a1, a2
	lw                  a2, 0(a2)                                                      # a2 = tids[a0]
	bne                 a2, a7, .while_block__get_tid

.return__get_tid:
# 返回 a0 ，这里 a0 就是循环变量
	ld                  ra, 0(sp)
	addi                sp, sp, 8
	addiw               a0, a0, -1                                                     # 下标 与 tid 正好差一位
	jr                  ra
	.cfi_endproc
.LFE6__get_tid:
	.size               __get_tid, .-__get_tid

	.text
	.align              1
	.globl              thrd_create
	.type               thrd_create, @function
thrd_create:
.__thrd_create_prepare: # 准备阶段
	.cfi_startproc

.__thrd_create_if_not_main_then_return:
	addi                sp, sp, -16
	sd                  ra, 0(sp)
	sd                  a0, 8(sp)                                                      # num
	call                __get_tid@plt
	ld                  ra, 0(sp)
	ld                  a1, 8(sp)                                                      # a1 = num
	addi                sp, sp, 16
	beqz                a0, .__thrd_create_fork_prepare                                # a0 = __get_tid()
# 是子线程调用了 thrd_create
	jr                  ra                                                             # 注意一下，我上面没有修改过 a0

.__thrd_create_fork_prepare:
# 也只有主线程能进来这里，那么这一部分是 顺序执行的
	sub                 a5, s0, sp                                                     # a5 = 主线程的 caller 栈 s0 - sp
	lla                 t1, tmp_mem
	sd                  a5, 0(t1)                                                      # s0 - sp
	sd                  a1, 8(t1)                                                      # num
	sd                  ra, 16(t1)                                                     # ra

.__thrd_create_if_cnt_0:
	lla                 t1, tids
	lw                  t2, 0(t1)
	bnez                t2, .__thrd_create_fork
# 如果是 0 ，那么就要初始化一下，总线程数 ++ ，tids[1] = gettid
	li                  t2, 1
	sw                  t2, 0(t1)
	li                  a0,178
	call                syscall@plt
	lla                 t1, tids
	sw                  a0, 4(t1)
	j                   .__thrd_create_fork

.__thrd_create_fork:
	lla                 t1, tmp_mem
	ld                  a0, 0(t1)
	call                _thrd_create@plt                                               # fork
	bnez                a0, .__thrd_create_son_while
	j                   .__thrd_create_main

.__thrd_create_son_while:
	nop
	lla                 t1, tmp_mem
	ld                  a1, 8(t1)                                                      # num
	bgtz                a1, .__thrd_create_son_while
	lla                 t1, tmp_mem
	ld                  ra, 16(t1)                                                     # ra
	jr                  ra                                                             # 这个时候 a0=id 是返回值
# 注意，这个时候不要破坏 a0 , 因为 a0 是返回值，也就是 tid


.__thrd_create_main: # 主线程
	lla                 t1, tmp_mem
	ld                  a1, 8(t1)
	addi                a1, a1, -1
	sd                  a1, 8(t1)
	bgtz                a1, .__thrd_create_fork
	lla                 t1, tmp_mem
	ld                  ra, 16(t1)                                                     # ra

	lla                 t1, tmp_mem
	li                  t2, 0

	li                  a0, 0                                                          # 主线程返回 0
	jr                  ra
	.cfi_endproc
.LFE6:
	.size               thrd_create, .-thrd_create

# ########## ########## son_leave.c ########## ########## #

	.text
	.align              1
	.globl              son_leave
	.type               son_leave, @function
son_leave:
	.cfi_startproc
	mv                  s8, a0                                                         # 保存一下 cnt 也就是 tid
	mv                  s11, sp                                                        # s11 = son sp

	lla                 a0, tmp_mem
	slliw               a2, s8, 8                                                      # cnt << 8
	add                 a1, a0, a2                                                     # a1 = tmp_mem[cnt]
	ld                  s10, 120(a1)                                                   # caller 栈大小 = tmp_mem[cnt][120]
	sub                 sp, sp, s10                                                    # 准备复制
	mv                  s9, a1                                                         # 暂存 s9 = tmp_mem[cnt]
	ld                  s7, 8(a1)                                                      # s7 = s0' 也就是 caller 的 s0
	sub                 s7, s7, s10                                                    # s7 = s7 - s10 得到 caller 的 sp

.LB_copy_frame:
	mv                  a0, sp                                                         # dest
	mv                  a1, s7                                                         # src
	mv                  a2, s10                                                        # 复制这么多的字节
	call                memcpy@plt                                                     # 浅拷贝 caller 的栈

	mv                  a7, s11                                                        # a7 看起来比较少用，son sp

.LB_restore:
	mv                  a1,s9                                                          # a1 = tmp_mem[cnt]
	mv                  a0,s8                                                          # FIXME s0 = cnt 返回值，后面不要改了

	ld                  s2,24(a1)
	ld                  s3,32(a1)
	ld                  s4,40(a1)
	ld                  s5,48(a1)
	ld                  s6,56(a1)
	ld                  s7,64(a1)
	ld                  s8,72(a1)
	ld                  s9,80(a1)
	ld                  s10,88(a1)
	ld                  s11,96(a1)

	ld                  ra, 0(a1)
	.cfi_restore        1
	mv                  s0, a7                                                         # 恢复 s0
	.cfi_restore        8
	ld                  s1, 16(a1)
	.cfi_restore        9
# li a0, 0 # 生成返回值
# a0 这个时候没有动过，注意
	addiw               a0, a0, -1
	jr                  ra
	.cfi_endproc
	.size               son_leave, .-son_leave

# ########## ########## son_leave.c ########## ########## #

	.text
	.align              1
	.globl              _thrd_create
	.type               _thrd_create, @function
_thrd_create:
._thrd_create_LFB7:
	.cfi_startproc
	addi                sp,sp,-64
	.cfi_def_cfa_offset 64
	sd                  ra,56(sp)
	sd                  s0,48(sp)
	sd                  s1,40(sp)
	sd                  a0, 32(sp)                                                     # a0 是传入的参数 = caller(s0-sp)
	.cfi_offset         1, -8
	.cfi_offset         8, -16
	.cfi_offset         9, -24
	addi                s0,sp,64
	.cfi_def_cfa        8, 0

._thrd_create_L5:
	lla                 a5,STACK_SIZE
	ld                  a4,0(a5)
	addi                a5,s0,-64
	mv                  a2,a4                                                          # s2 = stack_size
	li                  a1,16                                                          # a1 = 16 (align)
	mv                  a0,a5                                                          # a0 = a5 = &stack
	call                posix_memalign@plt
	ld                  a5,-64(s0)
	bnez                a5, ._thrd_create_L7                                           # if stack != NULL
	li                  a0, -1                                                         # fail
	ld                  ra,56(sp)
	.cfi_restore        1
	ld                  s0,48(sp)
	.cfi_restore        8
	.cfi_def_cfa        2, 64
	ld                  s1,40(sp)
	.cfi_restore        9
	addi                sp,sp,64
	.cfi_def_cfa_offset 0
	jr                  ra

._thrd_create_L7:
	ld                  a4,-64(s0)
	lla                 a5,STACK_SIZE
	ld                  a5,0(a5)
	add                 a5,a4,a5
	sd                  a5,-40(s0)                                                     # top
	li                  a5,20254720
	addi                a5,a5,-256
	sw                  a5,-44(s0)                                                     # flag

	lla                 t1, tids
	lw                  a5, 0(t1)
	addiw               a5, a5, 1
	sw                  a5, 0(t1)
	sw                  a5, -48(s0)                                                    # cnt

.LBB_backup:
	lla                 a1, tmp_mem
	lw                  a0, -48(s0)                                                    # cnt
	slliw               a0, a0, 8
	add                 a1,a1,a0                                                       # tmp_mem[cnt]
	ld                  ra, -8(s0)                                                     # ra
	sd                  ra,0(a1)
	ld                  a0, -16(s0)                                                    # s0'
	sd                  a0, 8(a1)
	ld                  a0, -24(s0)                                                    # s1
	sd                  a0,16(a1)
	sd                  s2,24(a1)
	sd                  s3,32(a1)
	sd                  s4,40(a1)
	sd                  s5,48(a1)
	sd                  s6,56(a1)
	sd                  s7,64(a1)
	sd                  s8,72(a1)
	sd                  s9,80(a1)
	sd                  s10,88(a1)
	sd                  s11,96(a1)

	ld                  a0, -32(s0)                                                    # caller 的 s0 - sp 栈大小
	sd                  a0, 120(a1)

# addi a0, sp, 64 # 因为 s0 是栈底
	mv                  a0, s0                                                         # 因为 s0 是栈底
	sd                  a0, 104(a1)                                                    # sp + 64 当前栈的 栈底

	ld                  a0, -64(s0)
	sd                  a0, 112(a1)                                                    # stack

.LB_clone:
	lw                  a5,-48(s0)
	slli                a4,a5,2
	lla                 a5,tids
	add                 a3,a4,a5
	lw                  a2,-44(s0)                                                     # a2 = flag
	li                  a6, 0                                                          # a6 = NULL
	li                  a5,0                                                           # a5 = NULL
	mv                  a4,a3                                                          # a4 = &tids[cnt] # 存放的是线程 tid
	lw                  a3,-48(s0)                                                     # a3 = 传入子线程的 id
	ld                  a1,-40(s0)                                                     # a1 = stack_top
	lla                 a0,son_leave                                                   # a0 = son_leave
	call                clone@plt                                                      # TODO 重点
	j                   .main_leave

.main_leave:
	li                  a0, 0                                                          # main thread 总是 == 0
	ld                  ra,56(sp)
	.cfi_restore        1
	ld                  s0,48(sp)
	.cfi_restore        8
	ld                  s1,40(sp)
	.cfi_restore        9
	addi                sp,sp,64
	.cfi_def_cfa_offset 0
	jr                  ra
	.cfi_endproc
._thrd_create_LFE7:
	.size               _thrd_create, .-_thrd_create

"#;
