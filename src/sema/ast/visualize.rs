use super::*;
use crate::op::*;
use crate::visualize::*;
// Implementation for individual expression types
impl Visualize for Binary {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Binary", &format!("{:?}", self.op), indent, is_last, prefix);
        let new_prefix = extend_prefix(prefix, !is_last);

        print_branch("Left", "", indent + 1, false, &new_prefix);
        self.lhs
            .visualize_with_context(indent + 2, true, &extend_prefix(&new_prefix, true));

        print_branch("Right", "", indent + 1, true, &new_prefix);
        self.rhs
            .visualize_with_context(indent + 2, true, &extend_prefix(&new_prefix, false));
    }
}

impl Visualize for Unary {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Unary", &format!("{:?}", self.op), indent, is_last, prefix);
        print_branch(
            "Operand",
            "",
            indent + 1,
            true,
            &extend_prefix(prefix, !is_last),
        );
        self.expr.visualize_with_context(
            indent + 2,
            true,
            &extend_prefix(&extend_prefix(prefix, !is_last), false),
        );
    }
}

impl Visualize for Call {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("FunctionCall", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch(
            "Function",
            "",
            indent + 1,
            self.args.is_empty(),
            &next_prefix,
        );
        self.func.visualize_with_context(
            indent + 2,
            true,
            &extend_prefix(&next_prefix, !self.args.is_empty()),
        );

        if !self.args.is_empty() {
            print_branch("Arguments", "", indent + 1, true, &next_prefix);
            for (i, arg) in self.args.iter().enumerate() {
                let is_last_arg = i == self.args.len() - 1;
                arg.visualize_with_context(
                    indent + 2,
                    is_last_arg,
                    &extend_prefix(&extend_prefix(&next_prefix, false), false),
                );
            }
        }
    }
}

impl Visualize for Assign {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch(
            "Assignment",
            &format!("{:?}", self.op),
            indent,
            is_last,
            prefix,
        );
        let new_prefix = extend_prefix(prefix, !is_last);

        print_branch("Left", "", indent + 1, false, &new_prefix);
        self.lhs
            .visualize_with_context(indent + 2, true, &extend_prefix(&new_prefix, true));

        print_branch("Right", "", indent + 1, true, &new_prefix);
        self.rhs
            .visualize_with_context(indent + 2, true, &extend_prefix(&new_prefix, false));
    }
}

impl Visualize for Ternary {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Ternary", "", indent, is_last, prefix);
        let new_prefix = extend_prefix(prefix, !is_last);

        print_branch("Condition", "", indent + 1, false, &new_prefix);
        self.cond
            .visualize_with_context(indent + 2, true, &extend_prefix(&new_prefix, true));

        print_branch("Then", "", indent + 1, false, &new_prefix);
        self.then_branch.visualize_with_context(
            indent + 2,
            true,
            &extend_prefix(&new_prefix, true),
        );

        print_branch("Else", "", indent + 1, true, &new_prefix);
        self.else_branch.visualize_with_context(
            indent + 2,
            true,
            &extend_prefix(&new_prefix, false),
        );
    }
}

impl Visualize for Subscript {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("ArrayAccess", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch("Array", "", indent + 1, false, &next_prefix);
        self.subject
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, true));

        print_branch("Index", "", indent + 1, true, &next_prefix);
        self.index
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, false));
    }
}

impl Visualize for MemberAccess {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch(
            "MemberAccess",
            &format!("{:?}", self.kind),
            indent,
            is_last,
            prefix,
        );
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch("Base", "", indent + 1, false, &next_prefix);
        self.base
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, true));

        print_branch("Member", &self.member.name, indent + 1, true, &next_prefix);
    }
}

impl Visualize for Sizeof {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Sizeof", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        match self {
            Sizeof::Type(ty) => {
                print_branch("Type", &ty.to_rust_format(), indent + 1, true, &next_prefix);
            }
            Sizeof::TypedExpr(expr) => {
                print_branch("Expression", "", indent + 1, true, &next_prefix);
                expr.visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, false));
            }
        }
    }
}

impl Visualize for Cast {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Cast", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch(
            "Type",
            &self.r#type.to_rust_format(),
            indent + 1,
            false,
            &next_prefix,
        );
        print_branch("Expression", "", indent + 1, true, &next_prefix);
        self.expr
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, false));
    }
}

