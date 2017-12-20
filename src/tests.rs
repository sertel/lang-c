use ast::*;
use env::Env;
use span::{Node, Span};

fn ident<T: From<Identifier>>(i: &str) -> T {
    Identifier {
        name: i.to_string(),
    }.into()
}

impl<T> From<T> for Node<T> {
    fn from(t: T) -> Node<T> {
        Node::new(t, Span::none())
    }
}

impl<T> From<Box<T>> for Node<T> {
    fn from(t: Box<T>) -> Node<T> {
        (*t).into()
    }
}

impl<T> From<T> for Box<Node<T>> {
    fn from(t: T) -> Box<Node<T>> {
        Box::new(t.into())
    }
}

impl<T> From<Box<T>> for Box<Node<T>> {
    fn from(t: Box<T>) -> Box<Node<T>> {
        (*t).into()
    }
}

impl<'a> From<&'a str> for Node<String> {
    fn from(t: &'a str) -> Node<String> {
        t.to_owned().into()
    }
}

macro_rules! mk_from_inner {
    ( $( $i:ident => $p:ident :: $v:ident ; )* ) => (
        $(
            impl From<$i> for Node<$p> {
                fn from(i: $i) -> Node<$p> {
                    $p::$v(i.into()).into()
                }
            }

            impl From<$i> for Box<Node<$p>> {
                fn from(i: $i) -> Box<Node<$p>> {
                    $p::$v(i.into()).into()
                }
            }
        )*
    );
}

mk_from_inner! {
    Attribute => Extension::Attribute;
    StructField => StructDeclaration::Field;
    FunctionDeclarator => DerivedDeclarator::Function;
    ArrayDeclarator => DerivedDeclarator::Array;
    TS18661FloatType => TypeSpecifier::TS18661Float;
    TS18661FloatType => DeclarationSpecifier::TypeSpecifier;
    Constant => Expression::Constant;
    TypeSpecifier => DeclarationSpecifier::TypeSpecifier;
    TypeSpecifier => SpecifierQualifier::TypeSpecifier;
    Identifier => DeclaratorKind::Identifier;
    StructType => TypeSpecifier::Struct;
    StructType => DeclarationSpecifier::TypeSpecifier;
    Declaration => ExternalDeclaration::Declaration;
    StaticAssert => ExternalDeclaration::StaticAssert;
}

impl From<Constant> for Node<Initializer> {
    fn from(c: Constant) -> Node<Initializer> {
        Initializer::Expression(c.into()).into()
    }
}

mod expr {
    use ast::*;
    use span::Node;

    pub fn var<T: From<Expression>>(i: &str) -> T {
        Expression::Identifier(super::ident(i)).into()
    }

    pub fn string<T: From<Expression>>(i: &str) -> T {
        Expression::StringLiteral(vec![i.to_string()].into()).into()
    }

    pub fn unop<T: From<Expression>>(op: UnaryOperator, e: Box<Expression>) -> T {
        Expression::UnaryOperator(
            UnaryOperatorExpression {
                operator: op.into(),
                operand: e.into(),
            }.into(),
        ).into()
    }

    pub fn binop<T: From<Expression>>(op: BinaryOperator, a: Box<Expression>, b: Box<Expression>) -> T {
        Expression::BinaryOperator(
            BinaryOperatorExpression {
                operator: op.into(),
                lhs: a.into(),
                rhs: b.into(),
            }.into(),
        ).into()
    }

    pub fn member<T: From<Expression>>(op: MemberOperator, e: Box<Expression>, i: Node<Identifier>) -> T {
        Expression::Member(
            MemberExpression {
                operator: op.into(),
                expression: Box::new(e.into()),
                identifier: i,
            }.into(),
        ).into()
    }

    pub fn cconst<T: From<Expression>>(c: Constant) -> T {
        Expression::Constant(c.into()).into()
    }
}

mod int {
    use ast::*;

    pub fn dec(i: &str) -> Constant {
        Constant::Integer(Integer::Decimal(i.to_string())).into()
    }

    pub fn oct(i: &str) -> Constant {
        Constant::Integer(Integer::Octal(i.to_string()))
    }

    pub fn hex(i: &str) -> Constant {
        Constant::Integer(Integer::Hexademical(i.to_string()))
    }
}

mod float {
    use ast::*;

    pub fn dec(i: &str) -> Constant {
        Constant::Float(Float::Decimal(i.to_string()))
    }

    pub fn hex(i: &str) -> Constant {
        Constant::Float(Float::Hexademical(i.to_string()))
    }
}

fn cchar(i: &str) -> Constant {
    Constant::Character(i.to_string())
}

fn cstr<T: From<StringLiteral>>(i: &[&str]) -> T {
    i.into_iter()
        .map(|s| String::from(*s))
        .collect::<Vec<String>>()
        .into()
}

#[test]
fn test_integer() {
    use self::int::*;
    use parser::constant;

    let env = &mut Env::new();

    assert_eq!(constant("0", env), Ok(oct("0")));
    assert_eq!(constant("1", env), Ok(dec("1")));
    assert_eq!(constant("1234567890", env), Ok(dec("1234567890")));
    assert_eq!(constant("01234567", env), Ok(oct("01234567")));
    assert_eq!(
        constant("0x1234567890abdefABCDEF", env),
        Ok(hex("0x1234567890abdefABCDEF"))
    );
    assert_eq!(constant("042lu", env), Ok(oct("042lu")));

    assert!(constant("1a", env).is_err());
    assert!(constant("08", env).is_err());
    assert!(constant("0xX", env).is_err());
}

