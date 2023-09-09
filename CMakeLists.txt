cmake_minimum_required(VERSION 3.26)
project(lucid)

set(CMAKE_CXX_STANDARD 20)

# add_compile_options(-Wall -Wextra -o2 -g -lc++)

add_executable(lucid server.cpp
                avl.cpp
                avl.h
                common.h
                hashtable.cpp
                hashtable.h
                heap.h
                list.h
                thread_pool.cpp
                thread_pool.h
                zset.cpp
                zset.h  )