impl Visualize for Comma {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Comma", "", indent, is_last, prefix);
        let new_prefix = extend_prefix(prefix, !is_last);

        for (i, expr) in self.assigns.iter().enumerate() {
            let is_last_expr = i == self.assigns.len() - 1;
            expr.visualize_with_context(indent + 1, is_last_expr, &new_prefix);
        }
    }
}

impl Visualize for Ident {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Identifier", &self.name, indent, is_last, prefix);
    }
}

// Implementation for Expr that delegates to individual types
impl Visualize for SemaExpr {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        match self {
            SemaExpr::NumInt(n) => {
                print_branch("Int_Number", &n.to_string(), indent, is_last, prefix);
            }
            SemaExpr::NumFloat(n) => {
                print_branch("Float_Number", &n.to_string(), indent, is_last, prefix);
            }
            SemaExpr::Char(c) => {
                print_branch("Character", &format!("'{}'", c), indent, is_last, prefix);
            }
            SemaExpr::String(chars) => {
                let s: String = chars.iter().collect();
                print_branch("String", &format!("\"{:?}\"", s), indent, is_last, prefix);
            }
            SemaExpr::Symbol(ident) => {
                ident.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::Binary(binary) => {
                binary.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::Unary(unary) => {
                unary.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::Call(call) => {
                call.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::Assign(assign) => {
                assign.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::Ternary(ternary) => {
                ternary.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::Subscript(subscript) => {
                subscript.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::MemberAccess(member_access) => {
                member_access.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::Sizeof(sizeof) => {
                sizeof.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::Cast(cast) => {
                cast.visualize_with_context(indent, is_last, prefix);
            }
            SemaExpr::Comma(comma) => {
                comma.visualize_with_context(indent, is_last, prefix);
            }
        }
    }
}

impl Visualize for Symbol {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        self.ident.visualize_with_context(indent, is_last, prefix);
    }
}

impl Visualize for TypedExpr {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch(
            "TypedExpr",
            &format!("Type: {}", self.r#type.to_rust_format()),
            indent,
            is_last,
            prefix,
        );

        // Update prefix for child node - !is_last because we have a child
        let new_prefix = extend_prefix(prefix, !is_last);

        self.r#expr
            .visualize_with_context(indent + 1, true, &new_prefix);
    }
}
// Implementation for Program and TopLevel
impl Visualize for Program {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Program", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        for (i, item) in self.items.iter().enumerate() {
            let is_last_item = i == self.items.len() - 1;
            item.visualize_with_context(indent + 1, is_last_item, &next_prefix);
        }
    }
}

impl Visualize for TopLevel {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        match self {
            TopLevel::FunctionDef(func) => {
                func.visualize_with_context(indent, is_last, prefix);
            }
            TopLevel::FunctionProto(proto) => {
                proto.visualize_with_context(indent, is_last, prefix);
            }
            TopLevel::Stmt(stmt) => {
                stmt.visualize_with_context(indent, is_last, prefix);
            }
        }
    }
}

impl Visualize for FunctionProto {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch(
            "FunctionProto",
            &self.sig.ident.name,
            indent,
            is_last,
            prefix,
        );
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch(
            "Type",
            &self.sig.ty.to_rust_format(),
            indent + 1,
            true,
            &next_prefix,
        );
    }
}

impl Visualize for FunctionDef {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("FunctionDef", &self.sig.ident.name, indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        let has_body = !self.body.statements.is_empty();
        let has_params = !self.param_names.is_empty();
        let mut remaining_items = 1; // Type is always present
        if has_params {
            remaining_items += 1;
        }
        if has_body {
            remaining_items += 1;
        }

        // Type
        remaining_items -= 1;
        print_branch(
            "Type",
            &self.sig.ty.to_rust_format(),
            indent + 1,
            remaining_items == 0,
            &next_prefix,
        );

        // Parameters
        if has_params {
            remaining_items -= 1;
            print_branch("Params", "", indent + 1, remaining_items == 0, &next_prefix);
            let param_prefix = extend_prefix(&next_prefix, remaining_items > 0);
            for (i, param) in self.param_names.iter().enumerate() {
                let is_last_param = i == self.param_names.len() - 1;
                print_branch(
                    "Param",
                    &param.ident.name,
                    indent + 2,
                    is_last_param,
                    &param_prefix,
                );
            }
        }

        // Body
        if has_body {
            print_branch("Body", "", indent + 1, true, &next_prefix);
            self.body
                .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, false));
        }
    }
}

// Implementation for statements
impl Visualize for Stmt {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        match self {
            Stmt::ExprStmt(expr) => {
                print_branch("ExprStmt", "", indent, is_last, prefix);
                expr.visualize_with_context(indent + 1, true, &extend_prefix(prefix, !is_last));
            }
            Stmt::DeclStmt(decl_stmt) => {
                decl_stmt.visualize_with_context(indent, is_last, prefix);
            }
            Stmt::Control(control) => {
                control.visualize_with_context(indent, is_last, prefix);
            }
            Stmt::Return(ret) => {
                if let Some(expr) = &ret.value {
                    print_branch("Return", "", indent, is_last, prefix);
                    expr.visualize_with_context(indent + 1, true, &extend_prefix(prefix, !is_last));
                } else {
                    print_branch("Return", "(void)", indent, is_last, prefix);
                }
            }
            Stmt::Goto(goto) => {
                print_branch(
                    "Goto",
                    &format!("→ {}", goto.label.name),
                    indent,
                    is_last,
                    prefix,
                );
            }
            Stmt::Label(label) => {
                print_branch(
                    "Label",
                    &format!("{}:", label.name.name),
                    indent,
                    is_last,
                    prefix,
                );
                label.stmt.visualize_with_context(
                    indent + 1,
                    true,
                    &extend_prefix(prefix, !is_last),
                );
            }
            Stmt::Block(block) => {
                block.visualize_with_context(indent, is_last, prefix);
            }
            Stmt::Break => {
                print_branch("Break", "", indent, is_last, prefix);
            }
            Stmt::Continue => {
                print_branch("Continue", "", indent, is_last, prefix);
            }
        }
    }
}

