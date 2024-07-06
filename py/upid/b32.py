"""
Base32 encoding modified from
https://github.com/mdomke/python-ulid

In turn adapted from
https://github.com/RobThree/NUlid
"""

# Note the binary order is TIMESTAMP_RANDO_PREFIX+VERSION
TIME_BIN_LEN = 5
RANDO_BIN_LEN = 8
END_RANDO_BIN = TIME_BIN_LEN + RANDO_BIN_LEN
PREFIX_BIN_LEN = 3  # includes version
BIN_LEN = TIME_BIN_LEN + RANDO_BIN_LEN + PREFIX_BIN_LEN

# But the string order is PREFIX_TIME_RANDO_VERSION
PREFIX_CHAR_LEN = 4  # excluding the version char
TIME_CHAR_LEN = 8
END_TIME_CHAR = PREFIX_CHAR_LEN + TIME_CHAR_LEN
RANDO_CHAR_LEN = 13
VERSION_CHAR_LEN = 1
CHAR_LEN = PREFIX_CHAR_LEN + TIME_CHAR_LEN + RANDO_CHAR_LEN + VERSION_CHAR_LEN

# 32-character alphabet modified from Crockford's
# Numbers first for sensible sorting, but full lower-case
# latin alphabet so any sensible prefix can be used
# Effectively a mapping from 8 bit byte -> 5 bit int -> base32 character
ENCODE = "234567abcdefghijklmnopqrstuvwxyz"

# Speedy O(1) inverse lookup
# base32 char -> ascii byte int -> base32 alphabet index
# fmt: off
DECODE = [
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
      0,   1,   2,   3,   4,   5, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255, 255, 255, 255,
    255, 255, 255, 255, 255, 255, 255,   6,   7,   8,
      9,  10,  11,  12,  13,  14,  15,  16,  17,  18,
     19,  20,  21,  22,  23,  24,  25,  26,  27,  28,
     29,  30,  31, 255, 255, 255, 255, 255, 255, 255,
 ]
# fmt: on


def encode(binary: bytes) -> str:
    if len(binary) != BIN_LEN:
        raise ValueError(f"UPID has to be exactly {BIN_LEN} bytes long")

    time = encode_time(binary[:TIME_BIN_LEN])
    rando = encode_rando(binary[TIME_BIN_LEN:END_RANDO_BIN])
    prefix, version = encode_prefix(binary[END_RANDO_BIN:])
    return prefix + "_" + time + rando + version


def encode_prefix(binary: bytes) -> tuple[str, str]:
    if len(binary) != PREFIX_BIN_LEN:
        raise ValueError(f"Prefix value has to be exactly {PREFIX_BIN_LEN} bytes long.")
    lut = ENCODE
    return "".join(
        [
            lut[(binary[0] & 248) >> 3],
            lut[((binary[0] & 7) << 2) | ((binary[1] & 192) >> 6)],
            lut[(binary[1] & 62) >> 1],
            lut[((binary[1] & 1) << 4) | ((binary[2] & 240) >> 4)],
        ]
    ), lut[(binary[2] & 15)]  # implicitly "add" a 0 bit


def encode_time(binary: bytes) -> str:
    if len(binary) != TIME_BIN_LEN:
        raise ValueError(f"Timestamp value has to be exactly {TIME_BIN_LEN} bytes long.")
    lut = ENCODE
    return "".join(
        [
            lut[(binary[0] & 248) >> 3],
            lut[((binary[0] & 7) << 2) | ((binary[1] & 192) >> 6)],
            lut[(binary[1] & 62) >> 1],
            lut[((binary[1] & 1) << 4) | ((binary[2] & 240) >> 4)],
            lut[((binary[2] & 15) << 1) | ((binary[3] & 128) >> 7)],
            lut[(binary[3] & 124) >> 2],
            lut[((binary[3] & 3) << 3) | ((binary[4] & 224) >> 5)],
            lut[(binary[4] & 31)],
        ]
    )


