#ifndef STARDESK_FRAME_PROTOCOL_H
#define STARDESK_FRAME_PROTOCOL_H

#include <stdint.h>

#define STARDESK_FRAME_MAGIC 0x53544152u /* "STAR" */
#define STARDESK_FRAME_VERSION 1u

#define STARDESK_FORMAT_BGRA32 1u
#define STARDESK_FORMAT_RGBA32 2u
#define STARDESK_FORMAT_NV12   3u

#pragma pack(push, 1)

typedef struct stardesk_frame_header {
    uint32_t magic;         /* STARDESK_FRAME_MAGIC */
    uint32_t version;       /* STARDESK_FRAME_VERSION */
    uint32_t sequence;      /* Incremental frame counter */
    uint32_t width;         /* Frame width in pixels */
    uint32_t height;        /* Frame height in pixels */
    uint32_t stride;        /* Bytes per scanline */
    uint32_t format;        /* Pixel format */
    uint64_t timestamp_us;  /* Capture timestamp in microseconds */
    uint32_t payload_bytes; /* Total payload byte length */
    uint32_t cursor_x;      /* Virtual cursor X */
    uint32_t cursor_y;      /* Virtual cursor Y */
    uint8_t  cursor_visible;/* Is cursor currently visible */
    uint8_t  reserved[15];  /* Reserved for alignment */
} stardesk_frame_header_t;

#pragma pack(pop)

#endif /* STARDESK_FRAME_PROTOCOL_H */