impl Visualize for Block {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        if self.statements.is_empty() {
            print_branch("Block", "(empty)", indent, is_last, prefix);
        } else {
            print_branch("Block", "", indent, is_last, prefix);
            let next_prefix = extend_prefix(prefix, !is_last);

            for (i, stmt) in self.statements.iter().enumerate() {
                let is_last_stmt = i == self.statements.len() - 1;
                stmt.visualize_with_context(indent + 1, is_last_stmt, &next_prefix);
            }
        }
    }
}

impl Visualize for Control {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        match self {
            Control::If(if_stmt) => {
                if_stmt.visualize_with_context(indent, is_last, prefix);
            }
            Control::While(while_stmt) => {
                while_stmt.visualize_with_context(indent, is_last, prefix);
            }
            Control::DoWhile(do_while_stmt) => {
                do_while_stmt.visualize_with_context(indent, is_last, prefix);
            }
            Control::For(for_stmt) => {
                for_stmt.visualize_with_context(indent, is_last, prefix);
            }
            Control::Switch(switch_stmt) => {
                switch_stmt.visualize_with_context(indent, is_last, prefix);
            }
        }
    }
}

impl Visualize for If {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("If", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch("Condition", "", indent + 1, false, &next_prefix);
        self.cond
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, true));

        print_branch(
            "Then",
            "",
            indent + 1,
            self.else_branch.is_none(),
            &next_prefix,
        );
        self.then_branch.visualize_with_context(
            indent + 2,
            true,
            &extend_prefix(&next_prefix, self.else_branch.is_some()),
        );

        if let Some(else_branch) = &self.else_branch {
            print_branch("Else", "", indent + 1, true, &next_prefix);
            else_branch.visualize_with_context(
                indent + 2,
                true,
                &extend_prefix(&next_prefix, false),
            );
        }
    }
}

impl Visualize for While {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("While", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch("Condition", "", indent + 1, false, &next_prefix);
        self.cond
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, true));

        print_branch("Body", "", indent + 1, true, &next_prefix);
        self.body
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, false));
    }
}

impl Visualize for DoWhile {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("DoWhile", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch("Body", "", indent + 1, false, &next_prefix);
        self.body
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, true));

        print_branch("Condition", "", indent + 1, true, &next_prefix);
        self.cond
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, false));
    }
}

