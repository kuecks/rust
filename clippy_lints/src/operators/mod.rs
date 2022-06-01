use rustc_hir::{Body, Expr, ExprKind, UnOp};
use rustc_lint::{LateContext, LateLintPass};
use rustc_session::{declare_tool_lint, impl_lint_pass};

mod absurd_extreme_comparisons;
mod assign_op_pattern;
mod bit_mask;
mod double_comparison;
mod misrefactored_assign_op;
mod numeric_arithmetic;
mod verbose_bit_mask;

declare_clippy_lint! {
    /// ### What it does
    /// Checks for comparisons where one side of the relation is
    /// either the minimum or maximum value for its type and warns if it involves a
    /// case that is always true or always false. Only integer and boolean types are
    /// checked.
    ///
    /// ### Why is this bad?
    /// An expression like `min <= x` may misleadingly imply
    /// that it is possible for `x` to be less than the minimum. Expressions like
    /// `max < x` are probably mistakes.
    ///
    /// ### Known problems
    /// For `usize` the size of the current compile target will
    /// be assumed (e.g., 64 bits on 64 bit systems). This means code that uses such
    /// a comparison to detect target pointer width will trigger this lint. One can
    /// use `mem::sizeof` and compare its value or conditional compilation
    /// attributes
    /// like `#[cfg(target_pointer_width = "64")] ..` instead.
    ///
    /// ### Example
    /// ```rust
    /// let vec: Vec<isize> = Vec::new();
    /// if vec.len() <= 0 {}
    /// if 100 > i32::MAX {}
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub ABSURD_EXTREME_COMPARISONS,
    correctness,
    "a comparison with a maximum or minimum value that is always true or false"
}

declare_clippy_lint! {
    /// ### What it does
    /// Checks for integer arithmetic operations which could overflow or panic.
    ///
    /// Specifically, checks for any operators (`+`, `-`, `*`, `<<`, etc) which are capable
    /// of overflowing according to the [Rust
    /// Reference](https://doc.rust-lang.org/reference/expressions/operator-expr.html#overflow),
    /// or which can panic (`/`, `%`). No bounds analysis or sophisticated reasoning is
    /// attempted.
    ///
    /// ### Why is this bad?
    /// Integer overflow will trigger a panic in debug builds or will wrap in
    /// release mode. Division by zero will cause a panic in either mode. In some applications one
    /// wants explicitly checked, wrapping or saturating arithmetic.
    ///
    /// ### Example
    /// ```rust
    /// # let a = 0;
    /// a + 1;
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub INTEGER_ARITHMETIC,
    restriction,
    "any integer arithmetic expression which could overflow or panic"
}

declare_clippy_lint! {
    /// ### What it does
    /// Checks for float arithmetic.
    ///
    /// ### Why is this bad?
    /// For some embedded systems or kernel development, it
    /// can be useful to rule out floating-point numbers.
    ///
    /// ### Example
    /// ```rust
    /// # let a = 0.0;
    /// a + 1.0;
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub FLOAT_ARITHMETIC,
    restriction,
    "any floating-point arithmetic statement"
}

declare_clippy_lint! {
    /// ### What it does
    /// Checks for `a = a op b` or `a = b commutative_op a`
    /// patterns.
    ///
    /// ### Why is this bad?
    /// These can be written as the shorter `a op= b`.
    ///
    /// ### Known problems
    /// While forbidden by the spec, `OpAssign` traits may have
    /// implementations that differ from the regular `Op` impl.
    ///
    /// ### Example
    /// ```rust
    /// let mut a = 5;
    /// let b = 0;
    /// // ...
    ///
    /// a = a + b;
    /// ```
    ///
    /// Use instead:
    /// ```rust
    /// let mut a = 5;
    /// let b = 0;
    /// // ...
    ///
    /// a += b;
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub ASSIGN_OP_PATTERN,
    style,
    "assigning the result of an operation on a variable to that same variable"
}

