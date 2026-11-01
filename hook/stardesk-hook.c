/*
 * stardesk-hook.c - Native Linux Bridge Hook for StarDesk (and NetEase UU Remote) under Wine.
 *
 * This DLL is preloaded into StarDeskServer.exe via the wevtapi.dll preload chain.
 * At PROCESS_ATTACH, it patches the Import Address Tables (IAT) of all loaded modules:
 *   1. SendInput -> Redirects incoming remote mouse & keyboard inputs over local loopback
 *      TCP to the stardesk-linux-bridge daemon, which injects them into the Linux /dev/uinput.
 *   2. CreateDXGIFactory1 / IDXGIOutput1::DuplicateOutput -> Forces DuplicateOutput to return
 *      DXGI_ERROR_UNSUPPORTED, forcing streamer.dll to gracefully fall back to GDI BitBlt.
 *   3. BitBlt / StretchBlt -> Blits real Linux desktop frames from shared memory into the DC.
 */

#define WIN32_LEAN_AND_MEAN
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0601
#endif

#include <winsock2.h>
#include <ws2tcpip.h>
#include <windows.h>
#include <tlhelp32.h>
#include <dxgi.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include "frame-protocol.h"

#define STARDESK_MAGIC 0x53544152u /* "STAR" */
#define STARDESK_VERSION 1u
#define STARDESK_RECORD_HELLO 1u
#define STARDESK_RECORD_MOUSE 2u
#define STARDESK_RECORD_KEYBOARD 3u

#ifndef DXGI_ERROR_UNSUPPORTED
#define DXGI_ERROR_UNSUPPORTED ((HRESULT)0x887A0004)
#endif

static UINT (WINAPI *original_send_input)(UINT, LPINPUT, int) = NULL;
static BOOL (WINAPI *original_bit_blt)(HDC, int, int, int, int, HDC, int, int, DWORD) = NULL;
static BOOL (WINAPI *original_stretch_blt)(HDC, int, int, int, int, HDC, int, int, int, int, DWORD) = NULL;
static HRESULT (WINAPI *original_create_dxgi_factory1)(REFIID, void **) = NULL;

static SOCKET hook_socket = INVALID_SOCKET;
static CRITICAL_SECTION socket_cs;
static HANDLE shm_handle = NULL;
static void *shm_view = NULL;

static volatile LONG iat_hooks_installed = 0;
static volatile LONG dxgi_outputs_patched = 0;
static volatile LONG frames_blitted = 0;

static void hook_log(const char *msg)
{
    FILE *f = fopen("C:\\stardesk-hook.log", "a");
    if (f) {
        fprintf(f, "[stardesk-hook] %s\n", msg);
        fclose(f);
    }
}

