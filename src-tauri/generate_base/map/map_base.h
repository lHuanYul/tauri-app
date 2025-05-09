#ifndef PRINCIPAL_MAP_BASE_H
#define PRINCIPAL_MAP_BASE_H

#include <stdint.h>

#define MAX_CONNECTIONS 8
typedef struct {
    uint16_t locate;
    uint32_t length;
} CONNECT;
typedef struct LOCATION {
    uint16_t id;
    CONNECT connect[MAX_CONNECTIONS];
} LOCATION;

#endif
