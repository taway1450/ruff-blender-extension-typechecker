<!-- WARNING: This file is auto-generated (cargo dev generate-all). Edit the lint-declarations in 'crates/ty_python_semantic/src/types/diagnostic.rs' if you want to change anything here. -->

# Rules

## `abstract-method-in-final-class`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'error'."><code>error</code></a> ·
Added in <a href="https://github.com/astral-sh/ty/releases/tag/0.0.13">0.0.13</a> ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22abstract-method-in-final-class%22" target="_blank">Related issues</a> ·
<a href="https://github.com/astral-sh/ruff/blob/main/crates%2Fty_python_semantic%2Fsrc%2Ftypes%2Fdiagnostic.rs#L2347" target="_blank">View source</a>
</small>


**What it does**

Checks for `@final` classes that have unimplemented abstract methods.

**Why is this bad?**

A class decorated with `@final` cannot be subclassed. If such a class has abstract
methods that are not implemented, the class can never be properly instantiated, as
the abstract methods can never be implemented (since subclassing is prohibited).

At runtime, instantiation of classes with unimplemented abstract methods is only
prevented for classes that have `ABCMeta` (or a subclass of it) as their metaclass.
However, type checkers also enforce this for classes that do not use `ABCMeta`, since
the intent for the class to be abstract is clear from the use of `@abstractmethod`.

**Example**


```python
from abc import ABC, abstractmethod
from typing import final

class Base(ABC):
    @abstractmethod
    def method(self) -> int: ...

@final
class Derived(Base):  # Error: `Derived` does not implement `method`
    pass
```

## `ambiguous-protocol-member`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'warn'."><code>warn</code></a> ·
Added in <a href="https://github.com/astral-sh/ty/releases/tag/0.0.1-alpha.20">0.0.1-alpha.20</a> ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22ambiguous-protocol-member%22" target="_blank">Related issues</a> ·
<a href="https://github.com/astral-sh/ruff/blob/main/crates%2Fty_python_semantic%2Fsrc%2Ftypes%2Fdiagnostic.rs#L655" target="_blank">View source</a>
</small>


**What it does**

Checks for protocol classes with members that will lead to ambiguous interfaces.

**Why is this bad?**

Assigning to an undeclared variable in a protocol class leads to an ambiguous
interface which may lead to the type checker inferring unexpected things. It's
recommended to ensure that all members of a protocol class are explicitly declared.

**Examples**


```py
from typing import Protocol

class BaseProto(Protocol):
    a: int                               # fine (explicitly declared as `int`)
    def method_member(self) -> int: ...  # fine: a method definition using `def` is considered a declaration
    c = "some variable"                  # error: no explicit declaration, leading to ambiguity
    b = method_member                    # error: no explicit declaration, leading to ambiguity

    # error: this creates implicit assignments of `d` and `e` in the protocol class body.
    # Were they really meant to be considered protocol members?
    for d, e in enumerate(range(42)):
        pass

class SubProto(BaseProto, Protocol):
    a = 42  # fine (declared in superclass)
```

## `assert-type-unspellable-subtype`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'error'."><code>error</code></a> ·
Added in <a href="https://github.com/astral-sh/ty/releases/tag/0.0.14">0.0.14</a> ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22assert-type-unspellable-subtype%22" target="_blank">Related issues</a> ·
<a href="https://github.com/astral-sh/ruff/blob/main/crates%2Fty_python_semantic%2Fsrc%2Ftypes%2Fdiagnostic.rs#L2483" target="_blank">View source</a>
</small>


**What it does**

Checks for `assert_type()` calls where the actual type
is an unspellable subtype of the asserted type.

**Why is this bad?**