impl Visualize for For {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("For", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        let has_init = self.init.is_some();
        let has_cond = self.cond.is_some();
        let has_step = self.step.is_some();
        let mut remaining_items = 1; // body is always present
        if has_init {
            remaining_items += 1;
        }
        if has_cond {
            remaining_items += 1;
        }
        if has_step {
            remaining_items += 1;
        }

        if let Some(init) = &self.init {
            remaining_items -= 1;
            print_branch("Init", "", indent + 1, remaining_items == 0, &next_prefix);
            init.visualize_with_context(
                indent + 2,
                true,
                &extend_prefix(&next_prefix, remaining_items > 0),
            );
        }

        if let Some(cond) = &self.cond {
            remaining_items -= 1;
            print_branch(
                "Condition",
                "",
                indent + 1,
                remaining_items == 0,
                &next_prefix,
            );
            cond.visualize_with_context(
                indent + 2,
                true,
                &extend_prefix(&next_prefix, remaining_items > 0),
            );
        }

        if let Some(step) = &self.step {
            remaining_items -= 1;
            print_branch("Step", "", indent + 1, remaining_items == 0, &next_prefix);
            step.visualize_with_context(
                indent + 2,
                true,
                &extend_prefix(&next_prefix, remaining_items > 0),
            );
        }

        print_branch("Body", "", indent + 1, true, &next_prefix);
        self.body
            .visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, false));
    }
}

impl Visualize for Switch {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Switch", "", indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch(
            "Condition",
            "",
            indent + 1,
            self.cases.is_empty(),
            &next_prefix,
        );
        self.cond.visualize_with_context(
            indent + 2,
            true,
            &extend_prefix(&next_prefix, !self.cases.is_empty()),
        );

        if !self.cases.is_empty() {
            print_branch("Cases", "", indent + 1, true, &next_prefix);
            let case_prefix = extend_prefix(&next_prefix, false);
            for (i, case) in self.cases.iter().enumerate() {
                let is_last_case = i == self.cases.len() - 1;
                case.visualize_with_context(indent + 2, is_last_case, &case_prefix);
            }
        }
    }
}

impl Visualize for SwitchCase {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        match self {
            SwitchCase::Case(case_stmt) => {
                print_branch("Case", "", indent, is_last, prefix);
                let next_prefix = extend_prefix(prefix, !is_last);

                print_branch(
                    "Value",
                    "",
                    indent + 1,
                    case_stmt.stmts.is_empty(),
                    &next_prefix,
                );
                case_stmt.const_expr.visualize_with_context(
                    indent + 2,
                    true,
                    &extend_prefix(&next_prefix, !case_stmt.stmts.is_empty()),
                );

                if !case_stmt.stmts.is_empty() {
                    print_branch("Statements", "", indent + 1, true, &next_prefix);
                    let stmt_prefix = extend_prefix(&next_prefix, false);
                    for (i, stmt) in case_stmt.stmts.iter().enumerate() {
                        let is_last_stmt = i == case_stmt.stmts.len() - 1;
                        stmt.visualize_with_context(indent + 2, is_last_stmt, &stmt_prefix);
                    }
                }
            }
            SwitchCase::Default(default_case) => {
                if default_case.stmts.is_empty() {
                    print_branch("Default", "(empty)", indent, is_last, prefix);
                } else {
                    print_branch("Default", "", indent, is_last, prefix);
                    let next_prefix = extend_prefix(prefix, !is_last);

                    print_branch("Statements", "", indent + 1, true, &next_prefix);
                    let stmt_prefix = extend_prefix(&next_prefix, false);
                    for (i, stmt) in default_case.stmts.iter().enumerate() {
                        let is_last_stmt = i == default_case.stmts.len() - 1;
                        stmt.visualize_with_context(indent + 2, is_last_stmt, &stmt_prefix);
                    }
                }
            }
        }
    }
}

impl Visualize for Return {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        if let Some(expr) = &self.value {
            print_branch("Return", "", indent, is_last, prefix);
            expr.visualize_with_context(indent + 1, true, &extend_prefix(prefix, !is_last));
        } else {
            print_branch("Return", "(void)", indent, is_last, prefix);
        }
    }
}

impl Visualize for Goto {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch(
            "Goto",
            &format!("→ {}", self.label.name),
            indent,
            is_last,
            prefix,
        );
    }
}

impl Visualize for Label {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch(
            "Label",
            &format!("{}:", self.name.name),
            indent,
            is_last,
            prefix,
        );
        self.stmt
            .visualize_with_context(indent + 1, true, &extend_prefix(prefix, !is_last));
    }
}

