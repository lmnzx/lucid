#include <assert.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <unistd.h>
#include <errno.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

static void die(const char *msg)
{
    int err = errno;
    fprintf(stderr, "[%d] %s\n", err, msg);
    abort();
}

int main()
{
    int fd = socket(AF_INET, SOCK_STREAM, 0);

    if (fd < 0)
        die("socket");

    struct sockaddr_in addr = {};
    addr.sin_family = AF_INET;
    addr.sin_port = ntohs(1234);
    addr.sin_addr.s_addr = htonl(INADDR_LOOPBACK); // 127.0.0.1

    // connect to server
    int rv = connect(fd, (struct sockaddr *)&addr, sizeof(addr));

    if (rv < 0)
        die("connect");

    // send data
    char msg[] = "hello";
    write(fd, msg, strlen(msg));

    // receive data
    char rbuf[64] = {};
    ssize_t n = read(fd, rbuf, sizeof(rbuf) - 1);

    if (n < 0)
        die("read");

    printf("server says: %s\n", rbuf);
    close(fd);
    return 0;
}