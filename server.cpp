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

static void msg(const char *msg)
{
    fprintf(stderr, "%s\n", msg);
}

static void die(const char *msg)
{
    int err = errno;
    fprintf(stderr, "[%d] %s\n", err, msg);
    abort();
}

static void do_something(int connfd)
{
    char rbuf[64] = {};

    ssize_t n = read(connfd, rbuf, sizeof(rbuf) - 1);
    if (n < 0)
    {
        msg("read() error");
        return;
    }

    printf("client says: %s\n", rbuf);

    char wbuf[] = "world";
    write(connfd, wbuf, strlen(wbuf));
}

int main()
{
    int fd = socket(AF_INET, SOCK_STREAM, 0);

    if (fd < 0)
        die("socket() error");

    // optval is 1, so we can reuse the port immediately after the server exits.
    int optval = 1;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &optval, sizeof(optval));

    // bind to port 1234
    struct sockaddr_in addr = {};
    addr.sin_family = AF_INET;
    addr.sin_port = ntohs(1234);
    addr.sin_addr.s_addr = ntohl(0); // wildcard address

    int rv = bind(fd, (struct sockaddr *)&addr, sizeof(addr));

    if (rv < 0)
        die("bind() error");

    // listen for incoming connections
    rv = listen(fd, SOMAXCONN);
    if (rv < 0)
        die("listen() error");

    while (true)
    {
        // accept a connection
        struct sockaddr_in client_addr = {};
        socklen_t socklen = sizeof(client_addr);
        int connfd = accept(fd, (struct sockaddr *)&client_addr, &socklen);
        if (connfd < 0)
            continue; // error is neglected

        do_something(connfd);
        close(connfd);
    }

    return 0;
}