#[test]
fn test_floating() {
    use self::float::*;
    use parser::constant;

    let env = &mut Env::new();

    assert_eq!(constant("2.", env), Ok(dec("2.")));
    assert_eq!(constant("2.e2", env), Ok(dec("2.e2")));
    assert_eq!(constant(".2", env), Ok(dec(".2")));
    assert_eq!(constant(".2e2", env), Ok(dec(".2e2")));
    assert_eq!(constant("2.0", env), Ok(dec("2.0")));

    assert_eq!(constant("24.01e100", env), Ok(dec("24.01e100")));
    assert_eq!(constant("24.01e+100", env), Ok(dec("24.01e+100")));
    assert_eq!(constant("24.01e-100", env), Ok(dec("24.01e-100")));
    assert_eq!(constant("24.01e100f", env), Ok(dec("24.01e100f")));

    assert_eq!(constant("0x2Ap19L", env), Ok(hex("0x2Ap19L")));
    assert_eq!(constant("0x2A.p19L", env), Ok(hex("0x2A.p19L")));
    assert_eq!(constant("0x.DEp19L", env), Ok(hex("0x.DEp19L")));
    assert_eq!(constant("0x2A.DEp19L", env), Ok(hex("0x2A.DEp19L")));
}

#[test]
fn test_character() {
    use parser::constant;

    let env = &mut Env::new();

    assert_eq!(constant("'a'", env), Ok(cchar("'a'")));
    assert_eq!(constant(r"'\n'", env), Ok(cchar(r"'\n'")));
    assert_eq!(constant(r"'\\'", env), Ok(cchar(r"'\\'")));
    assert_eq!(constant(r"'\''", env), Ok(cchar(r"'\''")));
    assert_eq!(constant(r"'\1'", env), Ok(cchar(r"'\1'")));
    assert_eq!(constant(r"'\02'", env), Ok(cchar(r"'\02'")));
    assert_eq!(constant(r"'\027'", env), Ok(cchar(r"'\027'")));
    assert_eq!(constant(r"'\xde'", env), Ok(cchar(r"'\xde'")));
}

#[test]
fn test_string() {
    use self::expr::*;
    use parser::expression;

    let env = &mut Env::new();

    assert_eq!(expression(r#""foo""#, env), Ok(string(r#""foo""#)));
    assert_eq!(expression(r#""foo\n""#, env), Ok(string(r#""foo\n""#)));
    assert_eq!(expression(r#""\'\"""#, env), Ok(string(r#""\'\"""#)));
    assert_eq!(expression(r#""\xaf""#, env), Ok(string(r#""\xaf""#)));
}

#[test]
fn test_postfix() {
    use self::expr::*;
    use ast::BinaryOperator::Index;
    use ast::MemberOperator::{Direct, Indirect};
    use ast::UnaryOperator::PostIncrement;
    use parser::expression;

    let env = &mut Env::new();

    assert_eq!(expression("a  ++", env), Ok(unop(PostIncrement, var("a"))));
    assert_eq!(
        expression("a.b->c[ d[ e ] ] ++", env),
        Ok(unop(
            PostIncrement,
            binop(
                Index,
                member(Indirect, member(Direct, var("a"), ident("b")), ident("c")),
                binop(Index, var("d"), var("e")),
            ),
        ),)
    );
}

#[test]
fn test_multiplicative() {
    use self::expr::*;
    use ast::BinaryOperator::{Divide, Multiply};
    use ast::UnaryOperator::{PostDecrement, PreIncrement};
    use parser::expression;

    let mut env = Env::new();
    let env = &mut env;

    assert_eq!(
        expression("a-- * ++b / c", env),
        Ok(binop(
            Divide,
            binop(
                Multiply,
                unop(PostDecrement, var("a")),
                unop(PreIncrement, var("b")),
            ),
            var("c"),
        ),)
    );
}

#[test]
fn test_comma() {
    use self::expr::*;
    use ast::Expression::Comma;
    use parser::expression;

    let env = &mut Env::new();

    assert_eq!(expression("a", env), Ok(var("a")));
    assert_eq!(
        expression("a, a, a,a\n,a", env),
        Ok(Comma(vec![var("a"); 5]).into())
    );
}

#[test]
fn test_cast() {
    use self::expr::*;
    use ast::Expression::Cast;
    use ast::SpecifierQualifier::TypeSpecifier;
    use ast::TypeName;
    use ast::TypeSpecifier::Int;
    use env::Env;
    use parser::expression;

    let env = &mut Env::new();

    assert_eq!(
        expression("(int) 1", env),
        Ok(Cast(
            CastExpression {
                type_name: TypeName {
                    specifiers: vec![TypeSpecifier(Int.into()).into()],
                    declarator: None,
                }.into(),
                expression: cconst(int::dec("1")),
            }.into()
        ).into())
    );

    assert!(expression("(foo) 1", env).is_err());
}

#[test]
fn test_declaration1() {
    use self::expr::*;
    use ast::ArraySize::{StaticExpression, VariableUnknown};
    use ast::DeclarationSpecifier::{StorageClass, TypeSpecifier};
    use ast::DerivedDeclarator::Pointer;
    use ast::Initializer::Expression;
    use ast::StorageClassSpecifier::Typedef;
    use ast::TypeQualifier::Const;
    use ast::TypeSpecifier::Int;
    use ast::UnaryOperator::Address;
    use parser::declaration;

    let mut env = Env::new();
    let env = &mut env;

    assert_eq!(
        declaration("int typedef * foo = &bar, baz[static 10][const *];", env),
        Ok(Declaration {
            specifiers: vec![
                TypeSpecifier(Int.into()).into(),
                StorageClass(Typedef.into()).into(),
            ],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("foo"),
                        derived: vec![Pointer(vec![]).into()],
                        extensions: vec![],
                    }.into(),
                    initializer: Some(Expression(unop(Address, var("bar"))).into()),
                }.into(),
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("baz"),
                        derived: vec![
                            ArrayDeclarator {
                                qualifiers: vec![],
                                size: StaticExpression(cconst(int::dec("10"))),
                            }.into(),
                            ArrayDeclarator {
                                qualifiers: vec![Const.into()],
                                size: VariableUnknown,
                            }.into(),
                        ],
                        extensions: vec![],
                    }.into(),
                    initializer: None,
                }.into(),
            ],
        }.into())
    );

    assert!(env.is_typename("foo"));
    assert!(env.is_typename("baz"));
}

#[test]
fn test_declaration2() {
    use self::expr::*;
    use ast::DeclarationSpecifier::{StorageClass, TypeSpecifier};
    use ast::DerivedDeclarator::Pointer;
    use ast::Enumerator;
    use ast::PointerQualifier::TypeQualifier;
    use ast::StorageClassSpecifier::Typedef;
    use ast::TypeQualifier::Const;
    use ast::TypeSpecifier::Enum;
    use parser::declaration;

    let mut env = Env::new();
    let env = &mut env;

    assert_eq!(
        declaration("typedef enum { FOO, BAR = 1 } * const foobar;", env),
        Ok(Declaration {
            specifiers: vec![
                StorageClass(Typedef.into()).into(),
                TypeSpecifier(
                    Enum(
                        EnumType {
                            identifier: None,
                            enumerators: vec![
                                Enumerator {
                                    identifier: ident("FOO"),
                                    expression: None,
                                }.into(),
                                Enumerator {
                                    identifier: ident("BAR"),
                                    expression: Some(cconst(int::dec("1"))),
                                }.into(),
                            ],
                        }.into(),
                    ).into(),
                ).into(),
            ],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("foobar"),
                        derived: vec![Pointer(vec![TypeQualifier(Const.into()).into()]).into()],
                        extensions: vec![],
                    }.into(),
                    initializer: None,
                }.into(),
            ],
        }.into())
    );

    assert!(env.is_typename("foobar"));
}

