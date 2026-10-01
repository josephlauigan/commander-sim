"""Test package. A runaway test (an endless loop that keeps allocating) fails with MemoryError under this ceiling
instead of filling the machine's memory and getting the whole session killed by the out-of-memory killer."""
try:
    import resource
    _CAP = 12 * 1024 ** 3
    _soft, _hard = resource.getrlimit(resource.RLIMIT_AS)
    if _soft == resource.RLIM_INFINITY or _soft > _CAP:
        resource.setrlimit(resource.RLIMIT_AS, (_CAP, _hard))
except (ImportError, ValueError, OSError):          # not on Linux / not allowed: run without the ceiling
    pass
