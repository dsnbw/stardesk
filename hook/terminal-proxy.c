/*
 * terminal-proxy.c - Bridges StarDesk remote terminal sessions to a real Linux PTY shell.
 */

#define WIN32_LEAN_AND_MEAN
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define STARDESK_TERM_MAGIC 0x5354544Du /* "STTM" */

int main(int argc, char **argv)
{
    (void)argc; (void)argv;
    int port = 54194;
    FILE *cfg = fopen("C:\\stardesk-terminal.runtime", "r");
    if (cfg) {
        char line[128];
        while (fgets(line, sizeof(line), cfg)) {
            if (strncmp(line, "port=", 5) == 0) port = atoi(line + 5);
        }
        fclose(cfg);
    }

    WSADATA wsa;
    if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0) return 1;

    SOCKET s = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (s == INVALID_SOCKET) return 1;

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons((u_short)port);
    addr.sin_addr.s_addr = inet_addr("127.0.0.1");

    if (connect(s, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        closesocket(s);
        return 1;
    }

    uint32_t magic = htonl(STARDESK_TERM_MAGIC);
    send(s, (char *)&magic, 4, 0);

    /* Bi-directional pipe forwarding stdin/stdout with socket */
    HANDLE h_stdin = GetStdHandle(STD_INPUT_HANDLE);
    HANDLE h_stdout = GetStdHandle(STD_OUTPUT_HANDLE);

    /* Standard I/O loop */
    char buffer[4096];
    fd_set read_fds;
    struct timeval tv;

    for (;;) {
        FD_ZERO(&read_fds);
        FD_SET(s, &read_fds);
        tv.tv_sec = 0;
        tv.tv_usec = 10000;

        int sel = select(0, &read_fds, NULL, NULL, &tv);
        if (sel > 0 && FD_ISSET(s, &read_fds)) {
            int received = recv(s, buffer, sizeof(buffer), 0);
            if (received <= 0) break;
            DWORD written;
            WriteFile(h_stdout, buffer, received, &written, NULL);
        }

        DWORD avail = 0;
        if (PeekNamedPipe(h_stdin, NULL, 0, NULL, &avail, NULL) && avail > 0) {
            DWORD read_bytes = 0;
            if (ReadFile(h_stdin, buffer, sizeof(buffer) < avail ? sizeof(buffer) : avail, &read_bytes, NULL) && read_bytes > 0) {
                send(s, buffer, read_bytes, 0);
            }
        }
    }

    closesocket(s);
    WSACleanup();
    return 0;
}
