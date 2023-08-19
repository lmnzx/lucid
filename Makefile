CC = clang
CXXFLAGS = -Wall -Wextra -O2 -g

all: server client

server: server.cpp
	$(CC) $(CXXFLAGS) -std=c++20 -lc++ server.cpp -o server

client: client.cpp
	$(CC) $(CXXFLAGS) -std=c++20 client.cpp -o client

clean:
	rm -f server client