#[test]
fn test_declaration3() {
    use ast::DeclarationSpecifier::TypeSpecifier;
    use ast::TypeSpecifier::{Float, Int};
    use ast::TypeSpecifier::Struct;
    use parser::declaration;

    let mut env = Env::new();
    let env = &mut env;

    assert_eq!(
        declaration("struct { int a, b; float c; } S;", env).unwrap(),
        Declaration {
            specifiers: vec![
                TypeSpecifier(
                    Struct(
                        StructType {
                            kind: StructKind::Struct.into(),
                            identifier: None,
                            declarations: vec![
                                StructField {
                                    specifiers: vec![SpecifierQualifier::TypeSpecifier(Int.into()).into()],
                                    declarators: vec![
                                        StructDeclarator {
                                            declarator: Some(
                                                Declarator {
                                                    kind: ident("a"),
                                                    derived: vec![],
                                                    extensions: vec![],
                                                }.into(),
                                            ),
                                            bit_width: None,
                                        }.into(),
                                        StructDeclarator {
                                            declarator: Some(
                                                Declarator {
                                                    kind: ident("b"),
                                                    derived: vec![],
                                                    extensions: vec![],
                                                }.into(),
                                            ),
                                            bit_width: None,
                                        }.into(),
                                    ],
                                }.into(),
                                StructField {
                                    specifiers: vec![SpecifierQualifier::TypeSpecifier(Float.into()).into()],
                                    declarators: vec![
                                        StructDeclarator {
                                            declarator: Some(
                                                Declarator {
                                                    kind: ident("c"),
                                                    derived: vec![],
                                                    extensions: vec![],
                                                }.into(),
                                            ),
                                            bit_width: None,
                                        }.into(),
                                    ],
                                }.into(),
                            ],
                        }.into(),
                    ).into(),
                ).into(),
            ],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("S"),
                        derived: vec![],
                        extensions: vec![],
                    }.into(),
                    initializer: None,
                }.into(),
            ],
        }.into()
    );
}

#[test]
fn test_declaration4() {
    use ast::DeclarationSpecifier::{TypeQualifier, TypeSpecifier};
    use ast::TypeQualifier::Restrict;
    use ast::TypeSpecifier::Int;
    use parser::declaration;

    assert_eq!(
        declaration("int __restrict__;", &mut Env::with_gnu(false)),
        Ok(Declaration {
            specifiers: vec![TypeSpecifier(Int.into()).into()],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("__restrict__"),
                        derived: vec![],
                        extensions: vec![],
                    }.into(),
                    initializer: None,
                }.into(),
            ],
        }.into())
    );

    assert_eq!(
        declaration("int __restrict__;", &mut Env::with_gnu(true)),
        Ok(Declaration {
            specifiers: vec![
                TypeSpecifier(Int.into()).into(),
                TypeQualifier(Restrict.into()).into(),
            ],
            declarators: vec![],
        }.into())
    );
}

#[test]
fn test_declaration5() {
    use self::expr::cconst;
    use self::int::dec;
    use ast::ArraySize::VariableExpression;
    use ast::DeclarationSpecifier::{TypeQualifier, TypeSpecifier};
    use ast::DeclaratorKind::Abstract;
    use ast::DerivedDeclarator::Pointer;
    use ast::TypeQualifier::Const;
    use ast::TypeSpecifier::{Char, Int, TypedefName};
    use parser::declaration;

    let env = &mut Env::new();

    env.add_typename("FILE");
    env.add_typename("size_t");

    assert_eq!(
        declaration(
            "char *fparseln(FILE *, size_t *, size_t *, const char[3], int);",
            env
        ),
        Ok(Declaration {
            specifiers: vec![TypeSpecifier(Char.into()).into()],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("fparseln"),
                        derived: vec![
                            Pointer(vec![]).into(),
                            FunctionDeclarator {
                                parameters: vec![
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(TypedefName(ident("FILE")).into()).into()],
                                        declarator: Some(
                                            Declarator {
                                                kind: Abstract.into(),
                                                derived: vec![Pointer(vec![]).into()],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![],
                                    }.into(),
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(TypedefName(ident("size_t")).into()).into()],
                                        declarator: Some(
                                            Declarator {
                                                kind: Abstract.into(),
                                                derived: vec![Pointer(vec![]).into()],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![],
                                    }.into(),
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(TypedefName(ident("size_t")).into()).into()],
                                        declarator: Some(
                                            Declarator {
                                                kind: Abstract.into(),
                                                derived: vec![Pointer(vec![]).into()],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![],
                                    }.into(),
                                    ParameterDeclaration {
                                        specifiers: vec![
                                            TypeQualifier(Const.into()).into(),
                                            TypeSpecifier(Char.into()).into(),
                                        ],
                                        declarator: Some(
                                            Declarator {
                                                kind: Abstract.into(),
                                                derived: vec![
                                                    ArrayDeclarator {
                                                        qualifiers: vec![],
                                                        size: VariableExpression(cconst(dec("3"))),
                                                    }.into(),
                                                ],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![],
                                    }.into(),
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(Int.into()).into()],
                                        declarator: None,
                                        extensions: vec![],
                                    }.into(),
                                ],
                                ellipsis: Ellipsis::None,
                            }.into(),
                        ],
                        extensions: vec![],
                    }.into(),
                    initializer: None,
                }.into(),
            ],
        }.into())
    );
}

