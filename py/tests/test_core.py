import datetime as dt
import time

import pytest

from upid import UPID, b32, consts, upid

TS_EPS = 256  # compare with 256ms (the lost byte) of precision


def test_roundtrip() -> None:
    want = upid("user")
    got = UPID.from_str(str(want))
    assert got == want
    assert str(got) == str(want)


def test_no_overflow_roundtrip() -> None:
    # This is the max string before roundtrip fails due to overflow
    # If either of the last two characters goes past j they will wrap
    # (But they will be caught by error checks in the decoder)
    want = "zzzz_zzzzzzzzzzzzzzzzzzzzjj"
    assert len(want.replace("_", "")) == b32.CHAR_LEN
    got = str(UPID.from_str(want))
    assert got == want


def test_prefix_roundtrip() -> None:
    want = "zzzz"
    got = UPID.from_prefix(want).prefix
    assert got == want


def test_timestamp_roundtrip() -> None:
    milliseconds = time.time_ns() // consts.NS_PER_MS
    a = UPID.from_prefix_and_milliseconds("user", milliseconds)
    assert abs(a.milliseconds - milliseconds) < TS_EPS


@pytest.mark.skip()
def check_timestamp_precision() -> None:
    base = time.time_ns() // consts.NS_PER_MS
    for i in range(0, 260):
        ts = base + i
        b = ts.to_bytes(6, "big")
        b_cut = b[:5]
        ts_after = int.from_bytes(b_cut + b"\x00", "big")
        assert ts - ts_after < TS_EPS


def test_datetime_roundtrip() -> None:
    want = dt.datetime.now(dt.timezone.utc)
    a = UPID.from_prefix_and_datetime("user", want)
    got = a.datetime
    diff = want - got
    assert diff.total_seconds() * consts.MS_PER_SEC < TS_EPS
