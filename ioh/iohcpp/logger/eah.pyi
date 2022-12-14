from typing import Tuple

class IntegerScale:
    def __init__(self, *args, **kwargs) -> None: ...
    def bounds(self, arg0: int) -> Tuple[float,float]: ...
    def index(self, arg0: float) -> int: ...
    @property
    def length(self) -> int: ...
    @property
    def max(self) -> int: ...
    @property
    def min(self) -> int: ...
    @property
    def size(self) -> int: ...

class LinearIntegerScale(IntegerScale):
    def __init__(self, arg0: int, arg1: int, arg2: int) -> None: ...
    def step(self) -> int: ...

class LinearRealScale(RealScale):
    def __init__(self, arg0: float, arg1: float, arg2: int) -> None: ...
    def step(self) -> float: ...

class Log10IntegerScale(IntegerScale):
    def __init__(self, arg0: int, arg1: int, arg2: int) -> None: ...

class Log10RealScale(RealScale):
    def __init__(self, arg0: float, arg1: float, arg2: int) -> None: ...

class Log2IntegerScale(IntegerScale):
    def __init__(self, arg0: int, arg1: int, arg2: int) -> None: ...

class Log2RealScale(RealScale):
    def __init__(self, arg0: float, arg1: float, arg2: int) -> None: ...

class RealScale:
    def __init__(self, *args, **kwargs) -> None: ...
    def bounds(self, arg0: int) -> Tuple[float,float]: ...
    def index(self, arg0: float) -> int: ...
    @property
    def length(self) -> float: ...
    @property
    def max(self) -> float: ...
    @property
    def min(self) -> float: ...
    @property
    def size(self) -> int: ...