declare_clippy_lint! {
    /// ### What it does
    /// Checks for `a op= a op b` or `a op= b op a` patterns.
    ///
    /// ### Why is this bad?
    /// Most likely these are bugs where one meant to write `a
    /// op= b`.
    ///
    /// ### Known problems
    /// Clippy cannot know for sure if `a op= a op b` should have
    /// been `a = a op a op b` or `a = a op b`/`a op= b`. Therefore, it suggests both.
    /// If `a op= a op b` is really the correct behavior it should be
    /// written as `a = a op a op b` as it's less confusing.
    ///
    /// ### Example
    /// ```rust
    /// let mut a = 5;
    /// let b = 2;
    /// // ...
    /// a += a + b;
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub MISREFACTORED_ASSIGN_OP,
    suspicious,
    "having a variable on both sides of an assign op"
}

declare_clippy_lint! {
    /// ### What it does
    /// Checks for incompatible bit masks in comparisons.
    ///
    /// The formula for detecting if an expression of the type `_ <bit_op> m
    /// <cmp_op> c` (where `<bit_op>` is one of {`&`, `|`} and `<cmp_op>` is one of
    /// {`!=`, `>=`, `>`, `!=`, `>=`, `>`}) can be determined from the following
    /// table:
    ///
    /// |Comparison  |Bit Op|Example      |is always|Formula               |
    /// |------------|------|-------------|---------|----------------------|
    /// |`==` or `!=`| `&`  |`x & 2 == 3` |`false`  |`c & m != c`          |
    /// |`<`  or `>=`| `&`  |`x & 2 < 3`  |`true`   |`m < c`               |
    /// |`>`  or `<=`| `&`  |`x & 1 > 1`  |`false`  |`m <= c`              |
    /// |`==` or `!=`| `\|` |`x \| 1 == 0`|`false`  |`c \| m != c`         |
    /// |`<`  or `>=`| `\|` |`x \| 1 < 1` |`false`  |`m >= c`              |
    /// |`<=` or `>` | `\|` |`x \| 1 > 0` |`true`   |`m > c`               |
    ///
    /// ### Why is this bad?
    /// If the bits that the comparison cares about are always
    /// set to zero or one by the bit mask, the comparison is constant `true` or
    /// `false` (depending on mask, compared value, and operators).
    ///
    /// So the code is actively misleading, and the only reason someone would write
    /// this intentionally is to win an underhanded Rust contest or create a
    /// test-case for this lint.
    ///
    /// ### Example
    /// ```rust
    /// # let x = 1;
    /// if (x & 1 == 2) { }
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub BAD_BIT_MASK,
    correctness,
    "expressions of the form `_ & mask == select` that will only ever return `true` or `false`"
}

declare_clippy_lint! {
    /// ### What it does
    /// Checks for bit masks in comparisons which can be removed
    /// without changing the outcome. The basic structure can be seen in the
    /// following table:
    ///
    /// |Comparison| Bit Op   |Example     |equals |
    /// |----------|----------|------------|-------|
    /// |`>` / `<=`|`\|` / `^`|`x \| 2 > 3`|`x > 3`|
    /// |`<` / `>=`|`\|` / `^`|`x ^ 1 < 4` |`x < 4`|
    ///
    /// ### Why is this bad?
    /// Not equally evil as [`bad_bit_mask`](#bad_bit_mask),
    /// but still a bit misleading, because the bit mask is ineffective.
    ///
    /// ### Known problems
    /// False negatives: This lint will only match instances
    /// where we have figured out the math (which is for a power-of-two compared
    /// value). This means things like `x | 1 >= 7` (which would be better written
    /// as `x >= 6`) will not be reported (but bit masks like this are fairly
    /// uncommon).
    ///
    /// ### Example
    /// ```rust
    /// # let x = 1;
    /// if (x | 1 > 3) {  }
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub INEFFECTIVE_BIT_MASK,
    correctness,
    "expressions where a bit mask will be rendered useless by a comparison, e.g., `(x | 1) > 2`"
}