#[test]
fn test_attribute() {
    use self::expr::cconst;
    use ast::DeclarationSpecifier::{StorageClass, TypeSpecifier};
    use ast::DerivedDeclarator::Pointer;
    use ast::Extension::AsmLabel;
    use ast::StorageClassSpecifier::Extern;
    use ast::TypeSpecifier::{Char, Int, TypedefName};
    use parser::declaration;

    let env = &mut Env::new();
    env.add_typename("size_t");

    assert_eq!(
        declaration(
            concat!(
                "extern int strerror_r (int __errnum, char *__buf, size_t __buflen)\n",
                "__asm__  (\"\" \"__xpg_strerror_r\") __attribute__ ((__nothrow__ , __leaf__))\n",
                "__attribute__ ((__nonnull__ (2)));",
            ),
            env,
        ),
        Ok(Declaration {
            specifiers: vec![
                StorageClass(Extern.into()).into(),
                TypeSpecifier(Int.into()).into(),
            ],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("strerror_r"),
                        derived: vec![
                            FunctionDeclarator {
                                parameters: vec![
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(Int.into()).into()],
                                        declarator: Some(
                                            Declarator {
                                                kind: ident("__errnum"),
                                                derived: vec![],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![],
                                    }.into(),
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(Char.into()).into()],
                                        declarator: Some(
                                            Declarator {
                                                kind: ident("__buf"),
                                                derived: vec![Pointer(vec![]).into()],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![],
                                    }.into(),
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(TypedefName(ident("size_t")).into()).into()],
                                        declarator: Some(
                                            Declarator {
                                                kind: ident("__buflen"),
                                                derived: vec![],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![],
                                    }.into(),
                                ],
                                ellipsis: Ellipsis::None,
                            }.into(),
                        ],
                        extensions: vec![
                            AsmLabel(cstr(&[r#""""#, r#""__xpg_strerror_r""#])).into(),
                            Attribute {
                                name: "__nothrow__".into(),
                                arguments: vec![],
                            }.into(),
                            Attribute {
                                name: "__leaf__".into(),
                                arguments: vec![],
                            }.into(),
                            Attribute {
                                name: "__nonnull__".into(),
                                arguments: vec![cconst(int::dec("2"))],
                            }.into(),
                        ],
                    }.into(),
                    initializer: None,
                }.into(),
            ],
        }.into())
    );
}

#[test]
fn test_attribute2() {
    use self::expr::*;
    use self::int::dec;
    use ast::DeclarationSpecifier::{Extension, TypeQualifier, TypeSpecifier};
    use ast::DeclaratorKind::Abstract;
    use ast::DerivedDeclarator::Pointer;
    use ast::TypeQualifier::Const;
    use ast::TypeSpecifier::{Char, Void};
    use parser::declaration;

    assert_eq!(
        declaration(
            r#"__attribute__((noreturn)) void d0 (void),
                __attribute__((format(printf, 1, 2))) d1 (const char *, ...),
                 d2 (void);"#,
            &mut Env::new()
        ),
        Ok(Declaration {
            specifiers: vec![
                Extension(vec![
                    Attribute {
                        name: "noreturn".into(),
                        arguments: vec![],
                    }.into(),
                ]).into(),
                TypeSpecifier(Void.into()).into(),
            ],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("d0"),
                        derived: vec![
                            FunctionDeclarator {
                                parameters: vec![
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(Void.into()).into()],
                                        declarator: None,
                                        extensions: vec![],
                                    }.into(),
                                ],
                                ellipsis: Ellipsis::None,
                            }.into(),
                        ],
                        extensions: vec![],
                    }.into(),
                    initializer: None,
                }.into(),
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("d1"),
                        derived: vec![
                            FunctionDeclarator {
                                parameters: vec![
                                    ParameterDeclaration {
                                        specifiers: vec![
                                            TypeQualifier(Const.into()).into(),
                                            TypeSpecifier(Char.into()).into(),
                                        ],
                                        declarator: Some(
                                            Declarator {
                                                kind: Abstract.into(),
                                                derived: vec![Pointer(vec![]).into()],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![],
                                    }.into(),
                                ],
                                ellipsis: Ellipsis::Some,
                            }.into(),
                        ],
                        extensions: vec![
                            Attribute {
                                name: "format".into(),
                                arguments: vec![var("printf"), cconst(dec("1")), cconst(dec("2"))],
                            }.into(),
                        ],
                    }.into(),
                    initializer: None,
                }.into(),
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("d2"),
                        derived: vec![
                            FunctionDeclarator {
                                parameters: vec![
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(Void.into()).into()],
                                        declarator: None,
                                        extensions: vec![],
                                    }.into(),
                                ],
                                ellipsis: Ellipsis::None,
                            }.into(),
                        ],
                        extensions: vec![],
                    }.into(),
                    initializer: None,
                }.into(),
            ],
        }.into())
    );
}