// Implementation for declarations
impl Visualize for DeclStmt {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        match self {
            DeclStmt::InitVec(inits) => {
                print_branch("DeclStmt", "InitVec", indent, is_last, prefix);
                let next_prefix = extend_prefix(prefix, !is_last);

                for (i, init) in inits.iter().enumerate() {
                    let is_last_init = i == inits.len() - 1;
                    init.visualize_with_context(indent + 1, is_last_init, &next_prefix);
                }
            }
            DeclStmt::Struct(struct_decl) => {
                struct_decl.visualize_with_context(indent, is_last, prefix);
            }
            DeclStmt::Union(union_decl) => {
                union_decl.visualize_with_context(indent, is_last, prefix);
            }
            DeclStmt::Enum(enum_decl) => {
                enum_decl.visualize_with_context(indent, is_last, prefix);
            }
            DeclStmt::Typedef(typedef) => {
                typedef.visualize_with_context(indent, is_last, prefix);
            }
        }
    }
}

impl Visualize for Init {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch("Init", &self.r.sympl.ident.name, indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        print_branch(
            "Type",
            &self.r.sympl.get_type().unwrap().to_rust_format(),
            indent + 1,
            self.l.is_none(),
            &next_prefix,
        );

        if let Some(init_data) = &self.l {
            print_branch("Initializer", "", indent + 1, true, &next_prefix);
            init_data.visualize_with_context(indent + 2, true, &extend_prefix(&next_prefix, false));
        }
    }
}

impl Visualize for InitData {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        match self {
            InitData::Expr(expr) => {
                expr.visualize_with_context(indent, is_last, prefix);
            }
            InitData::Compound(list) => {
                if list.is_empty() {
                    print_branch("Compound", "(empty)", indent, is_last, prefix);
                } else {
                    print_branch("Compound", "", indent, is_last, prefix);
                    let next_prefix = extend_prefix(prefix, !is_last);

                    for (i, init) in list.iter().enumerate() {
                        let is_last_init = i == list.len() - 1;
                        init.visualize_with_context(indent + 1, is_last_init, &next_prefix);
                    }
                }
            }
        }
    }
}

impl Visualize for Struct {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        let struct_name = self
            .ident
            .as_ref()
            .map(|n| n.name.as_str())
            .unwrap_or("anonymous");

        print_branch("StructDeclStmt", struct_name, indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        if self.member.is_empty() {
            print_branch("Members", "(empty)", indent + 1, true, &next_prefix);
        } else {
            print_branch("Members", "", indent + 1, true, &next_prefix);
            let member_prefix = extend_prefix(&next_prefix, false);

            for (i, member) in self.member.iter().enumerate() {
                let is_last_member = i == self.member.len() - 1;
                member.visualize_with_context(indent + 2, is_last_member, &member_prefix);
            }
        }
    }
}

impl Visualize for Union {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        let union_name = self
            .ident
            .as_ref()
            .map(|n| n.name.as_str())
            .unwrap_or("anonymous");

        print_branch("UnionDeclStmt", union_name, indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        if self.member.is_empty() {
            print_branch("Members", "(empty)", indent + 1, true, &next_prefix);
        } else {
            print_branch("Members", "", indent + 1, true, &next_prefix);
            let member_prefix = extend_prefix(&next_prefix, false);

            for (i, member) in self.member.iter().enumerate() {
                let is_last_member = i == self.member.len() - 1;
                member.visualize_with_context(indent + 2, is_last_member, &member_prefix);
            }
        }
    }
}

impl Visualize for Enum {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        let enum_name = self
            .ident
            .as_ref()
            .map(|n| n.name.as_str())
            .unwrap_or("anonymous");

        print_branch("EnumDeclStmt", enum_name, indent, is_last, prefix);
        let next_prefix = extend_prefix(prefix, !is_last);

        if self.variants.is_empty() {
            print_branch("Variants", "(empty)", indent + 1, true, &next_prefix);
        } else {
            print_branch("Variants", "", indent + 1, true, &next_prefix);
            let variant_prefix = extend_prefix(&next_prefix, false);

            for (i, variant) in self.variants.iter().enumerate() {
                let is_last_variant = i == self.variants.len() - 1;
                variant.visualize_with_context(indent + 2, is_last_variant, &variant_prefix);
            }
        }
    }
}