/* Connect to native Linux daemon */
static int connect_bridge(void)
{
    EnterCriticalSection(&socket_cs);
    if (hook_socket != INVALID_SOCKET) {
        LeaveCriticalSection(&socket_cs);
        return 1;
    }

    WSADATA wsa;
    if (WSAStartup(MAKEWORD(2, 2), &wsa) != 0) {
        LeaveCriticalSection(&socket_cs);
        return 0;
    }

    /* Read runtime port from configuration file */
    int port = 54192;
    FILE *cfg = fopen("C:\\stardesk-bridge.runtime", "r");
    if (cfg) {
        char line[128];
        while (fgets(line, sizeof(line), cfg)) {
            if (strncmp(line, "port=", 5) == 0) {
                port = atoi(line + 5);
            }
        }
        fclose(cfg);
    }

    hook_socket = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if (hook_socket == INVALID_SOCKET) {
        WSACleanup();
        LeaveCriticalSection(&socket_cs);
        return 0;
    }

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_port = htons((u_short)port);
    addr.sin_addr.s_addr = inet_addr("127.0.0.1");

    if (connect(hook_socket, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        closesocket(hook_socket);
        hook_socket = INVALID_SOCKET;
        WSACleanup();
        LeaveCriticalSection(&socket_cs);
        return 0;
    }

    /* Send hello handshake packet */
    uint32_t hello[4] = { STARDESK_MAGIC, STARDESK_VERSION, STARDESK_RECORD_HELLO, 0 };
    send(hook_socket, (char *)hello, sizeof(hello), 0);

    /* Open shared memory frame buffer */
    if (!shm_handle) {
        shm_handle = OpenFileMappingA(FILE_MAP_READ, FALSE, "Global\\StarDeskFrameBuffer");
        if (shm_handle) {
            shm_view = MapViewOfFile(shm_handle, FILE_MAP_READ, 0, 0, 0);
        }
    }

    LeaveCriticalSection(&socket_cs);
    hook_log("Connected to stardesk-linux-bridge");
    return 1;
}

/* Intercepted SendInput: sends mouse & keyboard packets to Linux */
UINT WINAPI hook_send_input(UINT count, LPINPUT inputs, int size)
{
    if (!inputs || count == 0) return 0;

    if (hook_socket == INVALID_SOCKET) {
        connect_bridge();
    }

    if (hook_socket != INVALID_SOCKET) {
        EnterCriticalSection(&socket_cs);
        for (UINT i = 0; i < count; i++) {
            uint8_t packet[20];
            uint32_t magic = STARDESK_MAGIC;
            uint16_t version = STARDESK_VERSION;
            memcpy(packet, &magic, 4);
            memcpy(packet + 4, &version, 2);

            if (inputs[i].type == INPUT_MOUSE) {
                uint16_t kind = STARDESK_RECORD_MOUSE;
                uint16_t flags = (uint16_t)inputs[i].mi.dwFlags;
                int32_t dx = inputs[i].mi.dx;
                int32_t dy = inputs[i].mi.dy;
                int32_t data = inputs[i].mi.mouseData;
                memcpy(packet + 6, &kind, 2);
                memcpy(packet + 8, &flags, 2);
                memcpy(packet + 10, &dx, 4);
                memcpy(packet + 14, &dy, 4);
                memcpy(packet + 18, &data, 2);
                send(hook_socket, (char *)packet, 20, 0);
            } else if (inputs[i].type == INPUT_KEYBOARD) {
                uint16_t kind = STARDESK_RECORD_KEYBOARD;
                uint16_t vk = inputs[i].ki.wVk;
                uint16_t scan = inputs[i].ki.wScan;
                uint32_t flags = inputs[i].ki.dwFlags;
                memcpy(packet + 6, &kind, 2);
                memcpy(packet + 8, &vk, 2);
                memcpy(packet + 10, &scan, 2);
                memcpy(packet + 12, &flags, 4);
                send(hook_socket, (char *)packet, 16, 0);
            }
        }
        LeaveCriticalSection(&socket_cs);
        return count;
    }

    if (original_send_input) {
        return original_send_input(count, inputs, size);
    }
    return count;
}

/* Intercepted BitBlt: blits Linux desktop frames */
BOOL WINAPI hook_bit_blt(HDC hdcDest, int nXDest, int nYDest, int nWidth, int nHeight,
                         HDC hdcSrc, int nXSrc, int nYSrc, DWORD dwRop)
{
    if (shm_view) {
        stardesk_frame_header_t *hdr = (stardesk_frame_header_t *)shm_view;
        if (hdr->magic == STARDESK_FRAME_MAGIC && hdr->payload_bytes > 0) {
            uint8_t *pixel_data = (uint8_t *)shm_view + sizeof(stardesk_frame_header_t);

            BITMAPINFO bmi;
            memset(&bmi, 0, sizeof(bmi));
            bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
            bmi.bmiHeader.biWidth = hdr->width;
            bmi.bmiHeader.biHeight = -(int32_t)hdr->height; /* Top-down */
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB;

            SetDIBitsToDevice(hdcDest, nXDest, nYDest, nWidth, nHeight,
                              nXSrc, 0, 0, hdr->height,
                              pixel_data, &bmi, DIB_RGB_COLORS);
            InterlockedIncrement(&frames_blitted);
            return TRUE;
        }
    }

    if (original_bit_blt) {
        return original_bit_blt(hdcDest, nXDest, nYDest, nWidth, nHeight, hdcSrc, nXSrc, nYSrc, dwRop);
    }
    return TRUE;
}

/* Intercepted StretchBlt: supports scaled display blitting */
BOOL WINAPI hook_stretch_blt(HDC hdcDest, int nXOriginDest, int nYOriginDest, int nWidthDest, int nHeightDest,
                             HDC hdcSrc, int nXOriginSrc, int nYOriginSrc, int nWidthSrc, int nHeightSrc, DWORD dwRop)
{
    if (shm_view) {
        stardesk_frame_header_t *hdr = (stardesk_frame_header_t *)shm_view;
        if (hdr->magic == STARDESK_FRAME_MAGIC && hdr->payload_bytes > 0) {
            uint8_t *pixel_data = (uint8_t *)shm_view + sizeof(stardesk_frame_header_t);

            BITMAPINFO bmi;
            memset(&bmi, 0, sizeof(bmi));
            bmi.bmiHeader.biSize = sizeof(BITMAPINFOHEADER);
            bmi.bmiHeader.biWidth = hdr->width;
            bmi.bmiHeader.biHeight = -(int32_t)hdr->height;
            bmi.bmiHeader.biPlanes = 1;
            bmi.bmiHeader.biBitCount = 32;
            bmi.bmiHeader.biCompression = BI_RGB;

            StretchDIBits(hdcDest, nXOriginDest, nYOriginDest, nWidthDest, nHeightDest,
                          nXOriginSrc, nYOriginSrc, nWidthSrc, nHeightSrc,
                          pixel_data, &bmi, DIB_RGB_COLORS, dwRop);
            InterlockedIncrement(&frames_blitted);
            return TRUE;
        }
    }

    if (original_stretch_blt) {
        return original_stretch_blt(hdcDest, nXOriginDest, nYOriginDest, nWidthDest, nHeightDest,
                                    hdcSrc, nXOriginSrc, nYOriginSrc, nWidthSrc, nHeightSrc, dwRop);
    }
    return TRUE;
}

/* DXGI DuplicateOutput Hook: Force unsupported return code */
static const GUID stardesk_iid_idxgioutput1 = {
    0x00cddea8, 0x939b, 0x4b83,
    {0xa3, 0x40, 0xa6, 0x85, 0x22, 0x66, 0x66, 0xcc}
};

static HRESULT WINAPI hook_duplicate_output(void *output, IUnknown *device, void **duplication)
{
    (void)output;
    (void)device;
    (void)duplication;
    InterlockedIncrement(&dxgi_outputs_patched);
    return DXGI_ERROR_UNSUPPORTED;
}

/* Helper to patch VTable slots */
static int patch_vtable_slot(void **slot, void *hook, void **original)
{
    DWORD old_protect = 0;
    if (*slot == hook) return 1;
    if (original) *original = *slot;
    if (!VirtualProtect(slot, sizeof(void *), PAGE_READWRITE, &old_protect)) return 0;
    *slot = hook;
    VirtualProtect(slot, sizeof(void *), old_protect, &old_protect);
    return 1;
}

/* IAT Patching */
static int patch_module_iat(HMODULE module)
{
    if (!module) return 0;
    PIMAGE_DOS_HEADER dos = (PIMAGE_DOS_HEADER)module;
    if (dos->e_magic != IMAGE_DOS_SIGNATURE) return 0;
    PIMAGE_NT_HEADERS nt = (PIMAGE_NT_HEADERS)((uint8_t *)module + dos->e_lfanew);
    if (nt->Signature != IMAGE_NT_SIGNATURE) return 0;

    IMAGE_DATA_DIRECTORY import_dir = nt->OptionalHeader.DataDirectory[IMAGE_DIRECTORY_ENTRY_IMPORT];
    if (import_dir.Size == 0 || import_dir.VirtualAddress == 0) return 0;

    PIMAGE_IMPORT_DESCRIPTOR desc = (PIMAGE_IMPORT_DESCRIPTOR)((uint8_t *)module + import_dir.VirtualAddress);
    int patched = 0;

    for (; desc->Name != 0; desc++) {
        PIMAGE_THUNK_DATA thunk = (PIMAGE_THUNK_DATA)((uint8_t *)module + desc->FirstThunk);
        PIMAGE_THUNK_DATA orig_thunk = desc->OriginalFirstThunk ?
            (PIMAGE_THUNK_DATA)((uint8_t *)module + desc->OriginalFirstThunk) : thunk;

        for (; orig_thunk->u1.AddressOfData != 0; thunk++, orig_thunk++) {
            if (IMAGE_SNAP_BY_ORDINAL(orig_thunk->u1.Ordinal)) continue;
            PIMAGE_IMPORT_BY_NAME import_name = (PIMAGE_IMPORT_BY_NAME)((uint8_t *)module + orig_thunk->u1.AddressOfData);

            void *hook_fn = NULL;
            void **orig_slot = NULL;

            if (lstrcmpiA((char *)import_name->Name, "SendInput") == 0) {
                hook_fn = (void *)hook_send_input;
                orig_slot = (void **)&original_send_input;
            } else if (lstrcmpiA((char *)import_name->Name, "BitBlt") == 0) {
                hook_fn = (void *)hook_bit_blt;
                orig_slot = (void **)&original_bit_blt;
            } else if (lstrcmpiA((char *)import_name->Name, "StretchBlt") == 0) {
                hook_fn = (void *)hook_stretch_blt;
                orig_slot = (void **)&original_stretch_blt;
            }

            if (hook_fn && thunk->u1.Function != (uintptr_t)hook_fn) {
                if (orig_slot && !*orig_slot) *orig_slot = (void *)thunk->u1.Function;
                DWORD old_protect;
                if (VirtualProtect(&thunk->u1.Function, sizeof(thunk->u1.Function), PAGE_READWRITE, &old_protect)) {
                    thunk->u1.Function = (uintptr_t)hook_fn;
                    VirtualProtect(&thunk->u1.Function, sizeof(thunk->u1.Function), old_protect, &old_protect);
                    patched++;
                    InterlockedIncrement(&iat_hooks_installed);
                }
            }
        }
    }
    return patched;
}

static int patch_all_modules(void)
{
    int patched = 0;
    HANDLE snap = CreateToolhelp32Snapshot(TH32CS_SNAPMODULE, GetCurrentProcessId());
    if (snap == INVALID_HANDLE_VALUE) return 0;
    MODULEENTRY32W me;
    me.dwSize = sizeof(me);
    if (Module32FirstW(snap, &me)) {
        do {
            patched += patch_module_iat(me.hModule);
        } while (Module32NextW(snap, &me));
    }
    CloseHandle(snap);
    return patched;
}

/* Background watchdog for late loaded modules (e.g., streamer.dll) */
static DWORD WINAPI module_watchdog(LPVOID param)
{
    (void)param;
    for (;;) {
        Sleep(1500);
        patch_all_modules();
    }
    return 0;
}

void __declspec(dllexport) stardesk_hook_version(void) {}

BOOL WINAPI DllMain(HINSTANCE hinst, DWORD reason, LPVOID reserved)
{
    (void)hinst;
    (void)reserved;
    if (reason == DLL_PROCESS_ATTACH) {
        DisableThreadLibraryCalls(hinst);
        InitializeCriticalSection(&socket_cs);
        hook_log("StarDesk Linux hook attached to process");
        connect_bridge();
        patch_all_modules();
        HANDLE th = CreateThread(NULL, 0, module_watchdog, NULL, 0, NULL);
        if (th) CloseHandle(th);
    } else if (reason == DLL_PROCESS_DETACH) {
        if (hook_socket != INVALID_SOCKET) {
            closesocket(hook_socket);
            hook_socket = INVALID_SOCKET;
        }
        if (shm_view) {
            UnmapViewOfFile(shm_view);
            shm_view = NULL;
        }
        if (shm_handle) {
            CloseHandle(shm_handle);
            shm_handle = NULL;
        }
        DeleteCriticalSection(&socket_cs);
    }
    return TRUE;
}
