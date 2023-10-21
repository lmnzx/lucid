cmake_minimum_required(VERSION 3.22)

project(lucid)

set(CMAKE_CXX_STANDARD 20)

# Collect all .cpp and .h files in the current directory
file(GLOB SOURCES "*.cpp" "*.h")
list(FILTER SOURCES EXCLUDE REGEX "test_.*")


# Create an executable (you can use add_library for a library)
add_executable(lucid ${SOURCES})

