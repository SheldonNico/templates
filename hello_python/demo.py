from typing import Any, Dict, Callable, TypeVar  # noqa
import functools, time

T = TypeVar("T")
def timer(func: Callable[..., T]) -> Callable[..., T]:
    """Print the runtime of the decorated function"""
    @functools.wraps(func)
    def wrapper_timer(*args, **kwargs):
        start_time = time.perf_counter()  # 1
        value = func(*args, **kwargs)
        end_time = time.perf_counter()  # 2
        run_time = end_time - start_time  # 3
        print(f"Finished {func.__name__!r} in {run_time:.4f} secs")
        return value
    return wrapper_timer

def test_fstring():
    user = "Jane Doe"
    action = "buy"
    print(f"{user} has logged in adn did action. {action} ")

def test_pathlib():
    from pathlib import Path, PurePath
    p = Path(".")
    print("iter over path", [d for d in p.iterdir() if d.is_file()])
    print("glob path", list(p.glob("**/*.rs")))
    print("concat", p / "next" / "more")
    print("check exist:", (p / "tui_simple/").exists())
    with (p / "README.md").open(mode="r") as fh:
        print("open file: ", fh.readline().strip())
    print("Accessing: ", p.absolute())
    print("Accessing: ", p.drive)
    print("Accessing: ", p.anchor)
    print("Accessing: ", PurePath("sdfsad.py").stem)
    print("Accessing: ", PurePath("sdfsad.py").suffix)

def test_enum():
    from enum import Enum, auto

    class Monster(Enum):
        ZOMBIE = auto()
        BEAR = auto()

    print(Monster.ZOMBIE)

@timer
def test_fib(number: int) -> int:
    if number == 0 or number == 1: return number
    return test_fib(number-1) + test_fib(number-2)

@timer
@functools.lru_cache(maxsize=512)
def test_fib_lru(number: int) -> int:
    if number == 0 or number == 1: return number
    return test_fib(number-1) + test_fib(number-2)

def test_iterable_unpacking():
    head, *body, tail = range(5)
    print(body)

def test_dataclass():
    from dataclasses import dataclass

    @dataclass
    class Armor:
        armor: float
        description: str
        level: int = 1

        def power(self) -> float:
            return self.level*self.armor

    armor = Armor(5.2, "Common armor", 2)
    print(armor.power())

if __name__ == "__main__":
    test_fstring()
    test_pathlib()
    test_enum()
    test_fib(20)
    test_fib_lru(20)
    test_iterable_unpacking()
    test_dataclass()