impl Visualize for EnumMember {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        let variant_info = match &self.value {
            Some(value) => format!("{} = {}", self.symbol.ident.name, value),
            None => self.symbol.ident.name.clone(),
        };
        print_branch("Variant", &variant_info, indent, is_last, prefix);
    }
}

impl Visualize for MemberDecl {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch(
            "Member",
            &format!(
                "{}: {}",
                self.sympl.ident.name,
                self.sympl.get_type().unwrap().to_rust_format()
            ),
            indent,
            is_last,
            prefix,
        );
    }
}

impl Visualize for Type {
    fn visualize(&self) {
        self.visualize_with_context(0, true, &[]);
    }

    fn visualize_with_context(&self, indent: usize, is_last: bool, prefix: &[bool]) {
        print_branch(&self.to_rust_format(), "", indent, is_last, prefix);
    }
}

// Implementation for individual expression types
impl OneLine for Binary {
    fn oneline(&self) -> String {
        format!(
            "({} {} {})",
            self.lhs.oneline(),
            self.op.to_string(),
            self.rhs.oneline()
        )
    }
}

impl OneLine for Unary {
    fn oneline(&self) -> String {
        format!("{}{}", self.op.to_string(), self.expr.oneline())
    }
}

impl OneLine for Call {
    fn oneline(&self) -> String {
        let args: Vec<String> = self.args.iter().map(|arg| arg.oneline()).collect();
        format!("{}({})", self.func.oneline(), args.join(", "))
    }
}

impl OneLine for Assign {
    fn oneline(&self) -> String {
        format!(
            "{} {} {}",
            self.lhs.oneline(),
            self.op.to_string(),
            self.rhs.oneline()
        )
    }
}

impl OneLine for Ternary {
    fn oneline(&self) -> String {
        format!(
            "{} ? {} : {}",
            self.cond.oneline(),
            self.then_branch.oneline(),
            self.else_branch.oneline()
        )
    }
}

impl OneLine for Subscript {
    fn oneline(&self) -> String {
        format!("{}[{}]", self.subject.oneline(), self.index.oneline())
    }
}

impl OneLine for MemberAccess {
    fn oneline(&self) -> String {
        let op_str = match self.kind {
            MemberAccessOp::Dot => ".",
            MemberAccessOp::MinusGreater => "->",
        };
        format!("{}{}{}", self.base.oneline(), op_str, self.member.name)
    }
}

impl OneLine for Sizeof {
    fn oneline(&self) -> String {
        match self {
            Sizeof::Type(ty) => format!("sizeof({})", ty.to_rust_format()),
            Sizeof::TypedExpr(expr) => format!("sizeof({})", expr.oneline()),
        }
    }
}

impl OneLine for Cast {
    fn oneline(&self) -> String {
        format!("({}){}", self.r#type.to_rust_format(), self.expr.oneline())
    }
}

impl OneLine for Comma {
    fn oneline(&self) -> String {
        let exprs: Vec<String> = self.assigns.iter().map(|expr| expr.oneline()).collect();
        exprs.join(", ")
    }
}

impl OneLine for Symbol {
    fn oneline(&self) -> String {
        self.ident.name.clone()
    }
}

// Main implementation for SemaExpr
impl OneLine for SemaExpr {
    fn oneline(&self) -> String {
        match self {
            SemaExpr::NumInt(n) => n.to_string(),
            SemaExpr::NumFloat(n) => n.to_string(),
            SemaExpr::Char(c) => format!("'{}'", c),
            SemaExpr::String(chars) => {
                let s: String = chars.iter().collect();
                format!("\"{}\"", s)
            }
            SemaExpr::Symbol(symbol) => symbol.oneline(),
            SemaExpr::Binary(binary) => binary.oneline(),
            SemaExpr::Unary(unary) => unary.oneline(),
            SemaExpr::Call(call) => call.oneline(),
            SemaExpr::Assign(assign) => assign.oneline(),
            SemaExpr::Ternary(ternary) => ternary.oneline(),
            SemaExpr::Subscript(subscript) => subscript.oneline(),
            SemaExpr::MemberAccess(member_access) => member_access.oneline(),
            SemaExpr::Sizeof(sizeof) => sizeof.oneline(),
            SemaExpr::Cast(cast) => cast.oneline(),
            SemaExpr::Comma(comma) => comma.oneline(),
        }
    }
}

// Implementation for TypedExpr
impl OneLine for TypedExpr {
    fn oneline(&self) -> String {
        self.r#expr.oneline()
    }
}
