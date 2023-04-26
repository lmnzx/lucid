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
        die("socket() error");

    // connect
    struct sockaddr_in addr = {};
    addr.sin_family = AF_INET;
    addr.sin_port = htons(1234);                   // port
    addr.sin_addr.s_addr = ntohl(INADDR_LOOPBACK); // localhost
    if (connect(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0)
        die("connect() error");

    // send
    char msg[] = "hello";
    write(fd, msg, sizeof(msg));

    // recv
    char rbuf[64] = {};
    ssize_t n = read(fd, rbuf, sizeof(rbuf) - 1);
    if (n < 0)
        die("read() error");

    printf("server say: %s\n", rbuf);
    close(fd);

    return 0;
}