//! Contains representations of function literals. There are several complicating factors:
//!
//! - Functions can be generic, and can have specializations applied to them. These are not the
//!   same thing! For instance, a method of a generic class might not itself be generic, but it can
//!   still have the class's specialization applied to it.
//!
//! - Functions can be overloaded, and each overload can be independently generic or not, with
//!   different sets of typevars for different generic overloads. In some cases we need to consider
//!   each overload separately; in others we need to consider all of the overloads (and any
//!   implementation) as a single collective entity.
//!
//! - Certain "known" functions need special treatment — for instance, inferring a special return
//!   type, or raising custom diagnostics.
//!
//! - TODO: Some functions don't correspond to a function definition in the AST, and are instead
//!   synthesized as we mimic the behavior of the Python interpreter. Even though they are
//!   synthesized, and are "implemented" as Rust code, they are still functions from the POV of the
//!   rest of the type system.
//!
//! Given these constraints, we have the following representation: a function is a list of one or
//! more overloads, with zero or more specializations (more specifically, "type mappings") applied
//! to it. [`FunctionType`] is the outermost type, which is what [`Type::FunctionLiteral`] wraps.
//! It contains the list of type mappings to apply. It wraps a [`FunctionLiteral`], which collects
//! together all of the overloads (and implementation) of an overloaded function. An
//! [`OverloadLiteral`] represents an individual function definition in the AST — that is, each
//! overload (and implementation) of an overloaded function, or the single definition of a
//! non-overloaded function.
//!
//! Technically, each `FunctionLiteral` wraps a particular overload and all _previous_ overloads.
//! So it's only true that it wraps _all_ overloads if you are looking at the last definition. For
//! instance, in
//!
//! ```py
//! @overload
//! def f(x: int) -> None: ...
//! # <-- 1
//!
//! @overload
//! def f(x: str) -> None: ...
//! # <-- 2
//!
//! def f(x): pass
//! # <-- 3
//! ```
//!
//! resolving `f` at each of the three numbered positions will give you a `FunctionType`, which
//! wraps a `FunctionLiteral`, which contain `OverloadLiteral`s only for the definitions that
//! appear before that position. We rely on the fact that later definitions shadow earlier ones, so
//! the public type of `f` is resolved at position 3, correctly giving you all of the overloads
//! (and the implementation).

use std::str::FromStr;

use bitflags::bitflags;
use ruff_db::diagnostic::{Annotation, DiagnosticId, Severity, Span};
use ruff_db::files::{File, FileRange};
use ruff_db::parsed::{ParsedModuleRef, parsed_module};
use ruff_python_ast::{self as ast, ParameterWithDefault};
use ruff_text_size::Ranged;
use ty_module_resolver::{KnownModule, ModuleName, file_to_module, resolve_module};

use crate::place::{DefinedPlace, Definedness, Place, place_from_bindings};
use crate::semantic_index::ast_ids::HasScopedUseId;
use crate::semantic_index::definition::Definition;
use crate::semantic_index::scope::ScopeId;
use crate::semantic_index::{FileScopeId, SemanticIndex, semantic_index};
use crate::types::call::{Binding, CallArguments};
use crate::types::callable::CallableTypeKind;
use crate::types::constraints::ConstraintSet;
use crate::types::context::InferContext;
use crate::types::diagnostic::{
    ASSERT_TYPE_UNSPELLABLE_SUBTYPE, INVALID_ARGUMENT_TYPE, REDUNDANT_CAST, STATIC_ASSERT_ERROR,
    TYPE_ASSERTION_FAILURE, report_bad_argument_to_get_protocol_members,
    report_bad_argument_to_protocol_interface, report_invalid_total_ordering_call,
    report_issubclass_check_against_protocol_with_non_method_members,
    report_runtime_check_against_non_runtime_checkable_protocol,
    report_runtime_check_against_typed_dict,
};
use crate::types::display::DisplaySettings;
use crate::types::generics::{GenericContext, typing_self};
use crate::types::infer::nearest_enclosing_class;
use crate::types::known_instance::DeprecatedInstance;
use crate::types::list_members::all_members;
use crate::types::narrow::ClassInfoConstraintFunction;
use crate::types::relation::TypeRelationChecker;
use crate::types::signatures::{CallableSignature, Signature};
use crate::types::visitor::any_over_type;
use crate::types::{
    ApplyTypeMappingVisitor, BoundMethodType, BoundTypeVarInstance, CallableType, ClassBase,
    ClassLiteral, ClassType, DynamicType, FindLegacyTypeVarsVisitor, IntersectionBuilder,
    KnownClass, KnownInstanceType, LiteralValueType, SpecialFormType, SubclassOfInner,
    SubclassOfType, Truthiness, Type, TypeContext, TypeMapping, TypeVarBoundOrConstraints,
    UnionBuilder, UnionType, binding_type, definition_expression_type, infer_definition_types,
    walk_signature,
};
use crate::{Db, FxOrderSet};

