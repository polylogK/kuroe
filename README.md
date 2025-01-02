# kuroe

kuroe は軽量かつステートレスな競技プログラミングの作問支援ツールです。
kuroe では，テストケース生成・バリデーション・想定解生成・ジャッジが可能です。

## サブコマンド：generate

ジェネレータからテストケースを自動的に生成します。

生成ケース数はオプションで指定可能ですが，ファイル名で指定することも可能です。`kuroe.5.cpp` のようにジェネレータの拡張子の直前に数字が指定されている場合は，その数字の個数だけ生成されます（この場合 5 個）。

```bash
$ kuroe generate example/generator/
[Generate] ████████████████████    5/5
+--------+-------------------------------------------+------------------------------------------+
| status | generated_case                            | from                                     |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_000.in" | "example/generator/example_by_cpp.3.cpp" |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_001.in" | "example/generator/example_by_cpp.3.cpp" |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_002.in" | "example/generator/example_by_cpp.3.cpp" |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_py_000.in"  | "example/generator/example_by_py.py"     |
+--------+-------------------------------------------+------------------------------------------+
| OK     | "./testcases/input/example_by_txt_000.in" | "example/generator/example_by_txt.in"    |
+--------+-------------------------------------------+------------------------------------------+
```

- 引数
  - `generators`：ジェネレータを含むディレクトリ or ジェネレータへのパス（複数可能）
- オプション
  - `-r`, `--recursive`：再帰的にジェネレータを探索するかどうか。
  - `-o`, `--outdir`：出力先ディレクトリ。デフォルトは `./testcases/input`
  - `-n`, `--count`：ファイルごとに n 個生成する。デフォルトは 1。ただしファイル名で指定されている場合はファイル名が優先。
  - `-s`, `--seed`：seed, seed+1, ..., seed+(n-1)。デフォルトは 0
  - `--tl`, `--timelimit`：生成のタイムリミット（秒）。デフォルトは 10.0
  - `-l`, `--language`：カスタム言語
- 出力
  - `outdir` に入力が生成される

### 補足

ジェネレータの実行では `./a.out 0` のように `seed` が渡されます。

## サブコマンド：validate

テストケースを検証します。
`testlib.h` による `validator` を想定しています。

```bash
$ kuroe validate example/validator.cpp
[Validate] ████████████████████    5/5
+--------+-------------------------------------------+-----------------------------------------------+
| status | target                                    | stderr                                        |
+--------+-------------------------------------------+-----------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_000.in" | "./testcases/validate/example_by_cpp_000.val" |
+--------+-------------------------------------------+-----------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_001.in" | "./testcases/validate/example_by_cpp_001.val" |
+--------+-------------------------------------------+-----------------------------------------------+
| OK     | "./testcases/input/example_by_cpp_002.in" | "./testcases/validate/example_by_cpp_002.val" |
+--------+-------------------------------------------+-----------------------------------------------+
| OK     | "./testcases/input/example_by_py_000.in"  | "./testcases/validate/example_by_py_000.val"  |
+--------+-------------------------------------------+-----------------------------------------------+
| OK     | "./testcases/input/example_by_txt_000.in" | "./testcases/validate/example_by_txt_000.val" |
+--------+-------------------------------------------+-----------------------------------------------+
```

- 引数
  - `validator`：検証器へのパス（1 つ）
- オプション
  - `-t`, `--testcases`：テストケースを含むディレクトリ or テストケースへのパス（複数可能）。デフォルトは `./testcases/input`
  - `-r`, `--recursive`：再帰的にテストケースを探索するかどうか
  - `-o`, `--outdir`：エラー出力先ディレクトリ。デフォルトは `./testcases/validate`
  - `-q`, `--quiet`：エラー出力を保存しない。
  - `-l`, `--language`：カスタム言語
- 出力
  - `--quiet` が指定されていない場合，`outdir` にエラー出力が生成される。

## サブコマンド：solve

想定解コードによって answer を生成する。

```bash
$ kuroe solve example/solver/correct.cpp
[Solve] ████████████████████    5/5
+--------+---------------------------------------------+-------------------------------------------+
| status | generated_answer                            | input                                     |
+--------+---------------------------------------------+-------------------------------------------+
| OK     | "./testcases/answer/example_by_cpp_000.ans" | "./testcases/input/example_by_cpp_000.in" |
+--------+---------------------------------------------+-------------------------------------------+
| OK     | "./testcases/answer/example_by_cpp_001.ans" | "./testcases/input/example_by_cpp_001.in" |
+--------+---------------------------------------------+-------------------------------------------+
| OK     | "./testcases/answer/example_by_cpp_002.ans" | "./testcases/input/example_by_cpp_002.in" |
+--------+---------------------------------------------+-------------------------------------------+
| OK     | "./testcases/answer/example_by_py_000.ans"  | "./testcases/input/example_by_py_000.in"  |
+--------+---------------------------------------------+-------------------------------------------+
| OK     | "./testcases/answer/example_by_txt_000.ans" | "./testcases/input/example_by_txt_000.in" |
+--------+---------------------------------------------+-------------------------------------------+
```