declare_clippy_lint! {
    /// ### What it does
    /// Checks for bit masks that can be replaced by a call
    /// to `trailing_zeros`
    ///
    /// ### Why is this bad?
    /// `x.trailing_zeros() > 4` is much clearer than `x & 15
    /// == 0`
    ///
    /// ### Known problems
    /// llvm generates better code for `x & 15 == 0` on x86
    ///
    /// ### Example
    /// ```rust
    /// # let x = 1;
    /// if x & 0b1111 == 0 { }
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub VERBOSE_BIT_MASK,
    pedantic,
    "expressions where a bit mask is less readable than the corresponding method call"
}

declare_clippy_lint! {
    /// ### What it does
    /// Checks for double comparisons that could be simplified to a single expression.
    ///
    ///
    /// ### Why is this bad?
    /// Readability.
    ///
    /// ### Example
    /// ```rust
    /// # let x = 1;
    /// # let y = 2;
    /// if x == y || x < y {}
    /// ```
    ///
    /// Use instead:
    ///
    /// ```rust
    /// # let x = 1;
    /// # let y = 2;
    /// if x <= y {}
    /// ```
    #[clippy::version = "pre 1.29.0"]
    pub DOUBLE_COMPARISONS,
    complexity,
    "unnecessary double comparisons that can be simplified"
}

pub struct Operators {
    arithmetic_context: numeric_arithmetic::Context,
    verbose_bit_mask_threshold: u64,
}
impl_lint_pass!(Operators => [
    ABSURD_EXTREME_COMPARISONS,
    INTEGER_ARITHMETIC,
    FLOAT_ARITHMETIC,
    ASSIGN_OP_PATTERN,
    MISREFACTORED_ASSIGN_OP,
    BAD_BIT_MASK,
    INEFFECTIVE_BIT_MASK,
    VERBOSE_BIT_MASK,
    DOUBLE_COMPARISONS,
]);
impl Operators {
    pub fn new(verbose_bit_mask_threshold: u64) -> Self {
        Self {
            arithmetic_context: numeric_arithmetic::Context::default(),
            verbose_bit_mask_threshold,
        }
    }
}
impl<'tcx> LateLintPass<'tcx> for Operators {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, e: &'tcx Expr<'_>) {
        match e.kind {
            ExprKind::Binary(op, lhs, rhs) => {
                if !e.span.from_expansion() {
                    absurd_extreme_comparisons::check(cx, e, op.node, lhs, rhs);
                }
                self.arithmetic_context.check_binary(cx, e, op.node, lhs, rhs);
                bit_mask::check(cx, e, op.node, lhs, rhs);
                verbose_bit_mask::check(cx, e, op.node, lhs, rhs, self.verbose_bit_mask_threshold);
                double_comparison::check(cx, op.node, lhs, rhs, e.span);
            },
            ExprKind::AssignOp(op, lhs, rhs) => {
                self.arithmetic_context.check_binary(cx, e, op.node, lhs, rhs);
                misrefactored_assign_op::check(cx, e, op.node, lhs, rhs);
            },
            ExprKind::Assign(lhs, rhs, _) => {
                assign_op_pattern::check(cx, e, lhs, rhs);
            },
            ExprKind::Unary(op, arg) => {
                if op == UnOp::Neg {
                    self.arithmetic_context.check_negate(cx, e, arg);
                }
            },
            _ => (),
        }
    }

    fn check_expr_post(&mut self, _: &LateContext<'_>, e: &Expr<'_>) {
        self.arithmetic_context.expr_post(e.hir_id);
    }

    fn check_body(&mut self, cx: &LateContext<'tcx>, b: &'tcx Body<'_>) {
        self.arithmetic_context.enter_body(cx, b);
    }

    fn check_body_post(&mut self, cx: &LateContext<'tcx>, b: &'tcx Body<'_>) {
        self.arithmetic_context.body_post(cx, b);
    }
}