/// A collection of useful spans for annotating functions.
///
/// This can be retrieved via `FunctionType::spans` or
/// `Type::function_spans`.
pub(crate) struct FunctionSpans {
    /// The span of the entire function "signature." This includes
    /// the name, parameter list and return type (if present).
    pub(crate) signature: Span,
    /// The span of the function name. i.e., `foo` in `def foo(): ...`.
    pub(crate) name: Span,
    /// The span of the parameter list, including the opening and
    /// closing parentheses.
    pub(crate) parameters: Span,
    /// The span of the annotated return type, if present.
    pub(crate) return_type: Option<Span>,
}

bitflags! {
    #[derive(Copy, Clone, Debug, Eq, PartialEq, Default, Hash)]
    pub struct FunctionDecorators: u8 {
        /// `@classmethod`
        const CLASSMETHOD = 1 << 0;
        /// `@typing.no_type_check`
        const NO_TYPE_CHECK = 1 << 1;
        /// `@typing.overload`
        const OVERLOAD = 1 << 2;
        /// `@abc.abstractmethod`
        const ABSTRACT_METHOD = 1 << 3;
        /// `@typing.final`
        const FINAL = 1 << 4;
        /// `@staticmethod`
        const STATICMETHOD = 1 << 5;
        /// `@typing.override`
        const OVERRIDE = 1 << 6;
        /// `@typing.type_check_only`
        const TYPE_CHECK_ONLY = 1 << 7;
    }
}

impl get_size2::GetSize for FunctionDecorators {}

impl FunctionDecorators {
    pub(super) fn from_decorator_type(db: &dyn Db, decorator_type: Type) -> Self {
        match decorator_type {
            Type::FunctionLiteral(function) => match function.known(db) {
                Some(KnownFunction::NoTypeCheck) => FunctionDecorators::NO_TYPE_CHECK,
                Some(KnownFunction::Overload) => FunctionDecorators::OVERLOAD,
                Some(KnownFunction::AbstractMethod) => FunctionDecorators::ABSTRACT_METHOD,
                Some(KnownFunction::Final) => FunctionDecorators::FINAL,
                Some(KnownFunction::Override) => FunctionDecorators::OVERRIDE,
                Some(KnownFunction::TypeCheckOnly) => FunctionDecorators::TYPE_CHECK_ONLY,
                _ => FunctionDecorators::empty(),
            },
            Type::ClassLiteral(class) => match class.known(db) {
                Some(KnownClass::Classmethod) => FunctionDecorators::CLASSMETHOD,
                Some(KnownClass::Staticmethod) => FunctionDecorators::STATICMETHOD,
                _ => FunctionDecorators::empty(),
            },
            _ => FunctionDecorators::empty(),
        }
    }
}

bitflags! {
    /// Used for the return type of `dataclass_transform(…)` calls. Keeps track of the
    /// arguments that were passed in. For the precise meaning of the fields, see [1].
    ///
    /// [1]: https://docs.python.org/3/library/typing.html#typing.dataclass_transform
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, salsa::Update)]
    pub struct DataclassTransformerFlags: u8 {
        const EQ_DEFAULT = 1 << 0;
        const ORDER_DEFAULT = 1 << 1;
        const KW_ONLY_DEFAULT = 1 << 2;
        const FROZEN_DEFAULT = 1 << 3;
    }
}

impl get_size2::GetSize for DataclassTransformerFlags {}

impl Default for DataclassTransformerFlags {
    fn default() -> Self {
        Self::EQ_DEFAULT
    }
}

/// Metadata for a dataclass-transformer. Stored inside a `Type::DataclassTransformer(…)`
/// instance that we use as the return type for `dataclass_transform(…)` calls.
#[salsa::interned(debug, heap_size=ruff_memory_usage::heap_size)]
pub struct DataclassTransformerParams<'db> {
    pub flags: DataclassTransformerFlags,

    #[returns(deref)]
    pub field_specifiers: Box<[Type<'db>]>,
}

impl get_size2::GetSize for DataclassTransformerParams<'_> {}

