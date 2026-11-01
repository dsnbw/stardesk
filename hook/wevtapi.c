/*
 * wevtapi.c - Proxy DLL for Windows Event Log API to preload stardesk-hook.dll
 */

#include <windows.h>

/* Reference marker symbol from stardesk-hook.dll */
__declspec(dllimport) void stardesk_hook_version(void);

BOOL WINAPI DllMain(HINSTANCE hinst, DWORD reason, LPVOID reserved)
{
    (void)hinst;
    (void)reserved;
    if (reason == DLL_PROCESS_ATTACH) {
        stardesk_hook_version();
    }
    return TRUE;
}

/* Stubs for statically linked wevtapi functions */
EVT_HANDLE WINAPI EvtOpenPublisherMetadata(EVT_HANDLE session, LPCWSTR path, LPCWSTR file, LCID lcid, DWORD flags)
{
    (void)session; (void)path; (void)file; (void)lcid; (void)flags;
    SetLastError(ERROR_NOT_SUPPORTED);
    return NULL;
}

BOOL WINAPI EvtClose(EVT_HANDLE handle)
{
    (void)handle;
    return TRUE;
}

EVT_HANDLE WINAPI EvtQuery(EVT_HANDLE session, LPCWSTR path, LPCWSTR query, DWORD flags)
{
    (void)session; (void)path; (void)query; (void)flags;
    SetLastError(ERROR_NOT_SUPPORTED);
    return NULL;
}

BOOL WINAPI EvtNext(EVT_HANDLE result, DWORD size, EVT_HANDLE *events, DWORD timeout, DWORD flags, DWORD *returned)
{
    (void)result; (void)size; (void)events; (void)timeout; (void)flags;
    if (returned) *returned = 0;
    SetLastError(ERROR_NO_MORE_ITEMS);
    return FALSE;
}

BOOL WINAPI EvtRender(EVT_HANDLE context, EVT_HANDLE fragment, DWORD flags, DWORD buf_size, PVOID buffer, DWORD *used, DWORD *prop_count)
{
    (void)context; (void)fragment; (void)flags; (void)buf_size; (void)buffer;
    if (used) *used = 0;
    if (prop_count) *prop_count = 0;
    SetLastError(ERROR_NOT_SUPPORTED);
    return FALSE;
}
