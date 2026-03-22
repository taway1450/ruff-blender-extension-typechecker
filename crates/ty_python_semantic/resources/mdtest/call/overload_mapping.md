# Overload mapping

## Function via module import: mapped return

`bpy/__init__.pyi`:

```pyi
from . import stub_internal
from . import types
```

`bpy/stub_internal/__init__.pyi`:

```pyi
from . import overload_mapping
```

`bpy/stub_internal/overload_mapping.pyi`:

```pyi
from typing import Callable
from typing import TypeVar

_T = TypeVar("_T")

def overload_mapping(argument: str, mapping: dict[object, object]) -> Callable[[_T], _T]: ...
```

`bpy/types.pyi`:

```pyi
from .stub_internal.overload_mapping import overload_mapping

@overload_mapping("mode", {"A": int, "B": str})
def choose(mode: str) -> object: ...
```

`main.py`:

```py
import bpy.types as bpy_types

reveal_type(bpy_types.choose("A"))  # revealed: int
reveal_type(bpy_types.choose("B"))  # revealed: str
```

## Function via direct import: mapped return

`bpy/__init__.pyi`:

```pyi
from . import stub_internal
from . import types
```

`bpy/stub_internal/__init__.pyi`:

```pyi
from . import overload_mapping
```

`bpy/stub_internal/overload_mapping.pyi`:

```pyi
from typing import Callable
from typing import TypeVar

_T = TypeVar("_T")

def overload_mapping(argument: str, mapping: dict[object, object]) -> Callable[[_T], _T]: ...
```

`bpy/types.pyi`:

```pyi
from .stub_internal.overload_mapping import overload_mapping

@overload_mapping("mode", {"A": int, "B": str})
def choose(mode: str) -> object: ...
```

`main.py`:

```py
from bpy.types import choose

reveal_type(choose("A"))  # revealed: int
reveal_type(choose("B"))  # revealed: str
```

## Function via module import: no matching key

`bpy/__init__.pyi`:

```pyi
from . import stub_internal
from . import types
```

`bpy/stub_internal/__init__.pyi`:

```pyi
from . import overload_mapping
```

`bpy/stub_internal/overload_mapping.pyi`:

```pyi
from typing import Callable
from typing import TypeVar

_T = TypeVar("_T")

def overload_mapping(argument: str, mapping: dict[object, object]) -> Callable[[_T], _T]: ...
```

`bpy/types.pyi`:

```pyi
from .stub_internal.overload_mapping import overload_mapping

@overload_mapping("mode", {"A": int, "B": str})
def choose(mode: str) -> object: ...
```

`main.py`:

```py
import bpy.types as bpy_types

bpy_types.choose("C")  # error: [no-matching-overload]
```

## Function via direct import: no matching key

`bpy/__init__.pyi`:

```pyi
from . import stub_internal
from . import types
```

`bpy/stub_internal/__init__.pyi`:

```pyi
from . import overload_mapping
```

`bpy/stub_internal/overload_mapping.pyi`:

```pyi
from typing import Callable
from typing import TypeVar

_T = TypeVar("_T")

def overload_mapping(argument: str, mapping: dict[object, object]) -> Callable[[_T], _T]: ...
```

`bpy/types.pyi`:

```pyi
from .stub_internal.overload_mapping import overload_mapping

@overload_mapping("mode", {"A": int, "B": str})
def choose(mode: str) -> object: ...
```

`main.py`:

```py
from bpy.types import choose

choose("C")  # error: [no-matching-overload]
```

## Method via module import: mapped return

`bpy/__init__.pyi`:

```pyi
from . import stub_internal
from . import types
```

`bpy/stub_internal/__init__.pyi`:

```pyi
from . import overload_mapping
```

`bpy/stub_internal/overload_mapping.pyi`:

```pyi
from typing import Callable
from typing import TypeVar

_T = TypeVar("_T")

def overload_mapping(argument: str, mapping: dict[object, object]) -> Callable[[_T], _T]: ...
```

`bpy/types.pyi`:

```pyi
from .stub_internal.overload_mapping import overload_mapping

class Parser:
    @overload_mapping("kind", {"int": int, "str": str})
    def parse(self, kind: str) -> object: ...
```

`main.py`:

```py
import bpy.types as bpy_types

parser_from_module = bpy_types.Parser()
reveal_type(parser_from_module.parse("int"))  # revealed: int
reveal_type(parser_from_module.parse("str"))  # revealed: str
```

## Method via direct class import: mapped return

`bpy/__init__.pyi`:

```pyi
from . import stub_internal
from . import types
```

`bpy/stub_internal/__init__.pyi`:

```pyi
from . import overload_mapping
```

`bpy/stub_internal/overload_mapping.pyi`:

```pyi
from typing import Callable
from typing import TypeVar

_T = TypeVar("_T")

def overload_mapping(argument: str, mapping: dict[object, object]) -> Callable[[_T], _T]: ...
```

`bpy/types.pyi`:

```pyi
from .stub_internal.overload_mapping import overload_mapping

class Parser:
    @overload_mapping("kind", {"int": int, "str": str})
    def parse(self, kind: str) -> object: ...
```

`main.py`:

```py
from bpy.types import Parser

parser_from_direct_import = Parser()
reveal_type(parser_from_direct_import.parse("int"))  # revealed: int
reveal_type(parser_from_direct_import.parse("str"))  # revealed: str
```

## Method via module import: no matching key

`bpy/__init__.pyi`:

```pyi
from . import stub_internal
from . import types
```

`bpy/stub_internal/__init__.pyi`:

```pyi
from . import overload_mapping
```

`bpy/stub_internal/overload_mapping.pyi`:

```pyi
from typing import Callable
from typing import TypeVar

_T = TypeVar("_T")

def overload_mapping(argument: str, mapping: dict[object, object]) -> Callable[[_T], _T]: ...
```

`bpy/types.pyi`:

```pyi
from .stub_internal.overload_mapping import overload_mapping

class Parser:
    @overload_mapping("kind", {"int": int, "str": str})
    def parse(self, kind: str) -> object: ...
```

`main.py`:

```py
import bpy.types as bpy_types

parser_from_module = bpy_types.Parser()
parser_from_module.parse("other")  # error: [no-matching-overload]
```

## Method via direct class import: no matching key

`bpy/__init__.pyi`:

```pyi
from . import stub_internal
from . import types
```

`bpy/stub_internal/__init__.pyi`:

```pyi
from . import overload_mapping
```

`bpy/stub_internal/overload_mapping.pyi`:

```pyi
from typing import Callable
from typing import TypeVar

_T = TypeVar("_T")

def overload_mapping(argument: str, mapping: dict[object, object]) -> Callable[[_T], _T]: ...
```

`bpy/types.pyi`:

```pyi
from .stub_internal.overload_mapping import overload_mapping

class Parser:
    @overload_mapping("kind", {"int": int, "str": str})
    def parse(self, kind: str) -> object: ...
```

`main.py`:

```py
from bpy.types import Parser

parser_from_direct_import = Parser()
parser_from_direct_import.parse("other")  # error: [no-matching-overload]
```
