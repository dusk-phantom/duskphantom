@g = dso_local global i32 0
@h = dso_local global i32 0
@f = dso_local global i32 0
@e = dso_local global i32 0
define i32 @EightWhile() {
entry:
%alloca_2 = alloca i32
%alloca_5 = alloca i32
store i32 5, ptr %alloca_5
%alloca_7 = alloca i32
%alloca_8 = alloca i32
store i32 6, ptr %alloca_7
store i32 7, ptr %alloca_8
%alloca_11 = alloca i32
store i32 10, ptr %alloca_11
br label %cond0

cond0:
%load_119 = load i32, ptr %alloca_5
%icmp_120 = icmp slt i32 %load_119, 20
br i1 %icmp_120, label %body1, label %final2

body1:
%load_17 = load i32, ptr %alloca_5
%Add_18 = add i32 %load_17, 3
store i32 %Add_18, ptr %alloca_5
br label %cond3

final2:
%load_122 = load i32, ptr %alloca_7
%load_123 = load i32, ptr %alloca_11
%Add_124 = add i32 %load_122, %load_123
%load_125 = load i32, ptr %alloca_5
%Add_126 = add i32 %load_125, %Add_124
%load_127 = load i32, ptr %alloca_8
%Add_128 = add i32 %Add_126, %load_127
%load_129 = load i32, ptr @e
%load_130 = load i32, ptr %alloca_11
%Add_131 = add i32 %load_129, %load_130
%load_132 = load i32, ptr @g
%Sub_133 = sub i32 %Add_131, %load_132
%load_134 = load i32, ptr @h
%Add_135 = add i32 %Sub_133, %load_134
%Sub_136 = sub i32 %Add_128, %Add_135
store i32 %Sub_136, ptr %alloca_2
br label %exit

cond3:
%load_112 = load i32, ptr %alloca_7
%icmp_113 = icmp slt i32 %load_112, 10
br i1 %icmp_113, label %body4, label %final5

exit:
%load_3 = load i32, ptr %alloca_2
ret i32 %load_3

body4:
%load_24 = load i32, ptr %alloca_7
%Add_25 = add i32 %load_24, 1
store i32 %Add_25, ptr %alloca_7
br label %cond6

final5:
%load_115 = load i32, ptr %alloca_7
%Sub_116 = sub i32 %load_115, 2
store i32 %Sub_116, ptr %alloca_7
br label %cond0

cond6:
%load_105 = load i32, ptr %alloca_8
%icmp_106 = icmp eq i32 %load_105, 7
br i1 %icmp_106, label %body7, label %final8

body7:
%load_31 = load i32, ptr %alloca_8
%Sub_32 = sub i32 %load_31, 1
store i32 %Sub_32, ptr %alloca_8
br label %cond9

final8:
%load_108 = load i32, ptr %alloca_8
%Add_109 = add i32 %load_108, 1
store i32 %Add_109, ptr %alloca_8
br label %cond3

cond9:
%load_98 = load i32, ptr %alloca_11
%icmp_99 = icmp slt i32 %load_98, 20
br i1 %icmp_99, label %body10, label %final11

body10:
%load_38 = load i32, ptr %alloca_11
%Add_39 = add i32 %load_38, 3
store i32 %Add_39, ptr %alloca_11
br label %cond12

final11:
%load_101 = load i32, ptr %alloca_11
%Sub_102 = sub i32 %load_101, 1
store i32 %Sub_102, ptr %alloca_11
br label %cond6

cond12:
%load_91 = load i32, ptr @e
%icmp_92 = icmp sgt i32 %load_91, 1
br i1 %icmp_92, label %body13, label %final14

body13:
%load_45 = load i32, ptr @e
%Sub_46 = sub i32 %load_45, 1
store i32 %Sub_46, ptr @e
br label %cond15

final14:
%load_94 = load i32, ptr @e
%Add_95 = add i32 %load_94, 1
store i32 %Add_95, ptr @e
br label %cond9

cond15:
%load_84 = load i32, ptr @f
%icmp_85 = icmp sgt i32 %load_84, 2
br i1 %icmp_85, label %body16, label %final17

body16:
%load_52 = load i32, ptr @f
%Sub_53 = sub i32 %load_52, 2
store i32 %Sub_53, ptr @f
br label %cond18

final17:
%load_87 = load i32, ptr @f
%Add_88 = add i32 %load_87, 1
store i32 %Add_88, ptr @f
br label %cond12

cond18:
%load_77 = load i32, ptr @g
%icmp_78 = icmp slt i32 %load_77, 3
br i1 %icmp_78, label %body19, label %final20

body19:
%load_59 = load i32, ptr @g
%Add_60 = add i32 %load_59, 10
store i32 %Add_60, ptr @g
br label %cond21

final20:
%load_80 = load i32, ptr @g
%Sub_81 = sub i32 %load_80, 8
store i32 %Sub_81, ptr @g
br label %cond15

cond21:
%load_70 = load i32, ptr @h
%icmp_71 = icmp slt i32 %load_70, 10
br i1 %icmp_71, label %body22, label %final23

body22:
%load_66 = load i32, ptr @h
%Add_67 = add i32 %load_66, 8
store i32 %Add_67, ptr @h
br label %cond21

final23:
%load_73 = load i32, ptr @h
%Sub_74 = sub i32 %load_73, 1
store i32 %Sub_74, ptr @h
br label %cond18


}
define i32 @main() {
entry:
%alloca_141 = alloca i32
store i32 1, ptr @g
store i32 2, ptr @h
store i32 4, ptr @e
store i32 6, ptr @f
%call_148 = call i32 @EightWhile()
store i32 %call_148, ptr %alloca_141
br label %exit

exit:
%load_142 = load i32, ptr %alloca_141
ret i32 %load_142


}
