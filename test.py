import re
from typing import List, Optional, Union, Tuple
from dataclasses import dataclass
from enum import Enum


@dataclass
class Ident:
    name: str


@dataclass
class Array:
    array_of: "Type"
    length: int


@dataclass
class Func:
    return_type: Optional["Type"]
    params: List["Type"]


@dataclass
class Typedef:
    type_name: Ident
    actual_type: "Type"


class BaseType(Enum):
    VOID = "void"
    INT = "int"
    DOUBLE = "double"
    CHAR = "char"


@dataclass
class Type:
    def __init__(
        self, base_type=None, pointer_level=0, array_dims=None, func_params=None
    ):
        self.base_type = base_type
        self.pointer_level = pointer_level
        self.array_dims = array_dims or []
        self.func_params = func_params

    def __repr__(self):
        result = ""

        # 関数の場合
        if self.func_params is not None:
            param_str = ", ".join(str(p) for p in self.func_params)
            result = f"func({param_str})"
            if self.base_type:
                result += f" {self.base_type.value}"
        else:
            if self.base_type:
                result = self.base_type.value

        # 配列の場合
        for dim in self.array_dims:
            result = f"[{dim}] {result}"

        # ポインタの場合
        for _ in range(self.pointer_level):
            result = f"* {result}"

        return result


class CDeclarationParser:
    def __init__(self):
        self.tokens = []
        self.pos = 0

    def tokenize(self, text: str) -> List[str]:
        # 基本的なトークナイザー
        token_pattern = r"(\w+|\*|\[|\]|\(|\)|,|;)"
        tokens = re.findall(token_pattern, text)
        return [t for t in tokens if t.strip()]

    def peek(self) -> Optional[str]:
        if self.pos < len(self.tokens):
            return self.tokens[self.pos]
        return None

    def consume(self) -> Optional[str]:
        if self.pos < len(self.tokens):
            token = self.tokens[self.pos]
            self.pos += 1
            return token
        return None

    def parse_base_type(self) -> Optional[BaseType]:
        token = self.peek()
        if token in ["void", "int", "double", "char"]:
            self.consume()
            return BaseType(token)
        return None

    def parse_declaration(self, text: str) -> Tuple[Type, str]:
        """
        C言語の宣言をパースして型と識別子を返す
        例: "int **x(int)" -> (Type, "x")
        """
        self.tokens = self.tokenize(text)
        self.pos = 0

        # ベース型をパース
        base_type = self.parse_base_type()
        if not base_type:
            raise ValueError("Invalid base type")

        # 宣言子をパース
        declarator_type, identifier = self.parse_declarator()

        # 型を結合
        final_type = self.combine_types(base_type, declarator_type)

        return final_type, identifier

    def parse_declarator(self) -> Tuple[Type, str]:
        """
        宣言子をパースする
        複雑な宣言（関数ポインタ、配列など）を処理
        """
        return self.parse_declarator_with_precedence()

    def parse_declarator_with_precedence(self) -> Tuple[Type, str]:
        """
        優先順位を考慮した宣言子のパース
        """
        # ポインタのパース
        pointer_level = 0
        while self.peek() == "*":
            self.consume()
            pointer_level += 1

        # 直接宣言子のパース
        direct_type, identifier = self.parse_direct_declarator()

        # ポインタレベルを適用
        if pointer_level > 0:
            direct_type.pointer_level += pointer_level

        return direct_type, identifier

    def parse_direct_declarator(self) -> Tuple[Type, str]:
        """
        直接宣言子をパースする
        """
        # 括弧で囲まれた宣言子の場合
        if self.peek() == "(":
            self.consume()  # "("

            # 次のトークンが型名や"*"の場合は関数パラメータ
            next_token = self.peek()
            if next_token in ["void", "int", "double", "char", ")"]:
                # 関数の場合
                self.pos -= 1  # "("を戻す
                return self.parse_function_declarator()
            else:
                # 括弧で囲まれた宣言子
                inner_type, identifier = self.parse_declarator()
                if self.peek() != ")":
                    raise ValueError("Expected ')'")
                self.consume()  # ")"

                # 後続の配列や関数をパース
                return self.parse_postfix_declarator(inner_type, identifier)

        # 識別子の場合
        identifier = self.consume()
        if not identifier or not identifier.isalpha():
            raise ValueError("Expected identifier")

        base_type = Type()
        return self.parse_postfix_declarator(base_type, identifier)

    def parse_postfix_declarator(
        self, base_type: Type, identifier: str
    ) -> Tuple[Type, str]:
        """
        後置演算子（配列、関数）をパースする
        """
        current_type = base_type

        while True:
            if self.peek() == "[":
                # 配列
                self.consume()  # "["
                size_token = self.consume()
                if not size_token or not size_token.isdigit():
                    raise ValueError("Expected array size")
                size = int(size_token)
                if self.peek() != "]":
                    raise ValueError("Expected ']'")
                self.consume()  # "]"

                current_type.array_dims.append(size)

            elif self.peek() == "(":
                # 関数
                self.consume()  # "("
                params = []

                while self.peek() != ")":
                    if self.peek() == "void":
                        self.consume()
                        break

                    param_type = self.parse_base_type()
                    if param_type:
                        params.append(Type(base_type=param_type))

                    if self.peek() == ",":
                        self.consume()

                if self.peek() != ")":
                    raise ValueError("Expected ')'")
                self.consume()  # ")"

                current_type.func_params = params

            else:
                break

        return current_type, identifier

    def parse_function_declarator(self) -> Tuple[Type, str]:
        """
        関数宣言子をパースする（識別子なし）
        """
        if self.peek() != "(":
            raise ValueError("Expected '('")

        self.consume()  # "("
        params = []

        while self.peek() != ")":
            if self.peek() == "void":
                self.consume()
                break

            param_type = self.parse_base_type()
            if param_type:
                params.append(Type(base_type=param_type))

            if self.peek() == ",":
                self.consume()

        if self.peek() != ")":
            raise ValueError("Expected ')'")
        self.consume()  # ")"

        func_type = Type(func_params=params)
        return func_type, ""

    def combine_types(self, base_type: BaseType, declarator_type: Type) -> Type:
        """
        ベース型と宣言子型を結合する
        """
        if declarator_type.func_params is not None:
            # 関数の場合
            return Type(
                base_type=base_type,
                pointer_level=declarator_type.pointer_level,
                array_dims=declarator_type.array_dims,
                func_params=declarator_type.func_params,
            )
        else:
            # 通常の変数の場合
            return Type(
                base_type=base_type,
                pointer_level=declarator_type.pointer_level,
                array_dims=declarator_type.array_dims,
            )


# 使用例とテスト
def test_parser():
    parser = CDeclarationParser()

    test_cases = [
        "int **x",
        "int *x[5]",
        "int (*x)()",
        "int x(int)",
        "int (*x[5])()",
        "void (*signal(int, void (*)(int)))(int)",
    ]

    print("C Declaration Parser Test Results:")
    print("=" * 50)

    for test in test_cases:
        try:
            result_type, identifier = parser.parse_declaration(test)
            print(f"Input: {test}")
            print(f"Identifier: {identifier}")
            print(f"Type: {result_type}")
            print("-" * 30)
        except Exception as e:
            print(f"Error parsing '{test}': {e}")
            print("-" * 30)


if __name__ == "__main__":
    test_parser()
