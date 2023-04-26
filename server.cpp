#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <errno.h>
#include <unistd.h>
#include <sys/socket.h>
#include <netinet/in.h>
#include <arpa/inet.h>

static void die(const char *msg)
{
    int err = errno;
    fprintf(stderr, "[%d] %s\n", err, msg);
    abort();
}

static void msg(const char *msg)
{
    fprintf(stderr, "%s\n", msg);
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
    write(connfd, wbuf, sizeof(wbuf));
}

int main()
{
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0)
        die("socket() error");

    int val = 1;
    setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &val, sizeof(val));

    // bind
    struct sockaddr_in addr = {};
    addr.sin_family = AF_INET;
    addr.sin_port = htons(1234);                   // port
    addr.sin_addr.s_addr = ntohl(INADDR_LOOPBACK); // localhost
    if (bind(fd, (struct sockaddr *)&addr, sizeof(addr)) < 0)
        die("bind() error");

    // listen
    if (listen(fd, SOMAXCONN) < 0)
        die("listen() error");

    while (true)
    {
        // accept
        struct sockaddr_in client_addr = {};
        socklen_t client_addrlen = sizeof(client_addr);
        int connfd = accept(fd, (struct sockaddr *)&client_addr, &client_addrlen);
        if (connfd < 0)
            continue;

        do_something(connfd);
        close(connfd);
    }

    return 0;
}