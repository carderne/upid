import math
import os
from collections.abc import Callable
from typing import Any

import pytest

from upid import b32


def test_decode_table() -> None:
    encode_bytes = bytes(b32.ENCODE, "ascii")
    table: list[int] = []
    for i in range(130):
        b = i.to_bytes(length=1, byteorder="big")
        try:
            idx = encode_bytes.index(b)
        except ValueError:
            idx = 255
        table.append(idx)
    assert table == b32.DECODE


def test_proba() -> None:
    k = 3e9  # number of tries
    n = 64  # bits of randomness
    proba = 1 - math.exp((-(k**2)) / (2**n + 1))
    assert proba < 0.5


@pytest.mark.parametrize(
    ("func", "value"),
    [
        (b32.encode, os.urandom(b32.BIN_LEN - 1)),
        (b32.encode, os.urandom(b32.BIN_LEN + 1)),
        (b32.encode_time, os.urandom(b32.TIME_BIN_LEN - 1)),
        (b32.encode_time, os.urandom(b32.TIME_BIN_LEN + 1)),
        (b32.encode_rando, os.urandom(b32.RANDO_BIN_LEN - 1)),
        (b32.encode_rando, os.urandom(b32.RANDO_BIN_LEN + 1)),
        (b32.encode_prefix, os.urandom(b32.PREFIX_BIN_LEN - 1)),
        (b32.encode_prefix, os.urandom(b32.PREFIX_BIN_LEN + 1)),
    ],
)
def test_bad_encode_input(func: Callable[[bytes], str], value: Any) -> None:
    with pytest.raises(ValueError):
        func(value)


@pytest.mark.parametrize(
    ("func", "value"),
    [
        (b32.decode, "a" * (b32.CHAR_LEN - 1)),
        (b32.decode, "a" * (b32.CHAR_LEN + 1)),
        (b32.decode_time, "a" * (b32.TIME_CHAR_LEN - 1)),
        (b32.decode_time, "a" * (b32.TIME_CHAR_LEN + 1)),
        (b32.decode_rando, "a" * (b32.RANDO_CHAR_LEN - 1)),
        (b32.decode_rando, "a" * (b32.RANDO_CHAR_LEN + 1)),
        (b32.decode_rando, "z" * (b32.RANDO_CHAR_LEN)),
        (b32.decode_prefix, "a" * (b32.PREFIX_CHAR_LEN - 1)),
        (b32.decode_prefix, "a" * (b32.PREFIX_CHAR_LEN + 2)),  # version char goes with
        (b32.decode_prefix, "z" * (b32.PREFIX_CHAR_LEN)),
    ],
)
def test_bad_decode_input(func: Callable[[str], bytes], value: Any) -> None:
    with pytest.raises(ValueError):
        func(value)
