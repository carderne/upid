import datetime as dt
import functools
import os
import time
import uuid
from typing import TypeVar

from upid import b32, consts

Self = TypeVar("Self", bound="UPID")

# version is 4 bits, restricted to first half of base32 alphabet
# first version is "a" purely for aesthetics
# this is not expected to change, but could allow for future
# versions with different timestamp/randomness configuration
VERSION = "a"


@functools.total_ordering
class UPID:
    """
    The `UPID` contains a 20-bit prefix, 40-bit timestamp and 68 bits of randomness.

    It is usually created using the `upid(prefix: str)` helper function:

        upid("user")  # UPID(user_3accvpp5_guht4dts56je5w)

    Note that helper methods to convert to hex/UUID/str/int are provided,
    but not always the inverse, as this may be lossy or meaningless.
    """

    b: bytes

    def __init__(self, b: bytes):
        """Not normally used directly."""
        self.b = b

    @classmethod
    def from_prefix(cls: type[Self], prefix: str) -> Self:
        """Create a new `UPID` from a `prefix`, using the current datetime."""
        milliseconds = time.time_ns() // consts.NS_PER_MS
        return cls.from_prefix_and_milliseconds(prefix, milliseconds)

    @classmethod
    def from_prefix_and_datetime(cls: type[Self], prefix: str, datetime: dt.datetime) -> Self:
        """Create a new `UPID` from a `prefix`, using the supplied `datetime`."""
        milliseconds = int(datetime.timestamp() * consts.MS_PER_SEC)
        return cls.from_prefix_and_milliseconds(prefix, milliseconds)

    @classmethod
    def from_prefix_and_milliseconds(cls: type[Self], prefix: str, milliseconds: int) -> Self:
        """
        Create a new `UPID` from a `prefix`, using the supplied `timestamp`.

        `milliseconds` must be an int in milliseconds since the epoch.

        The timestamp is converted to 6 bytes, but we drop 1 byte, resulting
        in a time precision of about 100 milliseconds

        The prefix is padded with 'z' characters (if too short) and
        trimmed to 4 characters (if too long). Supply a prefix of exactly
        4 characters if this isn't appealing!
        """
        # we drop one byte of millisecond time information
        time_bin = int.to_bytes((milliseconds >> 8), b32.TIME_BIN_LEN + 0, "big")

        # 8 bytes/64 bits of randomness
        rando_bin = os.urandom(b32.RANDO_BIN_LEN)

        # note the padding and trimming
        prefix_clean = prefix.ljust(4, "z")[:4]
        prefix_bin = b32.decode_prefix(prefix_clean + VERSION)

        # note the binary ordering
        return cls(time_bin + rando_bin + prefix_bin)

    @classmethod
    def from_str(cls: type[Self], string: str) -> Self:
        return cls(b32.decode(string))

    @property
    def prefix(self) -> str:
        prefix, _ = b32.encode_prefix(self.b[b32.END_RANDO_BIN :])
        return prefix

    @property
    def milliseconds(self) -> int:
        """Returns a time in integer milliseconds since the epoch."""
        # must add back the deleted byte to get a sensible timestamp
        return int.from_bytes(self.b[: b32.TIME_BIN_LEN] + consts.ZERO_BYTE, "big")

    @property
    def datetime(self) -> dt.datetime:
        return dt.datetime.fromtimestamp(self.milliseconds / consts.MS_PER_SEC, dt.timezone.utc)

    @property
    def hex(self) -> str:
        return self.b.hex()

    def to_uuid(self) -> uuid.UUID:
        return uuid.UUID(bytes=self.b)

    def __repr__(self) -> str:
        return f"UPID({self!s})"

    def __str__(self) -> str:
        return b32.encode(self.b)

    def __int__(self) -> int:
        return int.from_bytes(self.b, "big")

    def __bytes__(self) -> bytes:
        return self.b

    def __lt__(self, other: object) -> bool:
        if isinstance(other, UPID):
            return self.b < other.b
        return NotImplemented

    def __eq__(self, other: object) -> bool:
        if isinstance(other, UPID):
            return self.b == other.b
        return False

    def __hash__(self) -> int:
        return hash(self.b)


def upid(prefix: str) -> UPID:
    """Generate a UPID with the provided prefix."""
    return UPID.from_prefix(prefix)