- 引数
  - `solver`：想定解へのパス（1 つ）
- オプション
  - `-t`, `--testcases`：テストケースを含むディレクトリ or テストケースへのパス（複数可能）。デフォルトは `./testcases/input`
  - `-r`, `--recursive`：再帰的にテストケースを探索するかどうか
  - `-o`, `-outdir`：出力先ディレクトリ。デフォルトは `./testcases/answer`
  - `--tl`, `--timelimit`：生成のタイムリミット（秒）。デフォルトは 10.0
  - `-l`, `--language`：カスタム言語
- 出力
  - `outdir` に answer が生成される

## サブコマンド：judge

コードをジャッジする。
.in と .ans ファイルが揃っているケースを対象にジャッジする。

`testlib.h` による `checker` を使用することができる。

```bash
$ kuroe judge example/solver/correct.cpp # 厳密一致によるジャッジ
[Solve] ████████████████████    5/5
[Judge] ████████████████████    5/5
+--------+-------------------------------------------+---------------------------------------------+
| status | input                                     | output_and_answer                           |
+--------+-------------------------------------------+---------------------------------------------+
| AC     | "./testcases/input/example_by_cpp_000.in" | "./testcases/output/example_by_cpp_000.out" |
|        |                                           | "./testcases/answer/example_by_cpp_000.ans" |
+--------+-------------------------------------------+---------------------------------------------+
| AC     | "./testcases/input/example_by_cpp_001.in" | "./testcases/output/example_by_cpp_001.out" |
|        |                                           | "./testcases/answer/example_by_cpp_001.ans" |
+--------+-------------------------------------------+---------------------------------------------+
| AC     | "./testcases/input/example_by_cpp_002.in" | "./testcases/output/example_by_cpp_002.out" |
|        |                                           | "./testcases/answer/example_by_cpp_002.ans" |
+--------+-------------------------------------------+---------------------------------------------+
| AC     | "./testcases/input/example_by_py_000.in"  | "./testcases/output/example_by_py_000.out"  |
|        |                                           | "./testcases/answer/example_by_py_000.ans"  |
+--------+-------------------------------------------+---------------------------------------------+
| AC     | "./testcases/input/example_by_txt_000.in" | "./testcases/output/example_by_txt_000.out" |
|        |                                           | "./testcases/answer/example_by_txt_000.ans" |
+--------+-------------------------------------------+---------------------------------------------+
$ kuroe judge example/solver/correct.cpp -c example/checker.cpp # checker によるジャッジ
```

- 引数
  - `solver`：ソルバへのパス（1 つ）
- オプション
  - `-t`, `--testcase`：テストケースを含むディレクトリ（1 つ）。`.in` と `.ans` が揃っているケースのみジャッジ。再帰的に探索される。デフォルトは `./testcases`
  - `-o`, `--outdir`：ソルバ出力先ディレクトリ。デフォルトは `./testcases/output`
  - `--tl`, `--timelimit`：生成のタイムリミット（秒）。デフォルトは 2.0
- 出力
  - `outdir` にソルバの出力が生成される

## カスタム言語

kuroe はデフォルトで `C, C++, Python, Txt(txt|in)` に対応しています。
これら以外の言語を使用したい場合や，コマンドを修正したい場合はカスタム言語機能を利用することができます。

カスタム言語を使用するには `-language`, `-l` オプションを利用して，拡張子（正規表現）・コンパイルコマンド・実行コマンドを `,` で区切り指定します。以下の例を参考にしてください。

複数のカスタム言語を同時に使用することはできません（現時点）。

### 例 1：clang++ を使用する

kuroe はデフォルトでは C++ のコンパイルに g++ を利用しますが，以下のようなオプションによって clang++ によるコンパイルを行うことが可能です。
最初の要素が拡張子の指定です。二個目から最終要素の一個前までがコンパイルコマンドの指定，最終要素が実行コマンドの指定となります。

- `-l (cpp|cc),"clang++ %(target)","./a.out"`

### 例 2：pypy を使用する

コンパイルコマンドは省略可能です。

- `-l py,"pypy3 %(target)"`

## リファレンス兼謝辞

`kuroe` 実装にあたり以下を参考にしました。

- <https://rime.readthedocs.io/ja/latest/>
- <https://github.com/terry-u16/pahcer>
- <https://github.com/MikeMirzayanov/testlib/tree/master>
- <https://github.com/riantkb/testlib_for_yukicoder>

## その他？

`creating KyoUpRO problEm tool`
