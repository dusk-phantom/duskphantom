#!/bin/bash

# 示范运行脚本,放到符合规范的数据集下面执行
# echo "task.sh: start"

echo "qemu-riscv64 --version" > version.log
qemu-riscv64 --version >> version.log
echo "riscv64-linux-gnu-gcc-12 --version" >> version.log
riscv64-linux-gnu-gcc-12 --version >> version.log

# 输出当前时间到总日志中
echo $(date +%Y-%m-%d\ %H:%M:%S) >> all.log
# 读取time.log文件最后一行,获取最后一次修改文件时间
# 判断sy文件夹,asm文件夹,in文件夹,out文件夹,log文件夹,lib文件夹 是否存在
# 以要检查的文件夹名为参数,实现检查文件夹是否存在,不存在则创建
function check_folder() {
    dir=$1
    # 如果 $dir 文件夹不存在,则创建
    if [ ! -d "./$dir" ]; then
        mkdir $dir
    fi
}
check_folder sy
check_folder asm
check_folder my_asm
check_folder my_logs
check_folder my_out
check_folder my_exec
check_folder exec
check_folder in
check_folder out
check_folder logs
check_folder time
check_folder lib

# TODO,检查lib中的函数库是否存在

# 一个shell函数,获取制定$1文件夹参数中的 所有.sy结尾的且修改时间发生了更新的文件
function edited_files() {
    # 获取$1文件夹中所有以$2结尾的文件
    # 以空格分割,并且赋值给files变量
    files=$(find $1 -name "*.$2")
    # 遍历files变量中的所有文件
    for file in $files; do
        # 获取文件名,不包含后缀
        filename=$(basename $file .sy)
        # 获取文件的最后修改时间
        last_edit=$(stat -c %Y $file)
        # 获取文件无后缀名,使用basename工具以外的方式获得
        base_name=$(basename $file .sy)
        # 从 time/$base_name 中读取 上一次修改时间 
        time_path="time/${base_name}.time" 
        # 如果文件不存在,则赋值为0,如果文件为空,则赋值为0
        last_time=$(cat $time_path 2>/dev/null  || echo 0)
        # 如果文件的最后修改时间大于上一次测试时间,则输出
        if [ $last_edit -gt $last_time ]; then
            echo $filename
            # 并且把新修改时间写入对应文件
            echo $last_edit > $time_path
        fi
    done
}

# 遍历sy文件夹中的所有文件,取出基础文件名
# 遍历in文件夹中的所有文件,取出基础文件名,
# 合并两个基础文件名列表,去除重复部分

# 代码编写如下
# 1. 遍历sy文件夹中的所有文件,取出基础文件名
sy_edited_base_names=$(edited_files sy sy)
# 2. 遍历in文件夹中的所有文件,取出基础文件名,
in_edited_base_names=$(edited_files in in)
# 3. 合并两个基础文件名列表,去除重复部分
edited_base_names=$(echo $sy_edited_base_names $in_edited_base_names | tr ' ' '\n' | sort | uniq)

# 测试代码
for base_name in $edited_base_names; do
    echo "test $base_name" >> log
    # 使用 riscv64-linux-gnu-gcc-12 编译 sy/$base_name.sy 文件 为 asm/$base_name.s 文件
    # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
    riscv64-linux-gnu-gcc-12 -x c sy/$base_name.sy -S -o asm/$base_name.s -O3 2>logs/$base_name.log
    # 如果编译成功,则编译 asm/$base_name.s 文件 为 exec/$base_name 文件
    # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
    if [ $? -eq 0 ]; then
        riscv64-linux-gnu-gcc-12 asm/$base_name.s -o exec/$base_name 2>>logs/$base_name.log
    fi
    # 如果编译仍然成功
    # 则判断 in/$base_name.in 文件是否存在,
    # 如果存在,则 使用 qemu-riscv64 执行 exec/$base_name < in/$base_name.in > out/$base_name.out
    # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
    if [ $? -eq 0 ]; then
        if [ -f in/$base_name.in ]; then
            echo "with input"
            qemu-riscv64 -L /usr/riscv64-linux-gnu exec/$base_name <in/$base_name.in >out/$base_name.out 2>>logs/$base_name.log
        else
            echo "without input"
            qemu-riscv64 -L /usr/riscv64-linux-gnu exec/$base_name >out/$base_name.out 2>>logs/$base_name.log
        fi
        # 返回值换行写入到 out/$base_name.out 文件中
        ret=$?  
        echo "" >> out/$base_name.out
        echo $ret >> out/$base_name.out
    fi
    
done

# 使用我们的编译器执行编译任务得到汇编到my_asm中
# 当且仅当当前目录下存在一个名为compiler的应用程序时执行
if [ -f ./compiler ]; then
    # 测试代码
    for base_name in $edited_base_names; do
        echo "test $base_name" >> my_logs/log
        # 使用 riscv64-linux-gnu-gcc-12 编译 sy/$base_name.sy 文件 为 asm/$base_name.s 文件
        # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
        ./compiler -S sy/$base_name.sy -o my_asm/$base_name.s 2>my_logs/$base_name.log
        # 如果编译成功,则编译 asm/$base_name.s 文件 为 exec/$base_name 文件
        # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
        if [ $? -eq 0 ]; then
            riscv64-linux-gnu-gcc-12 my_asm/$base_name.s -o my_exec/$base_name 2>>my_logs/$base_name.log
        fi
        # 如果编译仍然成功
        # 则判断 in/$base_name.in 文件是否存在,
        # 如果存在,则 使用 qemu-riscv64 执行 exec/$base_name < in/$base_name.in > out/$base_name.out
        # 并且把标准输出、标准错误输出,输出到log文件夹中的$base_name.log文件中
        if [ $? -eq 0 ]; then
            if [ -f in/$base_name.in ]; then
                echo "with input"
                qemu-riscv64 -L /usr/riscv64-linux-gnu my_exec/$base_name <in/$base_name.in >my_out/$base_name.out 2>>my_logs/$base_name.log
            else
                echo "without input"
                qemu-riscv64 -L /usr/riscv64-linux-gnu my_exec/$base_name >my_out/$base_name.out 2>>my_logs/$base_name.log
            fi
            # 返回值换行写入到 out/$base_name.out 文件中
            ret=$?  
            echo "" >> my_out/$base_name.out
            echo $ret >> my_out/$base_name.out
        fi
    done
fi

# echo "task.sh: end"
# 对比两个out中的文件,如果不同,则输出文件名和差异到diff.log文件中
# 并且把标准输出、标准错误输出,输出到log文件夹中的diff.log文件中
echo "##################### " >> diff.log
for base_name in $edited_base_names; do
    echo "test $base_name" >> diff.log
    diff out/$base_name.out my_out/$base_name.out >> diff.log
done

