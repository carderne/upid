import sys

from upid import upid

if __name__ == "__main__":
    if len(sys.argv) > 1:
        prefix = sys.argv[1]
    else:
        prefix = ""
    print(str(upid(prefix)))
