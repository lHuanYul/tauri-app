void u16_to_u8s_be(uint16_t x, uint8_t bytes[2]) {
    bytes[0] = (uint8_t)(x >> 8);
    bytes[1] = (uint8_t)(x & 0xFF);
}

uint16_t u8s_to_u16_be(const uint8_t bytes[2]) {
    return (uint16_t)bytes[0] << 8
         | (uint16_t)bytes[1];
}

void u32_to_u8s_be(uint32_t x, uint8_t bytes[4]) {
    bytes[0] = (uint8_t)(x >> 24);
    bytes[1] = (uint8_t)((x >> 16) & 0xFF);
    bytes[2] = (uint8_t)((x >>  8) & 0xFF);
    bytes[3] = (uint8_t)( x        & 0xFF);
}

uint32_t u8s_to_u32_be(const uint8_t bytes[4]) {
    return (uint32_t)bytes[0] << 24
         | (uint32_t)bytes[1] << 16
         | (uint32_t)bytes[2] <<  8
         | (uint32_t)bytes[3];
}