#[test]
fn test_attribute3() {
    use ast::DeclarationSpecifier::{Extension, Function, StorageClass, TypeQualifier, TypeSpecifier};
    use ast::DerivedDeclarator::Pointer;
    use ast::FunctionSpecifier::Inline;
    use ast::Statement::Compound;
    use ast::StorageClassSpecifier::Extern;
    use ast::TypeQualifier::{Const, Restrict};
    use ast::TypeSpecifier::Char;
    use parser::translation_unit;

    assert_eq!(
        translation_unit(
            concat!(
                "extern __inline __attribute__ ((__always_inline__)) __attribute__ \n",
                "((__artificial__)) __attribute__ ((__warn_unused_result__)) char *\n",
                "__attribute__ ((__nothrow__ , __leaf__)) realpath (const char *__restrict\n",
                "__name, char *__restrict __resolved) {}"
            ),
            &mut Env::new()
        ),
        Ok(TranslationUnit(vec![
            ExternalDeclaration::FunctionDefinition(
                FunctionDefinition {
                    specifiers: vec![
                        StorageClass(Extern.into()).into(),
                        Function(Inline.into()).into(),
                        Extension(vec![
                            Attribute {
                                name: "__always_inline__".into(),
                                arguments: vec![],
                            }.into(),
                        ]).into(),
                        Extension(vec![
                            Attribute {
                                name: "__artificial__".into(),
                                arguments: vec![],
                            }.into(),
                        ]).into(),
                        Extension(vec![
                            Attribute {
                                name: "__warn_unused_result__".into(),
                                arguments: vec![],
                            }.into(),
                        ]).into(),
                        TypeSpecifier(Char.into()).into(),
                    ],
                    declarator: Declarator {
                        kind: ident("realpath"),
                        derived: vec![
                            Pointer(vec![
                                PointerQualifier::Extension(vec![
                                    Attribute {
                                        name: "__nothrow__".into(),
                                        arguments: vec![],
                                    }.into(),
                                    Attribute {
                                        name: "__leaf__".into(),
                                        arguments: vec![],
                                    }.into(),
                                ]).into(),
                            ]).into(),
                            DerivedDeclarator::Function(
                                FunctionDeclarator {
                                    parameters: vec![
                                        ParameterDeclaration {
                                            specifiers: vec![
                                                TypeQualifier(Const.into()).into(),
                                                TypeSpecifier(Char.into()).into(),
                                            ],
                                            declarator: Some(
                                                Declarator {
                                                    kind: ident("__name"),
                                                    derived: vec![
                                                        Pointer(vec![
                                                            PointerQualifier::TypeQualifier(Restrict.into()).into(),
                                                        ]).into(),
                                                    ],
                                                    extensions: vec![],
                                                }.into(),
                                            ),
                                            extensions: vec![],
                                        }.into(),
                                        ParameterDeclaration {
                                            specifiers: vec![TypeSpecifier(Char.into()).into()],
                                            declarator: Some(
                                                Declarator {
                                                    kind: ident("__resolved"),
                                                    derived: vec![
                                                        Pointer(vec![
                                                            PointerQualifier::TypeQualifier(Restrict.into()).into(),
                                                        ]).into(),
                                                    ],
                                                    extensions: vec![],
                                                }.into(),
                                            ),
                                            extensions: vec![],
                                        }.into(),
                                    ],
                                    ellipsis: Ellipsis::None,
                                }.into(),
                            ).into(),
                        ],
                        extensions: vec![],
                    }.into(),
                    declarations: vec![],
                    statement: Compound(vec![]).into(),
                }.into(),
            ).into(),
        ])).into()
    );
}

#[test]
fn test_alignof() {
    use ast::Expression::AlignOf;
    use ast::SpecifierQualifier::TypeSpecifier;
    use ast::TypeSpecifier::Long;
    use parser::expression;

    assert_eq!(
        expression("_Alignof(long long)", &mut Env::new()),
        Ok(AlignOf(
            TypeName {
                specifiers: vec![
                    TypeSpecifier(Long.into()).into(),
                    TypeSpecifier(Long.into()).into(),
                ],
                declarator: None,
            }.into(),
        ).into())
    );

    assert_eq!(
        expression("__alignof(long long)", &mut Env::new()),
        Ok(AlignOf(
            TypeName {
                specifiers: vec![
                    TypeSpecifier(Long.into()).into(),
                    TypeSpecifier(Long.into()).into(),
                ],
                declarator: None,
            }.into(),
        ).into())
    );

    assert_eq!(
        expression("__alignof__(long long)", &mut Env::new()),
        Ok(AlignOf(
            TypeName {
                specifiers: vec![
                    TypeSpecifier(Long.into()).into(),
                    TypeSpecifier(Long.into()).into(),
                ],
                declarator: None,
            }.into(),
        ).into())
    );
}

#[test]
fn test_stmt_expr() {
    use ast::DeclarationSpecifier::TypeSpecifier;
    use ast::Expression::Statement;
    use ast::Statement::{Compound, Expression};
    use ast::TypeSpecifier::Int;
    use parser::expression;

    use self::expr::*;
    use self::int::oct;

    assert_eq!(
        expression("({ int p = 0; p; })", &mut Env::new()),
        Ok(Statement(
            Compound(vec![
                BlockItem::Declaration(
                    Declaration {
                        specifiers: vec![TypeSpecifier(Int.into()).into()],
                        declarators: vec![
                            InitDeclarator {
                                declarator: Declarator {
                                    kind: ident("p"),
                                    derived: vec![],
                                    extensions: vec![],
                                }.into(),
                                initializer: Some(Initializer::Expression(cconst(oct("0"))).into()),
                            }.into(),
                        ],
                    }.into(),
                ).into(),
                BlockItem::Statement(Expression(Some(var("p"))).into()).into(),
            ]).into()
        ).into())
    );
}

#[test]
fn test_expr_cast() {
    use ast::Expression::Cast;
    use ast::SpecifierQualifier::TypeSpecifier;
    use ast::TypeName;
    use ast::TypeSpecifier::TypedefName;
    use parser::expression;

    use self::expr::*;

    let env = &mut Env::new();
    env.add_typename("U64");

    assert_eq!(
        expression("(U64)foo", env),
        Ok(Cast(
            CastExpression {
                type_name: TypeName {
                    specifiers: vec![TypeSpecifier(TypedefName(ident("U64")).into()).into()],
                    declarator: None,
                }.into(),
                expression: var("foo"),
            }.into()
        ).into())
    );
}

#[test]
fn test_directives() {
    use parser::translation_unit;

    assert_eq!(
        translation_unit(
            r#"# 1 "<stdin>"
# 1 "<built-in>"
# 1 "<command-line>"
# 31 "<command-line>"
# 1 "/usr/include/stdc-predef.h" 1 3 4
# 32 "<command-line>" 2
# 1 "<stdin>"
"#,
            &mut Env::new()
        ),
        Ok(TranslationUnit(vec![]))
    );
}

