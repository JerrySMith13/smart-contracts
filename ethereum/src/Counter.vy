# pragma version 0.4.3

# Same counter as Counter.sol.
#
# `count` is one storage variable (slot 0). Marking it `public` makes Vyper
# emit a getter, `count() -> uint256`. A read is a call. `increment` and `set`
# write storage, so they have to run inside a transaction.
# Storage is addressed through `self`. A bare `count = ...` would not compile.

count: public(uint256)


@external
def increment():
    self.count += 1


@external
def set(new_count: uint256):
    self.count = new_count