`assert_type()` is intended to ensure that the inferred type of a value
is exactly the same as the asserted type. But in some situations, ty
has nonstandard extensions to the type system that allow it to infer
more precise types than can be expressed in user annotations. ty emits a
different error code to [`type-assertion-failure`](#type-assertion-failure) in these situations so
that users can easily differentiate between the two cases.

**Example**


```python
def _(x: int):
    assert_type(x, int)  # fine
    if x:
        assert_type(x, int)  # error: [assert-type-unspellable-subtype]
                             # the actual type is `int & ~AlwaysFalsy`,
                             # which excludes types like `Literal[0]`
```

## `blender-property-outside-register`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'error'."><code>error</code></a> ·
Preview (since <a href="https://github.com/astral-sh/ty/releases/tag/1.0.0">1.0.0</a>) ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22blender-property-outside-register%22" target="_blank">Related issues</a> ·
<a href="https://github.com/astral-sh/ruff/blob/main/crates%2Fty_python_semantic%2Fsrc%2Fblender_property.rs#L20" target="_blank">View source</a>
</small>


**What it does**

Checks for Blender dynamic property definitions outside the `register()` function scope.

**Why is this bad?**

Blender properties should only be registered from the `register()` function
(or functions it calls) in the project root `__init__.py`. Properties defined
elsewhere will not be recognized by the type checker.

**Example**

```python
# Bad: property defined at module top level
bpy.types.Scene.my_prop = bpy.props.StringProperty()

# Good: property defined inside register()
def register():
    bpy.types.Scene.my_prop = bpy.props.StringProperty()
```

## `byte-string-type-annotation`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'error'."><code>error</code></a> ·
Added in <a href="https://github.com/astral-sh/ty/releases/tag/0.0.1-alpha.1">0.0.1-alpha.1</a> ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22byte-string-type-annotation%22" target="_blank">Related issues</a> ·
<a href="https://github.com/astral-sh/ruff/blob/main/crates%2Fty_python_semantic%2Fsrc%2Ftypes%2Fstring_annotation.rs#L37" target="_blank">View source</a>
</small>


**What it does**

Checks for byte-strings in type annotation positions.

**Why is this bad?**

Static analysis tools like ty can't analyze type annotations that use byte-string notation.

**Examples**

```python
def test(): -> b"int":
    ...
```

Use instead:
```python
def test(): -> "int":
    ...
```

## `call-abstract-method`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'error'."><code>error</code></a> ·
Preview (since <a href="https://github.com/astral-sh/ty/releases/tag/0.0.16">0.0.16</a>) ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22call-abstract-method%22" target="_blank">Related issues</a> ·
<a href="https://github.com/astral-sh/ruff/blob/main/crates%2Fty_python_semantic%2Fsrc%2Ftypes%2Fdiagnostic.rs#L2382" target="_blank">View source</a>
</small>


**What it does**

Checks for calls to abstract `@classmethod`s or `@staticmethod`s
with "trivial bodies" when accessed on the class object itself.

"Trivial bodies" are bodies that solely consist of `...`, `pass`,
a docstring, and/or `raise NotImplementedError`.

**Why is this bad?**

An abstract method with a trivial body has no concrete implementation
to execute, so calling such a method directly on the class will probably
not have the desired effect.

It is also unsound to call these methods directly on the class. Unlike
other methods, ty permits abstract methods with trivial bodies to have
non-`None` return types even though they always return `None` at runtime.
This is because it is expected that these methods will always be
overridden rather than being called directly. As a result of this
exception to the normal rule, ty may infer an incorrect type if one of
these methods is called directly, which may then mean that type errors
elsewhere in your code go undetected by ty.

Calling abstract classmethods or staticmethods via `type[X]` is allowed,
since the actual runtime type could be a concrete subclass with an implementation.

**Example**

```python
from abc import ABC, abstractmethod

class Foo(ABC):
    @classmethod
    @abstractmethod
    def method(cls) -> int: ...

Foo.method()  # Error: cannot call abstract classmethod
```

## `call-non-callable`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'error'."><code>error</code></a> ·
Added in <a href="https://github.com/astral-sh/ty/releases/tag/0.0.1-alpha.1">0.0.1-alpha.1</a> ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22call-non-callable%22" target="_blank">Related issues</a> ·
<a href="https://github.com/astral-sh/ruff/blob/main/crates%2Fty_python_semantic%2Fsrc%2Ftypes%2Fdiagnostic.rs#L171" target="_blank">View source</a>
</small>


**What it does**

Checks for calls to non-callable objects.

**Why is this bad?**

Calling a non-callable object will raise a `TypeError` at runtime.

**Examples**

```python
4()  # TypeError: 'int' object is not callable
```

## `call-top-callable`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'error'."><code>error</code></a> ·
Added in <a href="https://github.com/astral-sh/ty/releases/tag/0.0.7">0.0.7</a> ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22call-top-callable%22" target="_blank">Related issues</a> ·
<a href="https://github.com/astral-sh/ruff/blob/main/crates%2Fty_python_semantic%2Fsrc%2Ftypes%2Fdiagnostic.rs#L189" target="_blank">View source</a>
</small>


**What it does**

Checks for calls to objects typed as `Top[Callable[..., T]]` (the infinite union of all
callable types with return type `T`).

**Why is this bad?**

When an object is narrowed to `Top[Callable[..., object]]` (e.g., via `callable(x)` or
`isinstance(x, Callable)`), we know the object is callable, but we don't know its
precise signature. This type represents the set of all possible callable types
(including, e.g., functions that take no arguments and functions that require arguments),
so no specific set of arguments can be guaranteed to be valid.

**Examples**

```python
def f(x: object):
    if callable(x):
        x()  # error: We know `x` is callable, but not what arguments it accepts
```

## `conflicting-argument-forms`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'error'."><code>error</code></a> ·
Added in <a href="https://github.com/astral-sh/ty/releases/tag/0.0.1-alpha.1">0.0.1-alpha.1</a> ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22conflicting-argument-forms%22" target="_blank">Related issues</a> ·
<a href="https://github.com/astral-sh/ruff/blob/main/crates%2Fty_python_semantic%2Fsrc%2Ftypes%2Fdiagnostic.rs#L240" target="_blank">View source</a>
</small>


**What it does**

Checks whether an argument is used as both a value and a type form in a call.

**Why is this bad?**

Such calls have confusing semantics and often indicate a logic error.

**Examples**

```python
from typing import reveal_type
from ty_extensions import is_singleton

if flag:
    f = repr  # Expects a value
else:
    f = is_singleton  # Expects a type form

f(int)  # error
```

## `conflicting-declarations`

<small>
Default level: <a href="../../rules#rule-levels" title="This lint has a default level of 'error'."><code>error</code></a> ·
Added in <a href="https://github.com/astral-sh/ty/releases/tag/0.0.1-alpha.1">0.0.1-alpha.1</a> ·
<a href="https://github.com/astral-sh/ty/issues?q=sort%3Aupdated-desc%20is%3Aissue%20is%3Aopen%20%22conflicting-declarations%22" target="_blank">Related issues</a> ·