#[test]
fn test_gnu_asm() {
    use self::expr::var;
    use parser::statement;

    assert_eq!(
        statement(
            r#"__asm ("pmovmskb %1, %0" : "=r" (__m) : "x" (__x));"#,
            &mut Env::new()
        ),
        Ok(Statement::Asm(
            AsmStatement::GnuExtended(
                GnuExtendedAsmStatement {
                    qualifier: None,
                    template: cstr(&[r#""pmovmskb %1, %0""#]),
                    outputs: vec![
                        GnuAsmOperand {
                            symbolic_name: None,
                            constraints: cstr(&[r#""=r""#]),
                            variable_name: var("__m"),
                        }.into(),
                    ],
                    inputs: vec![
                        GnuAsmOperand {
                            symbolic_name: None,
                            constraints: cstr(&[r#""x""#]),
                            variable_name: var("__x"),
                        }.into(),
                    ],
                    clobbers: vec![],
                }.into()
            ).into(),
        ).into())
    );
}

#[test]
fn test_union() {
    use self::expr::*;
    use self::int::dec;
    use ast::ArraySize::VariableExpression;
    use ast::Designator::Member;
    use ast::Initializer::{Expression, List};
    use ast::SpecifierQualifier::TypeSpecifier;
    use ast::TypeSpecifier::{Double, Int, Long};
    use ast::TypeSpecifier::Struct;
    use parser::declaration;

    assert_eq!(
        declaration(
            "union { long double __l; int __i[3]; } __u = { __l: __x };",
            &mut Env::new()
        ),
        Ok(Declaration {
            specifiers: vec![
                DeclarationSpecifier::TypeSpecifier(
                    Struct(
                        StructType {
                            kind: StructKind::Union.into(),
                            identifier: None,
                            declarations: vec![
                                StructField {
                                    specifiers: vec![
                                        TypeSpecifier(Long.into()).into(),
                                        TypeSpecifier(Double.into()).into(),
                                    ],
                                    declarators: vec![
                                        StructDeclarator {
                                            declarator: Some(
                                                Declarator {
                                                    kind: ident("__l"),
                                                    derived: vec![],
                                                    extensions: vec![],
                                                }.into(),
                                            ),
                                            bit_width: None,
                                        }.into(),
                                    ],
                                }.into(),
                                StructField {
                                    specifiers: vec![TypeSpecifier(Int.into()).into()],
                                    declarators: vec![
                                        StructDeclarator {
                                            declarator: Some(
                                                Declarator {
                                                    kind: ident("__i"),
                                                    derived: vec![
                                                        ArrayDeclarator {
                                                            qualifiers: vec![],
                                                            size: VariableExpression(cconst(dec("3"))),
                                                        }.into(),
                                                    ],
                                                    extensions: vec![],
                                                }.into(),
                                            ),
                                            bit_width: None,
                                        }.into(),
                                    ],
                                }.into(),
                            ],
                        }.into(),
                    ).into(),
                ).into(),
            ],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("__u"),
                        derived: vec![],
                        extensions: vec![],
                    }.into(),
                    initializer: Some(
                        List(vec![
                            InitializerListItem {
                                designation: vec![Member(ident("__l")).into()],
                                initializer: Expression(var("__x")).into(),
                            }.into(),
                        ]).into(),
                    ),
                }.into(),
            ],
        }.into())
    );
}

#[test]
fn test_offsetof() {
    use self::expr::*;
    use self::int::dec;
    use ast::ArraySize::VariableExpression;
    use ast::Expression::OffsetOf;
    use ast::OffsetMember::IndirectMember;
    use ast::SpecifierQualifier::TypeSpecifier;
    use ast::TypeSpecifier::Int;
    use ast::TypeSpecifier::Struct;
    use parser::expression;

    assert_eq!(
        expression(
            "__builtin_offsetof(struct { struct { int b; } a[2]; }, a->b)",
            &mut Env::new()
        ),
        Ok(OffsetOf(
            OffsetOfExpression {
                type_name: TypeName {
                    specifiers: vec![
                        TypeSpecifier(
                            Struct(
                                StructType {
                                    kind: StructKind::Struct.into(),
                                    identifier: None,
                                    declarations: vec![
                                        StructField {
                                            specifiers: vec![
                                                TypeSpecifier(
                                                    Struct(
                                                        StructType {
                                                            kind: StructKind::Struct.into(),
                                                            identifier: None,
                                                            declarations: vec![
                                                                StructField {
                                                                    specifiers: vec![TypeSpecifier(Int.into()).into()],
                                                                    declarators: vec![
                                                                        StructDeclarator {
                                                                            declarator: Some(
                                                                                Declarator {
                                                                                    kind: ident("b"),
                                                                                    derived: vec![],
                                                                                    extensions: vec![],
                                                                                }.into(),
                                                                            ),
                                                                            bit_width: None,
                                                                        }.into(),
                                                                    ],
                                                                }.into(),
                                                            ],
                                                        }.into(),
                                                    ).into(),
                                                ).into(),
                                            ],
                                            declarators: vec![
                                                StructDeclarator {
                                                    declarator: Some(
                                                        Declarator {
                                                            kind: ident("a"),
                                                            derived: vec![
                                                                ArrayDeclarator {
                                                                    qualifiers: vec![],
                                                                    size: VariableExpression(cconst(dec("2"))),
                                                                }.into(),
                                                            ],
                                                            extensions: vec![],
                                                        }.into(),
                                                    ),
                                                    bit_width: None,
                                                }.into(),
                                            ],
                                        }.into(),
                                    ],
                                }.into(),
                            ).into(),
                        ).into(),
                    ],
                    declarator: None,
                }.into(),
                designator: OffsetDesignator {
                    base: ident("a"),
                    members: vec![IndirectMember(ident("b")).into()],
                }.into(),
            }.into()
        ).into())
    );
}

#[test]
fn test_call() {
    use self::expr::*;
    use ast::Expression::Call;
    use parser::expression;

    assert_eq!(
        expression("foo(bar, baz)", &mut Env::new()),
        Ok(Call(
            CallExpression {
                callee: var("foo"),
                arguments: vec![var("bar"), var("baz")],
            }.into()
        ).into())
    );
}

#[test]
fn test_typeof() {
    use self::expr::*;
    use ast::DeclarationSpecifier::TypeSpecifier;
    use ast::Expression::Call;
    use ast::TypeOf::Expression;
    use ast::TypeSpecifier::TypeOf;
    use parser::declaration;

    assert_eq!(
        declaration(
            "__typeof__(foo(bar, baz)) ook = foo(bar, baz);",
            &mut Env::new()
        ),
        Ok(Declaration {
            specifiers: vec![
                TypeSpecifier(
                    TypeOf(
                        Expression(
                            Call(
                                CallExpression {
                                    callee: var("foo"),
                                    arguments: vec![var("bar"), var("baz")],
                                }.into(),
                            ).into(),
                        ).into(),
                    ).into(),
                ).into(),
            ],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("ook"),
                        derived: vec![],
                        extensions: vec![],
                    }.into(),
                    initializer: Some(
                        Initializer::Expression(
                            Call(
                                CallExpression {
                                    callee: var("foo"),
                                    arguments: vec![var("bar"), var("baz")],
                                }.into(),
                            ).into(),
                        ).into(),
                    ),
                }.into(),
            ],
        }.into())
    );
}

