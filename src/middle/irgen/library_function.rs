// Mapping from function name to return type based on:

// #ifndef __SYLIB_H_
// #define __SYLIB_H_

// #include<stdio.h>
// #include<stdarg.h>
// #include<sys/time.h>
// /* Input & output functions */
// int getint(),getch(),getarray(int a[]);
// float getfloat();
// int getfarray(float a[]);

// void putint(int a),putch(int a),putarray(int n,int a[]);
// void putfloat(float a);
// void putfarray(int n, float a[]);

// void putf(char a[], ...);

// /* Timing function implementation */
// struct timeval _sysy_start,_sysy_end;
// #define starttime() _sysy_starttime(__LINE__)
// #define stoptime()  _sysy_stoptime(__LINE__)
// #define _SYSY_N 1024
// int _sysy_l1[_SYSY_N],_sysy_l2[_SYSY_N];
// int _sysy_h[_SYSY_N], _sysy_m[_SYSY_N],_sysy_s[_SYSY_N],_sysy_us[_SYSY_N];
// int _sysy_idx;
// __attribute((constructor)) void before_main();
// __attribute((destructor)) void after_main();
// void _sysy_starttime(int lineno);
// void _sysy_stoptime(int lineno);

// #endif

use crate::middle::ir::ValueType;

/// Get the return type of a library function
pub fn get_library_return_type(func_name: &str) -> Option<ValueType> {
    match func_name {
        "getint" => Some(ValueType::Int),
        "getch" => Some(ValueType::Int),
        "getarray" => Some(ValueType::Int),
        "getfloat" => Some(ValueType::Float),
        "getfarray" => Some(ValueType::Int),
        "putint" => Some(ValueType::Void),
        "putch" => Some(ValueType::Void),
        "putarray" => Some(ValueType::Void),
        "putfloat" => Some(ValueType::Void),
        "putfarray" => Some(ValueType::Void),
        "putf" => Some(ValueType::Void),
        "starttime" => Some(ValueType::Void),
        "stoptime" => Some(ValueType::Void),
        _ => None,
    }
}