/// Metadata for `@overload_mapping(...)` decorators.
#[salsa::interned(debug, heap_size=ruff_memory_usage::heap_size)]
pub struct OverloadMappingParams<'db> {
    #[returns(ref)]
    pub parameter_name: ast::name::Name,

    /// Pairs of `(argument literal value, return type)` in source order.
    #[returns(deref)]
    pub mappings: Box<[(LiteralValueType<'db>, Type<'db>)]>,
}

impl get_size2::GetSize for OverloadMappingParams<'_> {}

/// Whether a function should implicitly be treated as a staticmethod based on its name.
pub(crate) fn is_implicit_staticmethod(function_name: &str) -> bool {
    matches!(function_name, "__new__")
}

/// Whether a function should implicitly be treated as a classmethod based on its name.
pub(crate) fn is_implicit_classmethod(function_name: &str) -> bool {
    matches!(function_name, "__init_subclass__" | "__class_getitem__")
}

/// Representation of a function definition in the AST: either a non-generic function, or a generic
/// function that has not been specialized.
///
/// If a function has multiple overloads, each overload is represented by a separate function
/// definition in the AST, and is therefore a separate `OverloadLiteral` instance.
#[salsa::interned(debug, heap_size=ruff_memory_usage::heap_size)]
pub struct OverloadLiteral<'db> {
    /// Name of the function at definition.
    #[returns(ref)]
    pub name: ast::name::Name,

    /// Is this a function that we special-case somehow? If so, which one?
    pub(crate) known: Option<KnownFunction>,

    /// The scope that's created by the function, in which the function body is evaluated.
    pub(crate) body_scope: ScopeId<'db>,

    /// A set of special decorators that were applied to this function
    pub(crate) decorators: FunctionDecorators,

    /// If `Some` then contains the `@warnings.deprecated`
    pub(crate) deprecated: Option<DeprecatedInstance<'db>>,

    /// The arguments to `dataclass_transformer`, if this function was annotated
    /// with `@dataclass_transformer(...)`.
    pub(crate) dataclass_transformer_params: Option<DataclassTransformerParams<'db>>,

    /// Metadata for `@overload_mapping(...)`, if present.
    pub(crate) overload_mapping_params: Option<OverloadMappingParams<'db>>,
}

// The Salsa heap is tracked separately.
impl get_size2::GetSize for OverloadLiteral<'_> {}

#[salsa::tracked]
impl<'db> OverloadLiteral<'db> {
    fn with_dataclass_transformer_params(
        self,
        db: &'db dyn Db,
        params: DataclassTransformerParams<'db>,
    ) -> Self {
        Self::new(
            db,
            self.name(db).clone(),
            self.known(db),
            self.body_scope(db),
            self.decorators(db),
            self.deprecated(db),
            Some(params),
            self.overload_mapping_params(db),
        )
    }

    fn file(self, db: &'db dyn Db) -> File {
        // NOTE: Do not use `self.definition(db).file(db)` here, as that could create a
        // cross-module dependency on the full AST.
        self.body_scope(db).file(db)
    }

    pub(crate) fn has_known_decorator(self, db: &dyn Db, decorator: FunctionDecorators) -> bool {
        self.decorators(db).contains(decorator)
    }

    pub(crate) fn is_overload(self, db: &dyn Db) -> bool {
        self.has_known_decorator(db, FunctionDecorators::OVERLOAD)
    }

    /// Returns true if this overload is decorated with `@staticmethod`, or if it is implicitly a
    /// staticmethod.
    pub(crate) fn is_staticmethod(self, db: &dyn Db) -> bool {
        self.has_known_decorator(db, FunctionDecorators::STATICMETHOD)
            || is_implicit_staticmethod(self.name(db))
    }

    /// Returns true if this overload is decorated with `@classmethod`, or if it is implicitly a
    /// classmethod.
    pub(crate) fn is_classmethod(self, db: &dyn Db) -> bool {
        self.has_known_decorator(db, FunctionDecorators::CLASSMETHOD)
            || is_implicit_classmethod(self.name(db))
    }

    pub(crate) fn node<'ast>(
        self,
        db: &dyn Db,
        file: File,
        module: &'ast ParsedModuleRef,
    ) -> &'ast ast::StmtFunctionDef {
        debug_assert_eq!(
            file,
            self.file(db),
            "OverloadLiteral::node() must be called with the same file as the one where \
            the function is defined."
        );

        self.body_scope(db).node(db).expect_function().node(module)
    }

    /// Iterate through the