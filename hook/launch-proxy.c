/*
 * launch-proxy.c - Bridges StarDesk application launch commands to native Linux applications.
 */

#define WIN32_LEAN_AND_MEAN
#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define STARDESK_LAUNCH_MAGIC 0x53544C41u /* "STLA" */

int main(int argc, char **argv)
{
    (void)argc; (void)argv;
    char module_path[MAX_PATH];
    GetModuleFileNameA(NULL, module_path, sizeof(module_path));
    char *last_slash = strrchr(module_path, '\\');
    if (last_slash) *last_slash = '\0';

    char id_file[MAX_PATH];
    snprintf(id_file, sizeof(id_file), "%s\\app-id.txt", module_path);
    FILE *f = fopen(id_file, "r");
    if (!f) return 1;

    char app_id[512] = {0};
    if (!fgets(app_id, sizeof(app_id), f)) {
        fclose(f);
        return 1;
    }
    fclose(f);
    char *nl = strchr(app_id, '\n');
    if (nl) *nl = '\0';

    char runtime_path[MAX_PATH] = "C:\\stardesk-launcher.runtime";
    f = fopen(runtime_path, "r");
    if (!f) return 1;

    int port = 0;
    char token[128] = {0};
    char line[256];
    while (fgets(line, sizeof(line), f)) {
        if (strncmp(line, "port=", 5) == 0) port = atoi(line + 5);
        if (strncmp(line, "token=", 6) == 0) {
            strncpy(token, line + 6, sizeof(token) - 1);
            char *p = strchr(token, '\n');
            if (p) *p = '\0';
        }
    }
    fclose(f);
    if (port <= 0 || token[0] == '\0') return 1;

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

    uint8_t header[12];
    uint32_t magic = htonl(STARDESK_LAUNCH_MAGIC);
    uint16_t version = htons(1);
    uint16_t token_len = htons((uint16_t)strlen(token));
    uint16_t id_len = htons((uint16_t)strlen(app_id));

    memcpy(header, &magic, 4);
    memcpy(header + 4, &version, 2);
    memcpy(header + 6, &token_len, 2);
    memcpy(header + 8, &id_len, 2);
    memset(header + 10, 0, 2);

    send(s, (char *)header, 12, 0);
    send(s, token, (int)strlen(token), 0);
    send(s, app_id, (int)strlen(app_id), 0);

    uint8_t ack = 0;
    recv(s, (char *)&ack, 1, 0);
    closesocket(s);
    WSACleanup();
    return ack ? 0 : 1;
}