def encode_rando(binary: bytes) -> str:
    if len(binary) != RANDO_BIN_LEN:
        raise ValueError(f"Randomness value has to be exactly {RANDO_BIN_LEN} bytes long.")
    lut = ENCODE
    return "".join(
        [
            lut[(binary[0] & 248) >> 3],
            lut[((binary[0] & 7) << 2) | ((binary[1] & 192) >> 6)],
            lut[(binary[1] & 62) >> 1],
            lut[((binary[1] & 1) << 4) | ((binary[2] & 240) >> 4)],
            lut[((binary[2] & 15) << 1) | ((binary[3] & 128) >> 7)],
            lut[(binary[3] & 124) >> 2],
            lut[((binary[3] & 3) << 3) | ((binary[4] & 224) >> 5)],
            lut[(binary[4] & 31)],
            lut[(binary[5] & 248) >> 3],
            lut[((binary[5] & 7) << 2) | ((binary[6] & 192) >> 6)],
            lut[(binary[6] & 62) >> 1],
            lut[((binary[6] & 1) << 4) | ((binary[7] & 240) >> 4)],
            lut[(binary[7] & 15)],  # implicitly "add" a 0 bit
        ]
    )


def decode(encoded: str) -> bytes:
    encoded = encoded.replace("_", "")
    if len(encoded) != CHAR_LEN:
        raise ValueError(f"Encoded UPID has to be exactly {CHAR_LEN} characters long.")
    if any((c not in ENCODE) for c in encoded):
        raise ValueError(f"Encoded UPID can only consist of letters in {ENCODE}.")

    prefix = decode_prefix(encoded[:PREFIX_CHAR_LEN] + encoded[-1])
    time = decode_time(encoded[PREFIX_CHAR_LEN:END_TIME_CHAR])
    rando = decode_rando(encoded[END_TIME_CHAR:-1])
    return time + rando + prefix


def decode_prefix(encoded: str) -> bytes:
    if len(encoded) != PREFIX_CHAR_LEN + VERSION_CHAR_LEN:
        raise ValueError(f"UPID prefix has to be exactly {PREFIX_CHAR_LEN} characters long.")
    lut = DECODE
    values = bytes(encoded, "ascii")
    if lut[values[-1]] > 15:
        raise ValueError(f"Prefix value {encoded} is too large and will overflow 128-bits.")
    return bytes(
        [
            ((lut[values[0]] << 3) | (lut[values[1]] >> 2)) & 0xFF,
            ((lut[values[1]] << 6) | (lut[values[2]] << 1) | (lut[values[3]] >> 4)) & 0xFF,
            ((lut[values[3]] << 4) | (lut[values[4]] & 15)) & 0xFF,
            # lose 1 bit of data
        ]
    )


def decode_time(encoded: str) -> bytes:
    if len(encoded) != TIME_CHAR_LEN:
        raise ValueError(f"UPID timestamp has to be exactly {TIME_CHAR_LEN} characters long.")
    lut = DECODE
    values: bytes = bytes(encoded, "ascii")
    return bytes(
        [
            ((lut[values[0]] << 3) | (lut[values[1]] >> 2)) & 0xFF,
            ((lut[values[1]] << 6) | (lut[values[2]] << 1) | (lut[values[3]] >> 4)) & 0xFF,
            ((lut[values[3]] << 4) | (lut[values[4]] >> 1)) & 0xFF,
            ((lut[values[4]] << 7) | (lut[values[5]] << 2) | (lut[values[6]] >> 3)) & 0xFF,
            ((lut[values[6]] << 5) | (lut[values[7]])) & 0xFF,
        ]
    )


def decode_rando(encoded: str) -> bytes:
    if len(encoded) != RANDO_CHAR_LEN:
        raise ValueError(f"UPID randomness has to be exactly {RANDO_CHAR_LEN} characters long.")
    lut = DECODE
    values = bytes(encoded, "ascii")
    if lut[values[-1]] > 15:
        raise ValueError(f"Random value {encoded} is too large and will overflow 128-bits.")
    return bytes(
        [
            ((lut[values[0]] << 3) | (lut[values[1]] >> 2)) & 0xFF,
            ((lut[values[1]] << 6) | (lut[values[2]] << 1) | (lut[values[3]] >> 4)) & 0xFF,
            ((lut[values[3]] << 4) | (lut[values[4]] >> 1)) & 0xFF,
            ((lut[values[4]] << 7) | (lut[values[5]] << 2) | (lut[values[6]] >> 3)) & 0xFF,
            ((lut[values[6]] << 5) | (lut[values[7]])) & 0xFF,
            ((lut[values[8]] << 3) | (lut[values[9]] >> 2)) & 0xFF,
            ((lut[values[9]] << 6) | (lut[values[10]] << 1) | (lut[values[11]] >> 4)) & 0xFF,
            ((lut[values[11]] << 4) | (lut[values[12]] & 15)) & 0xFF,
            # lose 1 bit of data
        ]
    )
