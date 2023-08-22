CC = clang
CXXFLAGS = -Wall -Wextra -O2 -g

all: server 

server: server.cpp
	$(CC) $(CXXFLAGS) -std=c++20 -lc++ server.cpp -o server

clean:
	rm -f server 
