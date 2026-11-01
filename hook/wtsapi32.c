/*
 * wtsapi32.c - Stubs for Windows Terminal Services API
 */

#include <windows.h>
#include <wtsapi32.h>

BOOL WINAPI DllMain(HINSTANCE hinst, DWORD reason, LPVOID reserved)
{
    (void)hinst; (void)reason; (void)reserved;
    return TRUE;
}

BOOL WINAPI WTSEnumerateSessionsW(HANDLE hServer, DWORD Reserved, DWORD Version, PWTS_SESSION_INFOW *ppSessionInfo, DWORD *pCount)
{
    (void)hServer; (void)Reserved; (void)Version;
    if (ppSessionInfo) *ppSessionInfo = NULL;
    if (pCount) *pCount = 0;
    return TRUE;
}

void WINAPI WTSFreeMemory(PVOID pMemory)
{
    if (pMemory) LocalFree(pMemory);
}

BOOL WINAPI WTSQuerySessionInformationW(HANDLE hServer, DWORD SessionId, WTS_INFO_CLASS WTSInfoClass, LPWSTR *ppBuffer, DWORD *pBytesReturned)
{
    (void)hServer; (void)SessionId; (void)WTSInfoClass;
    if (ppBuffer) *ppBuffer = NULL;
    if (pBytesReturned) *pBytesReturned = 0;
    return TRUE;
}