#[test]
fn test_if() {
    use self::expr::var;
    use ast::Expression::Call;
    use ast::Statement::{Compound, DoWhile, If};
    use parser::statement;

    assert_eq!(
        statement("if (x) do {} while(y); else z();", &mut Env::new()),
        Ok(Box::new(
            If(IfStatement {
                condition: var("x"),
                then_statement: DoWhile(
                    DoWhileStatement {
                        statement: Compound(vec![]).into(),
                        expression: var("y"),
                    }.into()
                ).into(),
                else_statement: Some(
                    Statement::Expression(Some(
                        Call(
                            CallExpression {
                                callee: var("z"),
                                arguments: vec![],
                            }.into()
                        ).into()
                    )).into()
                ),
            }.into())
                .into(),
        ))
    );
}

// Check that a typedef that can be mistaken for a K&R-style argument declaration is correctly
// parsed as an external declaration. What went wrong: until we encounter bar, the thing looks like
// a function definition, where the name is followed by a two declarations K&R-style, similar to:
//
// ```
// int foo(i)
// int i; // <-- __attribute__ and typedef occupy this slot, since both are valid declarations.
// { }
// ```:
#[test]
fn test_attribute4() {
    use ast::DeclarationSpecifier::{StorageClass, TypeSpecifier};
    use ast::Statement::Compound;
    use ast::StorageClassSpecifier::Typedef;
    use ast::TypeSpecifier::Int;
    use parser::translation_unit;

    let env = &mut Env::new();

    assert_eq!(
        translation_unit(
            r#"
                int foo (int) __attribute__ ((__nothrow__));
                typedef int named;
                int bar (int f) { }
            "#,
            env
        ),
        Ok(TranslationUnit(vec![
            ExternalDeclaration::Declaration(
                Declaration {
                    specifiers: vec![TypeSpecifier(Int.into()).into()],
                    declarators: vec![
                        InitDeclarator {
                            declarator: Declarator {
                                kind: ident("foo"),
                                derived: vec![
                                    FunctionDeclarator {
                                        parameters: vec![
                                            ParameterDeclaration {
                                                specifiers: vec![TypeSpecifier(Int.into()).into()],
                                                declarator: None,
                                                extensions: vec![],
                                            }.into(),
                                        ],
                                        ellipsis: Ellipsis::None,
                                    }.into(),
                                ],
                                extensions: vec![
                                    Attribute {
                                        name: "__nothrow__".into(),
                                        arguments: vec![],
                                    }.into(),
                                ],
                            }.into(),
                            initializer: None,
                        }.into(),
                    ],
                }.into(),
            ).into(),
            ExternalDeclaration::Declaration(
                Declaration {
                    specifiers: vec![
                        StorageClass(Typedef.into()).into(),
                        TypeSpecifier(Int.into()).into(),
                    ],
                    declarators: vec![
                        InitDeclarator {
                            declarator: Declarator {
                                kind: ident("named"),
                                derived: vec![],
                                extensions: vec![],
                            }.into(),
                            initializer: None,
                        }.into(),
                    ],
                }.into(),
            ).into(),
            ExternalDeclaration::FunctionDefinition(
                FunctionDefinition {
                    specifiers: vec![TypeSpecifier(Int.into()).into()],
                    declarator: Declarator {
                        kind: ident("bar"),
                        derived: vec![
                            FunctionDeclarator {
                                parameters: vec![
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(Int.into()).into()],
                                        declarator: Some(
                                            Declarator {
                                                kind: ident("f"),
                                                derived: vec![],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![],
                                    }.into(),
                                ],
                                ellipsis: Ellipsis::None,
                            }.into(),
                        ],
                        extensions: vec![],
                    }.into(),
                    declarations: vec![],
                    statement: Compound(vec![]).into(),
                }.into(),
            ).into(),
        ]))
    );
}

#[test]
fn test_attribute5() {
    use ast::DeclarationSpecifier::TypeSpecifier;
    use ast::Statement::Compound;
    use ast::TypeSpecifier::Int;
    use parser::translation_unit;

    assert_eq!(
        translation_unit(
            "int foo(int a __attribute__((unused)), int b __attribute__((unused))) {}",
            &mut Env::new(),
        ),
        Ok(TranslationUnit(vec![
            ExternalDeclaration::FunctionDefinition(
                FunctionDefinition {
                    specifiers: vec![TypeSpecifier(Int.into()).into()],
                    declarator: Declarator {
                        kind: ident("foo"),
                        derived: vec![
                            FunctionDeclarator {
                                parameters: vec![
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(Int.into()).into()],
                                        declarator: Some(
                                            Declarator {
                                                kind: ident("a"),
                                                derived: vec![],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![
                                            Attribute {
                                                name: "unused".into(),
                                                arguments: vec![],
                                            }.into(),
                                        ],
                                    }.into(),
                                    ParameterDeclaration {
                                        specifiers: vec![TypeSpecifier(Int.into()).into()],
                                        declarator: Some(
                                            Declarator {
                                                kind: ident("b"),
                                                derived: vec![],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        extensions: vec![
                                            Attribute {
                                                name: "unused".into(),
                                                arguments: vec![],
                                            }.into(),
                                        ],
                                    }.into(),
                                ],
                                ellipsis: Ellipsis::None,
                            }.into(),
                        ],
                        extensions: vec![],
                    }.into(),
                    declarations: vec![],
                    statement: Compound(vec![]).into(),
                }.into(),
            ).into(),
        ]))
    );
}

#[test]
fn test_declaration6() {
    use ast::Expression::AlignOf;
    use ast::SpecifierQualifier::TypeSpecifier;
    use ast::StorageClassSpecifier::Typedef;
    use ast::TypeSpecifier::{Double, Long, Struct};
    use parser::declaration;

    assert_eq!(
        declaration(
            r"typedef struct {
              long long __max_align_ll __attribute__((__aligned__(__alignof__(long long))));
              long double __max_align_ld __attribute__((__aligned__(__alignof__(long double))));
            } max_align_t;",
            &mut Env::new()
        ),
        Ok(Declaration {
            specifiers: vec![
                DeclarationSpecifier::StorageClass(Typedef.into()).into(),
                DeclarationSpecifier::TypeSpecifier(
                    Struct(
                        StructType {
                            kind: StructKind::Struct.into(),
                            identifier: None,
                            declarations: vec![
                                StructField {
                                    specifiers: vec![
                                        TypeSpecifier(Long.into()).into(),
                                        TypeSpecifier(Long.into()).into(),
                                    ],
                                    declarators: vec![
                                        StructDeclarator {
                                            declarator: Some(
                                                Declarator {
                                                    kind: ident("__max_align_ll"),
                                                    derived: vec![],
                                                    extensions: vec![
                                                        Attribute {
                                                            name: "__aligned__".into(),
                                                            arguments: vec![
                                                                AlignOf(
                                                                    TypeName {
                                                                        specifiers: vec![
                                                                            TypeSpecifier(Long.into()).into(),
                                                                            TypeSpecifier(Long.into()).into(),
                                                                        ],
                                                                        declarator: None,
                                                                    }.into(),
                                                                ).into(),
                                                            ],
                                                        }.into(),
                                                    ],
                                                }.into(),
                                            ),
                                            bit_width: None,
                                        }.into(),
                                    ],
                                }.into(),
                                StructField {
                                    specifiers: vec![
                                        TypeSpecifier(Long.into()).into(),
                                        TypeSpecifier(Double.into()).into(),
                                    ],
                                    declarators: vec![
                                        StructDeclarator {
                                            declarator: Some(
                                                Declarator {
                                                    kind: ident("__max_align_ld"),
                                                    derived: vec![],
                                                    extensions: vec![
                                                        Attribute {
                                                            name: "__aligned__".into(),
                                                            arguments: vec![
                                                                AlignOf(
                                                                    TypeName {
                                                                        specifiers: vec![
                                                                            TypeSpecifier(Long.into()).into(),
                                                                            TypeSpecifier(Double.into()).into(),
                                                                        ],
                                                                        declarator: None,
                                                                    }.into(),
                                                                ).into(),
                                                            ],
                                                        }.into(),
                                                    ],
                                                }.into(),
                                            ),
                                            bit_width: None,
                                        }.into(),
                                    ],
                                }.into(),
                            ],
                        }.into(),
                    ).into(),
                ).into(),
            ],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: ident("max_align_t"),
                        derived: vec![],
                        extensions: vec![],
                    }.into(),
                    initializer: None,
                }.into(),
            ],
        }.into())
    );
}

#[test]
fn test_keyword_expr() {
    use parser::expression;

    assert_eq!(
        expression("__func__", &mut Env::new()),
        Ok(Expression::Identifier(ident("__func__")).into())
    );

    assert_eq!(
        expression("__FUNCTION__", &mut Env::new()),
        Ok(Expression::Identifier(ident("__FUNCTION__")).into())
    );

    assert_eq!(
        expression("__PRETTY_FUNCTION__", &mut Env::new()),
        Ok(Expression::Identifier(ident("__PRETTY_FUNCTION__")).into())
    );
}

#[test]
fn test_ts18661_float() {
    use parser::declaration;
    assert_eq!(
        declaration("_Float64 foo = 1.5f64;", &mut Env::new()),
        Ok(Declaration {
            specifiers: vec![
                TS18661FloatType {
                    format: TS18661FloatFormat::BinaryInterchange,
                    width: 64,
                }.into(),
            ],
            declarators: vec![
                InitDeclarator {
                    declarator: Declarator {
                        kind: Identifier { name: "foo".into() }.into(),
                        derived: vec![],
                        extensions: vec![],
                    }.into(),
                    initializer: Some(float::dec("1.5f64").into()),
                }.into(),
            ],
        }.into())
    );
}

#[test]
fn test_gnu_extension() {
    use parser::translation_unit;
    assert_eq!(
        translation_unit("__extension__ union { long l; };", &mut Env::with_gnu(true)),
        Ok(TranslationUnit(vec![
            Declaration {
                specifiers: vec![
                    StructType {
                        kind: StructKind::Union.into(),
                        identifier: None,
                        declarations: vec![
                            StructField {
                                specifiers: vec![TypeSpecifier::Long.into()],
                                declarators: vec![
                                    StructDeclarator {
                                        declarator: Some(
                                            Declarator {
                                                kind: ident("l"),
                                                derived: vec![],
                                                extensions: vec![],
                                            }.into(),
                                        ),
                                        bit_width: None,
                                    }.into(),
                                ],
                            }.into(),
                        ],
                    }.into(),
                ],
                declarators: vec![],
            }.into(),
        ]).into()),
    );

    assert_eq!(
        translation_unit(r#"__extension__ _Static_assert(1,"ERR");"#, &mut Env::new()),
        Ok(TranslationUnit(vec![
            StaticAssert {
                expression: int::dec("1").into(),
                message: cstr(&[r#""ERR""#]),
            }.into(),
        ]).into()),
    );
}
